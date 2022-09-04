use std::{
    cell::Cell,
    collections::HashMap,
    ffi::{CStr, CString},
    mem::{size_of, MaybeUninit},
    os::{
        raw::{c_char, c_void},
        unix::prelude::OsStrExt,
    },
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::{Arc, Mutex},
    thread::ThreadId,
    time::Duration,
};

use arboard::Clipboard;
use ash::vk::Handle;
use log::Level;
use tokio::runtime::Runtime;
use wgpu::{Device, Instance, Queue, Surface};
use wgpu_hal::api::Vulkan;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, KeyEvent, MouseButton, MouseScrollDelta, TouchPhase},
    event_loop::EventLoopProxy,
    keyboard::{Key, ModifiersState},
};

use crate::{
    action_key::ActionKey,
    compositor::Compositor,
    flutter_application::text_input::{TextEditingValue, TextInput, TextInputClient},
    flutter_bindings::{
        FlutterCustomTaskRunners, FlutterEngine, FlutterEngineAOTData, FlutterEngineCollectAOTData,
        FlutterEngineGetCurrentTime, FlutterEngineInitialize, FlutterEngineOnVsync,
        FlutterEngineResult, FlutterEngineResult_kInternalInconsistency,
        FlutterEngineResult_kInvalidArguments, FlutterEngineResult_kInvalidLibraryVersion,
        FlutterEngineResult_kSuccess, FlutterEngineRunInitialized, FlutterEngineRunTask,
        FlutterEngineScheduleFrame, FlutterEngineSendKeyEvent, FlutterEngineSendPlatformMessage,
        FlutterEngineSendPlatformMessageResponse, FlutterEngineSendPointerEvent,
        FlutterEngineSendWindowMetricsEvent, FlutterEngineShutdown, FlutterFrameInfo,
        FlutterKeyEvent, FlutterKeyEventType_kFlutterKeyEventTypeDown,
        FlutterKeyEventType_kFlutterKeyEventTypeRepeat, FlutterKeyEventType_kFlutterKeyEventTypeUp,
        FlutterPlatformMessage, FlutterPlatformMessageResponseHandle,
        FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse, FlutterPointerEvent,
        FlutterPointerPhase, FlutterPointerPhase_kAdd, FlutterPointerPhase_kDown,
        FlutterPointerPhase_kHover, FlutterPointerPhase_kMove, FlutterPointerPhase_kRemove,
        FlutterPointerPhase_kUp, FlutterPointerSignalKind_kFlutterPointerSignalKindNone,
        FlutterPointerSignalKind_kFlutterPointerSignalKindScroll, FlutterProjectArgs,
        FlutterRendererConfig, FlutterRendererConfig__bindgen_ty_1, FlutterRendererType_kVulkan,
        FlutterSemanticsCustomAction, FlutterSemanticsNode, FlutterTask,
        FlutterTaskRunnerDescription, FlutterVulkanImage, FlutterVulkanInstanceHandle,
        FlutterVulkanRendererConfig, FlutterWindowMetricsEvent, FLUTTER_ENGINE_VERSION,
    },
    keyboard_logical_key_map::translate_logical_key,
    keyboard_physical_key_map::translate_physical_key,
    utils::flutter_asset_bundle_is_valid,
};

// mod keyboard_event;
// use keyboard_event::{FlutterKeyboardEvent, FlutterKeyboardEventType, LinuxToolkit};
mod text_input;

const PIXELS_PER_LINE: f64 = 10.0;
const FLUTTER_TEXTINPUT_CHANNEL: &str = "flutter/textinput";

struct PointerState {
    virtual_id: i32,
    position: PhysicalPosition<f64>,
    held_buttons: u64,
}

struct SendFlutterTask(FlutterTask);
unsafe impl Send for SendFlutterTask {}

struct SendFlutterPlatformMessageResponseHandle(*const FlutterPlatformMessageResponseHandle);
unsafe impl Send for SendFlutterPlatformMessageResponseHandle {}

pub type FlutterApplicationCallback = Box<dyn FnOnce(&mut FlutterApplication) + 'static + Send>;

struct FlutterApplicationUserData {
    event_loop_proxy: Mutex<EventLoopProxy<FlutterApplicationCallback>>,
    instance: Arc<Instance>,
    runtime: Arc<Runtime>,
    main_thread: ThreadId,
}

