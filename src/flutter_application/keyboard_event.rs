use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(super) enum LinuxToolkit {
    Glfw,
    Gtk,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub(super) enum FlutterKeyboardEventType {
    KeyUp,
    KeyDown,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "keymap")]
pub(super) enum FlutterKeyboardEvent {
    Android {
        r#type: FlutterKeyboardEventType,
        /// The current set of additional flags for this event.
        ///
        /// Flags indicate things like repeat state, etc.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getFlags()>
        /// for more information.
        flags: u64,
        /// The Unicode code point represented by the key event, if any.
        ///
        /// If there is no Unicode code point, this value is zero.
        ///
        /// Dead keys are represented as Unicode combining characters.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getUnicodeChar()>
        /// for more information.
        code_point: u64,
        /// The hardware key code corresponding to this key event.
        ///
        /// This is the physical key that was pressed, not the Unicode character.
        /// See [codePoint] for the Unicode character.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getKeyCode()>
        /// for more information.
        key_code: u64,
        /// The Unicode code point represented by the key event, if any, without
        /// regard to any modifier keys which are currently pressed.
        ///
        /// If there is no Unicode code point, this value is zero.
        ///
        /// Dead keys are represented as Unicode combining characters.
        ///
        /// This is the result of calling KeyEvent.getUnicodeChar(0) on Android.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getUnicodeChar(int)>
        /// for more information.
        plain_code_point: u64,
        /// The hardware scan code id corresponding to this key event.
        ///
        /// These values are not reliable and vary from device to device, so this
        /// information is mainly useful for debugging.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getScanCode()>
        /// for more information.
        scan_code: u64,
        /// The modifiers that were present when the key event occurred.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getMetaState()>
        /// for the numerical values of the `metaState`. Many of these constants are
        /// also replicated as static constants in this class.
        ///
        /// See also:
        ///
        ///  * [modifiersPressed], which returns a Map of currently pressed modifiers
        ///    and their keyboard side.
        ///  * [isModifierPressed], to see if a specific modifier is pressed.
        ///  * [isControlPressed], to see if a CTRL key is pressed.
        ///  * [isShiftPressed], to see if a SHIFT key is pressed.
        ///  * [isAltPressed], to see if an ALT key is pressed.
        ///  * [isMetaPressed], to see if a META key is pressed.
        meta_state: u64,
        /// The source of the event.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent.html#getSource()>
        /// for the numerical values of the `source`. Many of these constants are also
        /// replicated as static constants in this class.
        event_source: u64,
        /// The vendor ID of the device that produced the event.
        ///
        /// See <https://developer.android.com/reference/android/view/InputDevice.html#getVendorId()>
        /// for the numerical values of the `vendorId`.
        vendor_id: u64,
        /// The product ID of the device that produced the event.
        ///
        /// See <https://developer.android.com/reference/android/view/InputDevice.html#getProductId()>
        /// for the numerical values of the `productId`.
        product_id: u64,
        /// The ID of the device that produced the event.
        ///
        /// See https://developer.android.com/reference/android/view/InputDevice.html#getId()
        device_id: u64,
        /// The repeat count of the event.
        ///
        /// See <https://developer.android.com/reference/android/view/KeyEvent#getRepeatCount()>
        /// for more information.
        repeat_count: u64,
    },
    Macos {
        r#type: FlutterKeyboardEventType,
        /// The Unicode characters associated with a key-up or key-down event.
        ///
        /// See also:
        ///
        ///  * [Apple's NSEvent documentation](https://developer.apple.com/documentation/appkit/nsevent/1534183-characters?language=objc)
        characters: String,
        /// The characters generated by a key event as if no modifier key (except for
        /// Shift) applies.
        ///
        /// See also:
        ///
        ///  * [Apple's NSEvent documentation](https://developer.apple.com/documentation/appkit/nsevent/1524605-charactersignoringmodifiers?language=objc)
        characters_ignoring_modifiers: String,
        /// The virtual key code for the keyboard key associated with a key event.
        ///
        /// See also:
        ///
        ///  * [Apple's NSEvent documentation](https://developer.apple.com/documentation/appkit/nsevent/1534513-keycode?language=objc)
        key_code: u64,
        /// A mask of the current modifiers using the values in Modifier Flags.
        ///
        /// See also:
        ///
        ///  * [Apple's NSEvent documentation](https://developer.apple.com/documentation/appkit/nsevent/1535211-modifierflags?language=objc)
        modifiers: u64,
        specified_logical_key: u64,
    },
    Ios {
        r#type: FlutterKeyboardEventType,
        /// The Unicode characters associated with a key-up or key-down event.
        ///
        /// See also:
        ///
        ///  * [Apple's UIKey documentation](https://developer.apple.com/documentation/uikit/uikey/3526130-characters?language=objc)
        characters: String,
        /// The characters generated by a key event as if no modifier key (except for
        /// Shift) applies.
        ///
        /// See also:
        ///
        ///  * [Apple's UIKey documentation](https://developer.apple.com/documentation/uikit/uikey/3526131-charactersignoringmodifiers?language=objc)
        characters_ignoring_modifiers: String,
        /// The virtual key code for the keyboard key associated with a key event.
        ///
        /// See also:
        ///
        ///  * [Apple's UIKey documentation](https://developer.apple.com/documentation/uikit/uikey/3526132-keycode?language=objc)
        key_code: u64,
        /// A mask of the current modifiers using the values in Modifier Flags.
        ///
        /// See also:
        ///
        ///  * [Apple's UIKey documentation](https://developer.apple.com/documentation/uikit/uikey/3526133-modifierflags?language=objc)
        modifiers: u64,
    },
    Linux {
        r#type: FlutterKeyboardEventType,
        /// There is no real concept of a "native" window toolkit on Linux, and each implementation
        /// (GLFW, GTK, QT, etc) may have a different key code mapping.
        toolkit: LinuxToolkit,
        /// An int with up to two Unicode scalar values generated by a single keystroke. An assertion
        /// will fire if more than two values are encoded in a single keystroke.
        ///
        /// This is typically the character that [keyCode] would produce without any modifier keys.
        /// For dead keys, it is typically the diacritic it would add to a character. Defaults to 0,
        /// asserted to be not null.
        unicode_scalar_values: u64,
        /// The hardware key code corresponding to this key event.
        ///
        /// This is the physical key that was pressed, not the Unicode character.
        /// This value may be different depending on the window toolkit used. See [KeyHelper].
        key_code: u64,
        /// The hardware scan code id corresponding to this key event.
        ///
        /// These values are not reliable and vary from device to device, so this
        /// information is mainly useful for debugging.
        scan_code: u64,
        /// A mask of the current modifiers using the values in Modifier Flags.
        /// This value may be different depending on the window toolkit used. See [KeyHelper].
        modifiers: u64,
        /// A logical key specified by the embedding that should be used instead of
        /// deriving from raw data.
        ///
        /// The GTK embedding detects the keyboard layout and maps some keys to
        /// logical keys in a way that can not be derived from per-key information.
        ///
        /// This is not part of the native GTK key event.
        specified_logical_key: u64,
    },
    Windows {
        r#type: FlutterKeyboardEventType,
        /// The Unicode code point represented by the key event, if any.
        ///
        /// If there is no Unicode code point, this value is zero.
        character_code_point: u64,
        /// The hardware key code corresponding to this key event.
        ///
        /// This is the physical key that was pressed, not the Unicode character.
        /// See [characterCodePoint] for the Unicode character.
        key_code: u64,
        /// The hardware scan code id corresponding to this key event.
        ///
        /// These values are not reliable and vary from device to device, so this
        /// information is mainly useful for debugging.
        scan_code: u64,
        /// A mask of the current modifiers. The modifier values must be in sync with
        /// the ones defined in https://github.com/flutter/engine/blob/master/shell/platform/windows/key_event_handler.cc
        modifiers: u64,
    },
    Web {
        r#type: FlutterKeyboardEventType,
        /// The `KeyboardEvent.code` corresponding to this event.
        ///
        /// The [code] represents a physical key on the keyboard, a value that isn't
        /// altered by keyboard layout or the state of the modifier keys.
        ///
        /// See <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code>
        /// for more information.
        code: String,
        /// The `KeyboardEvent.key` corresponding to this event.
        ///
        /// The [key] represents the key pressed by the user, taking into
        /// consideration the state of modifier keys such as Shift as well as the
        /// keyboard locale and layout.
        ///
        /// See <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/key>
        /// for more information.
        key: String,
        /// The `KeyboardEvent.location` corresponding to this event.
        ///
        /// The [location] represents the location of the key on the keyboard or other
        /// input device, such as left or right modifier keys, or Numpad keys.
        ///
        /// See <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/location>
        /// for more information.
        location: u64,
        /// The modifiers that were present when the key event occurred.
        ///
        /// See `lib/src/engine/keyboard.dart` in the web engine for the numerical
        /// values of the `metaState`. These constants are also replicated as static
        /// constants in this class.
        meta_state: u64,
        /// The `KeyboardEvent.keyCode` corresponding to this event.
        ///
        /// See <https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/keyCode>
        /// for more information.
        key_code: u64,
    },
}

// https://github.com/flutter/flutter/blob/682aa387cfe4fbd71ccd5418b2c2a075729a1c66/packages/flutter/lib/src/services/raw_keyboard_linux.dart
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[repr(u64)]
pub enum Modifiers {
    Shift = 1 << 0,
    CapsLock = 1 << 1,
    Control = 1 << 2,
    Mod1 = 1 << 3,
    Mod2 = 1 << 4,
    Meta = 1 << 26,
}
