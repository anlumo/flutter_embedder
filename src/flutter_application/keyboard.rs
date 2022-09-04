use std::{
    cell::Cell,
    ffi::CString,
    mem::size_of,
    ptr::{null, null_mut},
};

use arboard::Clipboard;
use winit::{
    event::{ElementState, KeyEvent},
    keyboard::{Key, ModifiersState},
};

use crate::{
    action_key::ActionKey,
    flutter_application::{
        text_input::TextInputClient, FlutterApplication, FLUTTER_TEXTINPUT_CHANNEL,
    },
    flutter_bindings::{
        FlutterEngine, FlutterEngineSendKeyEvent, FlutterEngineSendPlatformMessage,
        FlutterKeyEvent, FlutterKeyEventType_kFlutterKeyEventTypeDown,
        FlutterKeyEventType_kFlutterKeyEventTypeRepeat, FlutterKeyEventType_kFlutterKeyEventTypeUp,
        FlutterPlatformMessage,
    },
    keyboard_logical_key_map::translate_logical_key,
    keyboard_physical_key_map::translate_physical_key,
};

use super::text_input::{TextEditingValue, TextInput};

pub struct Keyboard {
    client: Cell<Option<u64>>,
    modifiers: ModifiersState,
    editing_state: TextEditingValue,
    clipboard: Clipboard,
}

impl Default for Keyboard {
    fn default() -> Self {
        Self {
            client: Cell::new(None),
            modifiers: Default::default(),
            editing_state: Default::default(),
            clipboard: Clipboard::new().unwrap(),
        }
    }
}

impl Keyboard {
    pub(super) fn modifiers_changed(&mut self, state: ModifiersState) {
        self.modifiers = state;
    }

    fn move_home(&mut self) {
        self.editing_state.selection_base = Some(0);
        if !self.modifiers.shift_key() {
            self.editing_state.selection_extent = Some(0);
        }
    }

    fn move_end(&mut self) {
        let len = self.editing_state.text.chars().count();
        self.editing_state.selection_extent = Some(len as _);
        if !self.modifiers.shift_key() {
            self.editing_state.selection_base = self.editing_state.selection_extent;
        }
    }

    fn insert_text(&mut self, text: &str) {
        let editing_state = &mut self.editing_state;
        let len = editing_state.text.chars().count();
        let selection_base = editing_state.selection_base.unwrap_or(0) as usize;
        let selection_extent = editing_state.selection_extent.unwrap_or(0) as usize;
        let selection = selection_base.min(selection_extent)..selection_base.max(selection_extent);

        if len > 0 && selection.start < len {
            editing_state.text.replace_range(selection.clone(), text);
            editing_state.selection_base = Some((selection.start + text.chars().count()) as _);
        } else {
            editing_state.text.push_str(text);
            editing_state.selection_base = Some(editing_state.text.chars().count() as _);
        }
        editing_state.selection_extent = editing_state.selection_base;
    }