pub struct FlutterApplication {
    engine: FlutterEngine,
    compositor: Compositor,
    surface: Surface,
    instance: Arc<Instance>,
    device: Device,
    queue: Queue,
    aot_data: Vec<FlutterEngineAOTData>,
    mice: HashMap<DeviceId, PointerState>,
    current_mouse_id: i32,
    runtime: Arc<Runtime>,
    keyboard_client: Cell<Option<u64>>,
    keyboard_modifiers: ModifiersState,
    editing_state: TextEditingValue,
    clipboard: Clipboard,
    user_data: Box<FlutterApplicationUserData>,
}

impl FlutterApplication {
    pub fn new(
        runtime: Arc<Runtime>,
        asset_bundle_path: &Path,
        flutter_flags: Vec<String>,
        surface: Surface,
        instance: Arc<Instance>,
        device: Device,
        queue: Queue,
        event_loop_proxy: EventLoopProxy<FlutterApplicationCallback>,
    ) -> FlutterApplication {
        if !flutter_asset_bundle_is_valid(asset_bundle_path) {
            panic!("Flutter asset bundle was not valid.");
        }
        let mut icudtl_dat = PathBuf::new();
        icudtl_dat.push("linux");
        icudtl_dat.push("icudtl.dat");
        if !icudtl_dat.exists() {
            panic!("{icudtl_dat:?} not found.");
        }
        let (raw_instance, version, instance_extensions) = unsafe {
            instance.as_hal::<Vulkan, _, _>(|instance| {
                instance.map(|instance| {
                    let raw_instance = instance.shared_instance().raw_instance();
                    let raw_handle = raw_instance.handle().as_raw();
                    (
                        raw_handle,
                        0, // skip check, we're using 1.3 but flutter only supports up to 1.2 right now //instance.shared_instance().driver_api_version(),
                        instance
                            .shared_instance()
                            .extensions()
                            .into_iter()
                            .map(|&s| s.to_owned())
                            .collect::<Vec<CString>>(),
                    )
                })
            })
        }
        .expect("wgpu didn't choose Vulkan as rendering backend");

        let (raw_device, raw_physical_device, queue_family_index, raw_queue, device_extensions) =
            unsafe {
                device.as_hal::<Vulkan, _, _>(|device| {
                    device.map(|device| {
                        (
                            device.raw_device().handle().as_raw(),
                            device.raw_physical_device().as_raw(),
                            device.queue_family_index(),
                            device.raw_queue().as_raw(),
                            device
                                .enabled_device_extensions()
                                .into_iter()
                                .map(|&s| s.to_owned())
                                .collect::<Vec<CString>>(),
                        )
                    })
                })
            }
            .unwrap();

        let mut enabled_device_extensions: Vec<*const c_char> =
            device_extensions.iter().map(|ext| ext.as_ptr()).collect();
        let mut enabled_instance_extensions: Vec<*const c_char> =
            instance_extensions.iter().map(|ext| ext.as_ptr()).collect();

        let config = FlutterRendererConfig {
            type_: FlutterRendererType_kVulkan,
            __bindgen_anon_1: FlutterRendererConfig__bindgen_ty_1 {
                vulkan: FlutterVulkanRendererConfig {
                    struct_size: size_of::<FlutterVulkanRendererConfig>() as _,
                    version,
                    instance: raw_instance as _,
                    physical_device: raw_physical_device as _,
                    device: raw_device as _,
                    queue_family_index,
                    queue: raw_queue as _,
                    enabled_instance_extension_count: enabled_instance_extensions.len() as _,
                    enabled_instance_extensions: enabled_instance_extensions.as_mut_ptr(),
                    enabled_device_extension_count: enabled_device_extensions.len() as _,
                    enabled_device_extensions: enabled_device_extensions.as_mut_ptr(),
                    get_instance_proc_address_callback: Some(Self::instance_proc_address_callback),
                    get_next_image_callback: Some(Self::next_image),
                    present_image_callback: Some(Self::present_image),
                },
            },
        };

        let argv: Vec<CString> = flutter_flags
            .iter()
            .map(|arg| CString::new(arg.as_bytes()).unwrap())
            .collect();
        let argv_ptr: Vec<*const c_char> = argv
            .iter()
            .map(|arg| arg.as_bytes().as_ptr() as _)
            .collect();

        let user_data = Box::new(FlutterApplicationUserData {
            event_loop_proxy: Mutex::new(event_loop_proxy),
            instance: instance.clone(),
            runtime: runtime.clone(),
            main_thread: std::thread::current().id(),
        });

        let mut instance = Self {
            engine: null_mut(),
            compositor: Compositor::new(),
            surface,
            instance,
            device,
            queue,
            aot_data: vec![],
            mice: Default::default(),
            current_mouse_id: 0,
            runtime,
            keyboard_client: Cell::new(None),
            keyboard_modifiers: Default::default(),
            editing_state: Default::default(),
            clipboard: Clipboard::new().unwrap(),
            user_data,
        };

        let flutter_compositor = instance.compositor.flutter_compositor(&instance);

        let task_runner = FlutterTaskRunnerDescription {
            struct_size: size_of::<FlutterTaskRunnerDescription>() as _,
            user_data: &*instance.user_data as *const _ as _,
            runs_task_on_current_thread_callback: Some(Self::runs_task_on_current_thread_callback),
            post_task_callback: Some(Self::post_task_callback),
            identifier: 0,
        };
        let custom_task_runners = FlutterCustomTaskRunners {
            struct_size: size_of::<FlutterCustomTaskRunners>() as _,
            platform_task_runner: &task_runner,
            render_task_runner: &task_runner,
            thread_priority_setter: None,
        };

        let icu_data_path = CString::new(icudtl_dat.as_os_str().as_bytes()).unwrap();
        let mut args = unsafe { MaybeUninit::<FlutterProjectArgs>::zeroed().assume_init() };
        args.struct_size = size_of::<FlutterProjectArgs>() as _;
        args.assets_path = asset_bundle_path.as_os_str().as_bytes().as_ptr() as _;
        args.icu_data_path = icu_data_path.as_ptr() as _;
        args.command_line_argc = flutter_flags.len() as _;
        args.command_line_argv = argv_ptr.as_ptr();
        args.platform_message_callback = Some(Self::platform_message_callback);
        args.root_isolate_create_callback = Some(Self::root_isolate_create);
        args.update_semantics_node_callback = Some(Self::update_semantics_node);
        args.update_semantics_custom_action_callback = Some(Self::update_semantics_custom_action);
        args.vsync_callback = Some(Self::vsync_callback);
        args.custom_task_runners = &custom_task_runners;
        args.shutdown_dart_vm_when_done = true;
        args.compositor = &flutter_compositor as _;
        args.dart_old_gen_heap_size = -1;
        args.log_message_callback = Some(Self::log_message_callback);
        args.on_pre_engine_restart_callback = Some(Self::on_pre_engine_restart_callback);

        std::fs::create_dir("cache").ok();
        args.persistent_cache_path = b"cache".as_ptr() as _;

        Self::unwrap_result(unsafe {
            FlutterEngineInitialize(
                FLUTTER_ENGINE_VERSION.into(),
                &config as _,
                &args as _,
                &*instance.user_data as *const _ as _,
                &mut instance.engine,
            )
        });

        drop(enabled_device_extensions);
        drop(enabled_instance_extensions);
        drop(instance_extensions);
        drop(device_extensions);
        drop(flutter_compositor);
        drop(custom_task_runners);
        drop(task_runner);
        drop(argv);

        instance
    }

    pub fn run(&self) {
        Self::unwrap_result(unsafe { FlutterEngineRunInitialized(self.engine) });
    }

    pub fn metrics_changed(&self, width: u32, height: u32, pixel_ratio: f64, x: i32, y: i32) {
        self.user_data
            .event_loop_proxy
            .lock()
            .unwrap()
            .send_event(Box::new(move |application| {
                let metrics = FlutterWindowMetricsEvent {
                    struct_size: size_of::<FlutterWindowMetricsEvent>() as _,
                    width: width as _,
                    height: height as _,
                    pixel_ratio,
                    left: x.max(0) as _,
                    top: y.max(0) as _,
                    physical_view_inset_top: 0.0,
                    physical_view_inset_right: 0.0,
                    physical_view_inset_bottom: 0.0,
                    physical_view_inset_left: 0.0,
                };
                log::debug!("setting metrics to {metrics:?}");
                Self::unwrap_result(unsafe {
                    FlutterEngineSendWindowMetricsEvent(application.engine, &metrics)
                });
                drop(metrics);
            }))
            .ok()
            .unwrap();
    }

    fn get_mouse(&mut self, device_id: DeviceId) -> &mut PointerState {
        if !self.mice.contains_key(&device_id) {
            let virtual_id = self.current_mouse_id;
            self.current_mouse_id += 1;
            self.mice.insert(
                device_id,
                PointerState {
                    virtual_id,
                    position: PhysicalPosition::new(0.0, 0.0),
                    held_buttons: 0,
                },
            );
            self.send_pointer_event(device_id, FlutterPointerPhase_kAdd, None);
        }
        self.mice.get_mut(&device_id).unwrap()
    }