    pub(super) fn key_event(&mut self, engine: FlutterEngine, event: KeyEvent, synthesized: bool) {
        log::debug!(
            "keyboard input: logical {:?} physical {:?} (Translated {:?}, {:?})",
            event.logical_key,
            event.physical_key,
            translate_logical_key(event.logical_key),
            translate_physical_key(event.physical_key),
        );
        if let (Some(logical), Some(physical)) = (
            translate_logical_key(event.logical_key),
            translate_physical_key(event.physical_key),
        ) {
            // let flutter_event = FlutterKeyboardEvent::Linux {
            //     r#type: match event.state {
            //         ElementState::Pressed => FlutterKeyboardEventType::KeyDown,
            //         ElementState::Released => FlutterKeyboardEventType::KeyUp,
            //     },
            //     toolkit: LinuxToolkit::Gtk,
            //     unicode_scalar_values: if let Some(character) = event.text {
            //         let mut buffer = [0u8; 8];
            //         if character.as_bytes().read(&mut buffer).is_ok() {
            //             u64::from_le_bytes(buffer)
            //         } else {
            //             0
            //         }
            //     } else {
            //         0
            //     },
            //     key_code: physical,
            //     scan_code: logical,
            //     modifiers: 0,
            //     specified_logical_key: 0,
            // };
            // let flutter_event = FlutterKeyboardEvent::Web {
            //     r#type: match event.state {
            //         ElementState::Pressed => FlutterKeyboardEventType::KeyDown,
            //         ElementState::Released => FlutterKeyboardEventType::KeyUp,
            //     },
            //     code: event.text.unwrap_or_default().to_owned(),
            //     key: event.text.unwrap_or_default().to_owned(),
            //     location: 0,
            //     meta_state: 0,
            //     key_code: 0,
            // };

            // let json = serde_json::to_vec(&flutter_event).unwrap();
            // log::debug!("keyevent: {:?}", String::from_utf8(json.clone()));
            // let channel = CStr::from_bytes_with_nul(b"flutter/keyevent\0").unwrap();
            // let message = FlutterPlatformMessage {
            //     struct_size: size_of::<FlutterPlatformMessage>() as _,
            //     channel: channel.as_ptr(),
            //     message: json.as_ptr(),
            //     message_size: json.len() as _,
            //     response_handle: null(),
            // };

            // Self::unwrap_result(unsafe { FlutterEngineSendPlatformMessage(self.engine, &message) });

            // drop(message);
            // drop(channel);

            let type_ = match event.state {
                ElementState::Pressed => {
                    if event.repeat {
                        FlutterKeyEventType_kFlutterKeyEventTypeRepeat
                    } else {
                        FlutterKeyEventType_kFlutterKeyEventTypeDown
                    }
                }
                ElementState::Released => FlutterKeyEventType_kFlutterKeyEventTypeUp,
            };
            log::debug!(
                "keyboard event: physical {physical:#x} logical {logical:#x} text {:?}",
                event.text
            );
            let character = event.text.map(|text| CString::new(text).unwrap());
            let flutter_event = FlutterKeyEvent {
                struct_size: size_of::<FlutterKeyEvent>() as _,
                timestamp: FlutterApplication::current_time() as f64,
                type_,
                physical,
                logical,
                character: if event.state == ElementState::Released {
                    null()
                } else if let Some(character) = &character {
                    character.as_ptr()
                } else {
                    null()
                },
                synthesized,
            };
            FlutterApplication::unwrap_result(unsafe {
                FlutterEngineSendKeyEvent(engine, &flutter_event, None, null_mut())
            });
            drop(character);

            log::debug!(
                "Updating editing state for keyboard client {:?}",
                self.client.get()
            );

            if event.state == ElementState::Pressed
                && self
                    .editing_state
                    .selection_base
                    .map(|val| val >= 0)
                    .unwrap_or(false)
                && self
                    .editing_state
                    .selection_extent
                    .map(|val| val >= 0)
                    .unwrap_or(false)
            {
                // send flutter/textinput message
                {
                    let editing_state = &mut self.editing_state;
                    let len = editing_state.text.chars().count();
                    let selection_base = editing_state.selection_base.unwrap_or(0) as usize;
                    let selection_extent = editing_state.selection_extent.unwrap_or(0) as usize;
                    let selection =
                        selection_base.min(selection_extent)..selection_base.max(selection_extent);
                    match event.logical_key {
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        Key::ArrowLeft if self.keyboard_modifiers.meta_key() => {
                            self.move_home();
                        }
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        Key::ArrowRight if self.keyboard_modifiers.meta_key() => {
                            self.move_end();
                        }
                        Key::ArrowLeft => {
                            if selection.start > 0 {
                                if !self.modifiers.shift_key() && selection.start != selection.end {
                                    editing_state.selection_extent = editing_state.selection_base;
                                } else {
                                    editing_state.selection_base = Some((selection.start - 1) as _);
                                    if !self.modifiers.shift_key() {
                                        editing_state.selection_extent =
                                            editing_state.selection_base;
                                    }
                                }
                            } else if !self.modifiers.shift_key()
                                && selection.start != selection.end
                            {
                                editing_state.selection_extent = editing_state.selection_base;
                            }
                        }
                        Key::ArrowRight => {
                            if selection.end < len {
                                if !self.modifiers.shift_key() && selection.start != selection.end {
                                    editing_state.selection_base = editing_state.selection_extent;
                                } else {
                                    editing_state.selection_extent = Some((selection.end + 1) as _);
                                    if !self.modifiers.shift_key() {
                                        editing_state.selection_base =
                                            editing_state.selection_extent;
                                    }
                                }
                            } else if !self.modifiers.shift_key()
                                && selection.start != selection.end
                            {
                                editing_state.selection_base = editing_state.selection_extent;
                            }
                        }
                        Key::ArrowUp | Key::Home => {
                            self.move_home();
                        }
                        Key::ArrowDown | Key::End => {
                            self.move_end();
                        }
                        Key::Backspace => {
                            if selection.start == selection.end {
                                if selection.start > 0 {
                                    editing_state.text.remove(selection.start - 1);
                                }
                                editing_state.selection_base = Some((selection.start - 1) as _);
                                editing_state.selection_extent = editing_state.selection_base;
                            } else {
                                editing_state.text.replace_range(selection.clone(), "");
                                editing_state.selection_extent = editing_state.selection_base;
                            }
                        }
                        Key::Delete => {
                            if selection.start == selection.end {
                                if selection.start < len {
                                    editing_state.text.remove(selection.start);
                                }
                            } else {
                                editing_state.text.replace_range(selection.clone(), "");
                                editing_state.selection_extent = editing_state.selection_base;
                            }
                        }
                        Key::Character("a") if self.modifiers.action_key() => {
                            editing_state.selection_base = Some(0);
                            editing_state.selection_extent = Some(len as _);
                        }
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        Key::Character("a") if self.keyboard_modifiers.control_key() => {
                            self.move_home();
                        }
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        Key::Character("e") if self.modifiers.control_key() => {
                            self.move_end();
                        }
                        Key::Character("x") if self.modifiers.action_key() => {
                            if selection.start != selection.end {
                                let text = editing_state
                                    .text
                                    .chars()
                                    .skip(selection.start)
                                    .take(selection.end - selection.start)
                                    .collect();
                                editing_state.text.replace_range(selection.clone(), "");
                                editing_state.selection_extent = editing_state.selection_base;
                                self.clipboard.set_text(text).unwrap();
                            }
                        }
                        Key::Character("c") if self.modifiers.action_key() => {
                            if selection.start != selection.end {
                                let text = editing_state
                                    .text
                                    .chars()
                                    .skip(selection.start)
                                    .take(selection.end - selection.start)
                                    .collect();
                                self.clipboard.set_text(text).unwrap();
                            }
                        }
                        Key::Character("v") if self.modifiers.action_key() => {
                            if let Ok(text) = self.clipboard.get_text() {
                                self.insert_text(&text);
                            }
                        }
                        _ if self.modifiers.control_key() || self.modifiers.super_key() => {
                            // ignore
                        }
                        _ => {
                            if let Some(text) = event.text {
                                self.insert_text(text);
                            }
                        }
                    }
                }
                self.update_editing_state(engine);
            }
        }
    }