    pub fn mouse_buttons(&mut self, device_id: DeviceId, state: ElementState, button: MouseButton) {
        let mouse = self.get_mouse(device_id);
        let old_buttons_held = mouse.held_buttons != 0;
        let button_idx = match button {
            MouseButton::Left => 1,
            MouseButton::Right => 2,
            MouseButton::Middle => 4,
            MouseButton::Other(x) => 1 << x,
        };
        match state {
            ElementState::Pressed => mouse.held_buttons ^= button_idx,
            ElementState::Released => mouse.held_buttons &= !button_idx,
        }
        let new_buttons_held = mouse.held_buttons != 0;

        self.send_pointer_event(
            device_id,
            if state == ElementState::Pressed {
                if old_buttons_held {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kDown
                }
            } else {
                if new_buttons_held {
                    FlutterPointerPhase_kMove
                } else {
                    FlutterPointerPhase_kUp
                }
            },
            None,
        );
    }

    pub fn mouse_entered(&mut self, device_id: DeviceId) {
        self.get_mouse(device_id);
    }

    pub fn mouse_left(&mut self, device_id: DeviceId) {
        self.send_pointer_event(device_id, FlutterPointerPhase_kRemove, None);
        self.mice.remove(&device_id);
    }

    pub fn mouse_moved(&mut self, device_id: DeviceId, position: PhysicalPosition<f64>) {
        let mouse = self.get_mouse(device_id);
        mouse.position = position;
        let buttons = mouse.held_buttons;
        self.send_pointer_event(
            device_id,
            if buttons == 0 {
                FlutterPointerPhase_kHover
            } else {
                FlutterPointerPhase_kMove
            },
            None,
        );
    }

    pub fn mouse_wheel(
        &mut self,
        device_id: DeviceId,
        delta: MouseScrollDelta,
        _phase: TouchPhase,
    ) {
        let mouse = self.get_mouse(device_id);
        let buttons = mouse.held_buttons;
        self.send_pointer_event(
            device_id,
            if buttons == 0 {
                FlutterPointerPhase_kHover
            } else {
                FlutterPointerPhase_kMove
            },
            Some(delta),
        )
    }

    fn send_pointer_event(
        &self,
        device_id: DeviceId,
        phase: FlutterPointerPhase,
        scroll_delta: Option<MouseScrollDelta>,
    ) {
        if let Some(mouse) = self.mice.get(&device_id) {
            let scroll_delta_px = {
                match scroll_delta {
                    Some(MouseScrollDelta::LineDelta(x, y)) => PhysicalPosition::new(
                        (x as f64) * PIXELS_PER_LINE,
                        (y as f64) * PIXELS_PER_LINE,
                    ),
                    Some(MouseScrollDelta::PixelDelta(pt)) => pt,
                    None => PhysicalPosition::new(0.0, 0.0),
                }
            };
            let event = FlutterPointerEvent {
                struct_size: size_of::<FlutterPointerEvent>() as _,
                phase,
                timestamp: Self::current_time(),
                x: mouse.position.x,
                y: mouse.position.y,
                device: mouse.virtual_id,
                signal_kind: if scroll_delta.is_none() {
                    FlutterPointerSignalKind_kFlutterPointerSignalKindNone
                } else {
                    FlutterPointerSignalKind_kFlutterPointerSignalKindScroll
                },
                scroll_delta_x: scroll_delta_px.x,
                scroll_delta_y: scroll_delta_px.y,
                device_kind: FlutterPointerDeviceKind_kFlutterPointerDeviceKindMouse,
                buttons: mouse.held_buttons as _,
                pan_x: 0.0,
                pan_y: 0.0,
                scale: 1.0,
                rotation: 0.0,
            };
            self.user_data
                .event_loop_proxy
                .lock()
                .unwrap()
                .send_event(Box::new(move |application| {
                    Self::unwrap_result(unsafe {
                        FlutterEngineSendPointerEvent(application.engine, &event, 1)
                    });
                    drop(event);
                }))
                .ok()
                .unwrap();
        }
    }

    pub fn modifiers_changed(&mut self, state: ModifiersState) {
        self.keyboard_modifiers = state;
    }

    fn move_home(&mut self) {
        self.editing_state.selection_base = Some(0);
        if !self.keyboard_modifiers.shift_key() {
            self.editing_state.selection_extent = Some(0);
        }
    }

    fn move_end(&mut self) {
        let len = self.editing_state.text.chars().count();
        self.editing_state.selection_extent = Some(len as _);
        if !self.keyboard_modifiers.shift_key() {
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

    pub fn key_event(&mut self, _device_id: DeviceId, event: KeyEvent, synthesized: bool) {
        log::debug!("key_event: self = {:p}", self);

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
                timestamp: Self::current_time() as f64,
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
            Self::unwrap_result(unsafe {
                FlutterEngineSendKeyEvent(self.engine, &flutter_event, None, null_mut())
            });
            drop(character);

            log::debug!(
                "Updating editing state for keyboard client {:?}",
                self.keyboard_client.get()
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
                                if !self.keyboard_modifiers.shift_key()
                                    && selection.start != selection.end
                                {
                                    editing_state.selection_extent = editing_state.selection_base;
                                } else {
                                    editing_state.selection_base = Some((selection.start - 1) as _);
                                    if !self.keyboard_modifiers.shift_key() {
                                        editing_state.selection_extent =
                                            editing_state.selection_base;
                                    }
                                }
                            } else if !self.keyboard_modifiers.shift_key()
                                && selection.start != selection.end
                            {
                                editing_state.selection_extent = editing_state.selection_base;
                            }
                        }
                        Key::ArrowRight => {
                            if selection.end < len {
                                if !self.keyboard_modifiers.shift_key()
                                    && selection.start != selection.end
                                {
                                    editing_state.selection_base = editing_state.selection_extent;
                                } else {
                                    editing_state.selection_extent = Some((selection.end + 1) as _);
                                    if !self.keyboard_modifiers.shift_key() {
                                        editing_state.selection_base =
                                            editing_state.selection_extent;
                                    }
                                }
                            } else if !self.keyboard_modifiers.shift_key()
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
                        Key::Character("a") if self.keyboard_modifiers.action_key() => {
                            editing_state.selection_base = Some(0);
                            editing_state.selection_extent = Some(len as _);
                        }
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        Key::Character("a") if self.keyboard_modifiers.control_key() => {
                            self.move_home();
                        }
                        #[cfg(any(target_os = "macos", target_os = "ios"))]
                        Key::Character("e") if self.keyboard_modifiers.control_key() => {
                            self.move_end();
                        }
                        Key::Character("x") if self.keyboard_modifiers.action_key() => {
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
                        Key::Character("c") if self.keyboard_modifiers.action_key() => {
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
                        Key::Character("v") if self.keyboard_modifiers.action_key() => {
                            if let Ok(text) = self.clipboard.get_text() {
                                self.insert_text(&text);
                            }
                        }
                        _ if self.keyboard_modifiers.control_key()
                            || self.keyboard_modifiers.super_key() =>
                        {
                            // ignore
                        }
                        _ => {
                            if let Some(text) = event.text {
                                self.insert_text(text);
                            }
                        }
                    }
                }
                self.update_editing_state();
            }
        }
    }