    fn update_editing_state(&self, engine: FlutterEngine) {
        if let Some(client) = self.client.get() {
            let channel = CString::new(FLUTTER_TEXTINPUT_CHANNEL).unwrap();
            let message = TextInputClient::UpdateEditingState(client, self.editing_state.clone());
            log::info!("update_editing_state message: {message:?}");
            let message_json = serde_json::to_vec(&message).unwrap();
            FlutterApplication::unwrap_result(unsafe {
                FlutterEngineSendPlatformMessage(
                    engine,
                    &FlutterPlatformMessage {
                        struct_size: size_of::<FlutterPlatformMessage>() as _,
                        channel: channel.as_ptr(),
                        message: message_json.as_ptr(),
                        message_size: message_json.len() as _,
                        response_handle: null(),
                    },
                )
            });
            drop(channel);
        }
    }

    pub(super) fn handle_textinput_message(&mut self, textinput: TextInput) {
        match textinput {
            TextInput::SetClient(client_id, _parameters) => {
                self.client.set(Some(client_id));
                log::debug!("Setting keyboard client to {:?}", self.client.get());
            }
            TextInput::ClearClient => {
                self.client.set(None);
                log::debug!("Setting keyboard client to None");
            }
            TextInput::SetEditingState(state) => {
                log::debug!("set editing state: {:#?}", state);
                self.editing_state = state;
            }
            other => {
                log::warn!("Unhandled TextInput message: {:#?}", other);
            }
        }
    }
}