    fn update_editing_state(&self) {
        if let Some(keyboard_client) = self.keyboard_client.get() {
            let channel = CString::new(FLUTTER_TEXTINPUT_CHANNEL).unwrap();
            let message =
                TextInputClient::UpdateEditingState(keyboard_client, self.editing_state.clone());
            log::info!("update_editing_state message: {message:?}");
            let message_json = serde_json::to_vec(&message).unwrap();
            Self::unwrap_result(unsafe {
                FlutterEngineSendPlatformMessage(
                    self.engine,
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

    pub fn schedule_frame(&self) {
        Self::unwrap_result(unsafe { FlutterEngineScheduleFrame(self.engine) });
    }

    pub fn surface(&self) -> &Surface {
        &self.surface
    }
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
    pub fn device(&self) -> &Device {
        &self.device
    }
    pub fn queue(&self) -> &Queue {
        &self.queue
    }

    pub fn current_time() -> u64 {
        unsafe { FlutterEngineGetCurrentTime() }
    }

    extern "C" fn platform_message_callback(
        message: *const FlutterPlatformMessage,
        user_data: *mut c_void,
    ) {
        log::debug!("platform_message_callback");
        let message = unsafe { &*message };
        let channel = unsafe { CStr::from_ptr(message.channel) }
            .to_str()
            .to_owned();
        let user_data = unsafe { &*(user_data as *const FlutterApplicationUserData) };
        let response_handle = SendFlutterPlatformMessageResponseHandle(message.response_handle);
        let data =
            unsafe { std::slice::from_raw_parts(message.message, message.message_size as _) }
                .to_vec();
        user_data.event_loop_proxy.lock().unwrap().send_event(Box::new(move |this| {
            if let Ok(channel) = channel {
                if channel == "flutter/textinput" {
                    if let Ok(text_input) = serde_json::from_slice::<TextInput>(&data) {
                        match text_input {
                            TextInput::SetClient(client_id, _parameters) => {
                                this.keyboard_client.set(Some(client_id));
                                log::debug!(
                                    "Setting keyboard client to {:?}",
                                    this.keyboard_client.get()
                                );
                            }
                            TextInput::ClearClient => {
                                this.keyboard_client.set(None);
                                log::debug!("Setting keyboard client to None");
                            }
                            TextInput::SetEditingState(state) => {
                                log::debug!("set editing state: {:#?}", state);
                                this.editing_state = state;
                            }
                            other => {
                                log::warn!("Unhandled TextInput message: {:#?}", other);
                            }
                        }
                    } else {
                        log::debug!("Unknown textinput message: {:?}", std::str::from_utf8(&data));
                    }
                    Self::unwrap_result(unsafe {
                        FlutterEngineSendPlatformMessageResponse(
                            this.engine,
                            response_handle.0,
                            null(),
                            0,
                        )
                    });
                } else {
                        log::debug!(
                        "Unhandled platform message: channel = {channel}, message size = {}, message: {:?}",
                        data.len(),
                        data,
                    );

                    Self::unwrap_result(unsafe {
                        FlutterEngineSendPlatformMessageResponse(
                            this.engine,
                            response_handle.0,
                            null(),
                            0,
                        )
                    });
                }
            } else {
                Self::unwrap_result(unsafe {
                    FlutterEngineSendPlatformMessageResponse(
                        this.engine,
                        response_handle.0,
                        null(),
                        0,
                    )
                });
            }
            drop(response_handle);
        })).ok().unwrap();
    }

    extern "C" fn root_isolate_create(_user_data: *mut c_void) {
        log::trace!("root_isolate_create");
    }

    extern "C" fn update_semantics_node(
        semantics_node: *const FlutterSemanticsNode,
        _user_data: *mut c_void,
    ) {
        log::trace!("update_semantics_node {:?}", unsafe { *semantics_node });
    }
    extern "C" fn update_semantics_custom_action(
        semantics_custom_action: *const FlutterSemanticsCustomAction,
        _user_data: *mut c_void,
    ) {
        log::trace!("update_semantics_custom_action {:?}", unsafe {
            *semantics_custom_action
        });
    }

    extern "C" fn vsync_callback(user_data: *mut c_void, baton: isize) {
        let user_data = unsafe { &*(user_data as *const FlutterApplicationUserData) };

        user_data
            .event_loop_proxy
            .lock()
            .unwrap()
            .send_event(Box::new(move |this| {
                this.device().poll(wgpu::Maintain::Wait);
                let time = Self::current_time();
                Self::unwrap_result(unsafe {
                    FlutterEngineOnVsync(this.engine, baton, time, time + 1000000000 / 60)
                });
            }))
            .ok()
            .unwrap();
    }

    extern "C" fn on_pre_engine_restart_callback(_user_data: *mut c_void) {
        todo!()
    }

    extern "C" fn log_message_callback(
        tag: *const c_char,
        message: *const c_char,
        _user_data: *mut c_void,
    ) {
        let tag = unsafe { CStr::from_ptr(tag) };
        let message = unsafe { CStr::from_ptr(message) };
        log::logger().log(
            &log::Record::builder()
                .level(Level::Info)
                .module_path(tag.to_str().ok())
                .args(format_args!("{}", message.to_str().unwrap()))
                .build(),
        );
    }

    extern "C" fn instance_proc_address_callback(
        user_data: *mut c_void,
        _instance: FlutterVulkanInstanceHandle,
        name: *const c_char,
    ) -> *mut c_void {
        let user_data = unsafe { &*(user_data as *const FlutterApplicationUserData) };

        let result = unsafe {
            user_data.instance.as_hal::<Vulkan, _, _>(|instance| {
                instance.and_then(|instance| {
                    let shared = instance.shared_instance();
                    let entry = shared.entry();
                    let cname = CStr::from_ptr(name);
                    if cname == CStr::from_bytes_with_nul(b"vkCreateInstance\0").unwrap() {
                        Some(entry.fp_v1_0().create_instance as *mut c_void)
                    } else if cname
                        == CStr::from_bytes_with_nul(b"vkCreateDebugReportCallbackEXT\0").unwrap()
                    {
                        None
                    } else if cname
                        == CStr::from_bytes_with_nul(b"vkEnumerateInstanceExtensionProperties\0")
                            .unwrap()
                    {
                        Some(entry.fp_v1_0().enumerate_instance_extension_properties as *mut c_void)
                    } else if cname
                        == CStr::from_bytes_with_nul(b"vkEnumerateInstanceLayerProperties\0")
                            .unwrap()
                    {
                        Some(entry.fp_v1_0().enumerate_instance_layer_properties as *mut c_void)
                    } else {
                        entry
                            .get_instance_proc_addr(shared.raw_instance().handle(), name)
                            .map(|f| f as *mut c_void)
                    }
                })
            })
        }
        .unwrap_or_else(null_mut);
        log::trace!(
            "instance_proc_address_callback: {} -> {:?}",
            unsafe { CStr::from_ptr(name) }.to_str().unwrap(),
            result,
        );
        result
    }

    extern "C" fn next_image(
        _user_data: *mut c_void,
        _frame_info: *const FlutterFrameInfo,
    ) -> FlutterVulkanImage {
        unimplemented!()
        // Not used if a FlutterCompositor is supplied in FlutterProjectArgs.
    }

    extern "C" fn present_image(
        _user_data: *mut c_void,
        _image: *const FlutterVulkanImage,
    ) -> bool {
        unimplemented!()
        // Not used if a FlutterCompositor is supplied in FlutterProjectArgs.
    }

    extern "C" fn runs_task_on_current_thread_callback(user_data: *mut c_void) -> bool {
        let user_data = unsafe { &*(user_data as *const FlutterApplicationUserData) };
        user_data.main_thread == std::thread::current().id()
    }

    extern "C" fn post_task_callback(
        task: FlutterTask,
        target_time_nanos: u64,
        user_data: *mut c_void,
    ) {
        let user_data = unsafe { &*(user_data as *const FlutterApplicationUserData) };
        let task = SendFlutterTask(task);

        if Self::current_time() >= target_time_nanos {
            user_data
                .event_loop_proxy
                .lock()
                .unwrap()
                .send_event(Box::new(move |application| unsafe {
                    Self::unwrap_result(FlutterEngineRunTask(application.engine, &task.0));
                    drop(task);
                }))
                .ok()
                .unwrap();
        } else {
            let event_loop_proxy = user_data.event_loop_proxy.lock().unwrap().clone();
            user_data.runtime.spawn(async move {
                tokio::time::sleep(Duration::from_nanos(
                    target_time_nanos - Self::current_time(),
                ))
                .await;

                event_loop_proxy
                    .send_event(Box::new(move |application| unsafe {
                        Self::unwrap_result(FlutterEngineRunTask(application.engine, &task.0));
                        drop(task);
                    }))
                    .ok()
                    .unwrap();
            });
        }
    }

    fn unwrap_result(result: FlutterEngineResult) {
        #[allow(non_upper_case_globals)]
        match result {
            x if x == FlutterEngineResult_kSuccess => {}
            x if x == FlutterEngineResult_kInvalidLibraryVersion => {
                panic!("Invalid library version.");
            }
            x if x == FlutterEngineResult_kInvalidArguments => {
                panic!("Invalid arguments.");
            }
            x if x == FlutterEngineResult_kInternalInconsistency => {
                panic!("Internal inconsistency.");
            }
            x => {
                panic!("Unknown error {x}.");
            }
        }
    }
}

impl Drop for FlutterApplication {
    fn drop(&mut self) {
        Self::unwrap_result(unsafe { FlutterEngineShutdown(self.engine) });
        for &aot_data in &self.aot_data {
            unsafe {
                FlutterEngineCollectAOTData(aot_data);
            }
        }
    }
}
