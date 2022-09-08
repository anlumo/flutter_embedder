use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use winit::{
    event_loop::EventLoopProxy,
    window::{Fullscreen, UserAttentionType},
};

use crate::flutter_bindings::FlutterEngine;

use super::FlutterApplication;

pub(super) struct Platform;

impl Platform {
    pub(super) fn handle_message(
        _engine: FlutterEngine,
        message: PlatformMessage,
        application: &FlutterApplication,
    ) -> Option<Vec<u8>> {
        log::debug!("Platform message: {message:?}");
        match message {
            PlatformMessage::SystemChromeSetApplicationSwitcherDescription { label, .. } => {
                application.window.set_title(&label);
            }
            PlatformMessage::ClipboardSetData { text } => {
                application
                    .clipboard
                    .lock()
                    .unwrap()
                    .set_text(text)
                    .expect("Failed setting clipboard");
            }
            PlatformMessage::ClipboardGetData(_) => {
                let text = application
                    .clipboard
                    .lock()
                    .unwrap()
                    .get_text()
                    .expect("Failed reading clipboard");
                return Some(
                    serde_json::to_vec(&serde_json::json!({
                        "text": text,
                    }))
                    .unwrap(),
                );
            }
            PlatformMessage::ClipboardHasStrings(_) => {
                let has_strings = application.clipboard.lock().unwrap().get_text().is_ok();
                return Some(
                    serde_json::to_vec(&serde_json::json!({
                        "value": has_strings,
                    }))
                    .unwrap(),
                );
            }
            PlatformMessage::HapticFeedbackVibrate(feedback_type) => match feedback_type {
                HapticFeedbackType::LightImpact => {}
                HapticFeedbackType::MediumImpact => application
                    .window
                    .request_user_attention(Some(UserAttentionType::Informational)),
                HapticFeedbackType::HeavyImpact => application
                    .window
                    .request_user_attention(Some(UserAttentionType::Critical)),
                HapticFeedbackType::SelectionClick => {}
            },
            PlatformMessage::SystemSoundPlay(_) => {
                application
                    .window
                    .request_user_attention(Some(UserAttentionType::Critical));
            }
            PlatformMessage::SystemNavigatorPop => {
                application
                    .user_data
                    .event_loop_proxy
                    .lock()
                    .unwrap()
                    .send_event(|_| true)
                    .unwrap();
            }
            PlatformMessage::SystemChromeSetEnabledSystemUIMode(mode) => {
                if mode == SystemUiMode::Manual {
                    application.window.set_fullscreen(None);
                } else {
                    application
                        .window
                        .set_fullscreen(Some(Fullscreen::Borderless(None)));
                }
            }
            _ => {}
        }
        None
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
pub(super) enum PlatformMessage {
    /// Places the data from the `text` entry of the argument.
    #[serde(rename = "Clipboard.setData")]
    ClipboardSetData { text: String },
    /// Returns the data that has the format specified in
    /// the argument, a [String], from the system clipboard.
    #[serde(rename = "Clipboard.getData")]
    ClipboardGetData(ClipboardFormat),
    /// Should return `{"value":true}` iff the clipboard contains string data,
    /// otherwise `{"value":false}`.
    #[serde(rename = "Clipboard.hasStrings")]
    ClipboardHasStrings(ClipboardFormat),
    /// Triggers a system-default haptic response.
    #[serde(rename = "HapticFeedback.vibrate")]
    HapticFeedbackVibrate(HapticFeedbackType),
    /// Triggers a system audio effect. The argument must
    /// be a [String] describing the desired effect
    #[serde(rename = "SystemSound.play")]
    SystemSoundPlay(String),
    /// Informs the operating system of the desired orientation of the display.
    #[serde(rename = "SystemChrome.setPreferredOrientations")]
    SystemChromeSetPreferredOrientations(Vec<DeviceOrientation>),
    /// Informs the operating system of the desired label and color to be used
    /// to describe the application in any system-level application lists (e.g.
    /// application switchers).
    #[serde(rename = "SystemChrome.setApplicationSwitcherDescription")]
    SystemChromeSetApplicationSwitcherDescription {
        /// A label and description of the current state of the application.
        label: String,
        /// The application's primary color.
        ///
        /// This may influence the color that the operating system uses to represent
        /// the application.
        ///
        /// A 32 bit integer value
        /// (the lower eight bits being the blue channel, the next eight bits being
        /// the green channel, the next eight bits being the red channel, and the
        /// high eight bits being set, as from [Color.value] for an opaque color).
        /// The `primaryColor` can also be zero to indicate that the system default
        /// should be used.
        #[serde(rename = "primaryColor")]
        primary_color: u32,
    },
    /// Specifies the set of system overlays to have visible when the application
    /// is running.
    #[serde(rename = "SystemChrome.setEnabledSystemUIOverlays")]
    SystemChromeSetEnabledSystemUIOverlays(Vec<SystemUiOverlay>),
    /// Specifies the [SystemUiMode] for the application.
    #[serde(rename = "SystemChrome.setEnabledSystemUIMode")]
    SystemChromeSetEnabledSystemUIMode(SystemUiMode),
    /// Specifies whether system overlays (e.g. the status bar on Android or iOS)
    /// should be `light` or `dark`.
    #[serde(rename = "SystemChrome.setSystemUIOverlayStyle")]
    SystemChromeSetEnabledSystemUIOverlayStyle(SystemUiOverlayStyle),
    /// Tells the operating system to close the application, or the closest
    /// equivalent.
    #[serde(rename = "SystemNavigator.pop")]
    SystemNavigatorPop,
    /// Undocumented but sent when a listener for the event below is registered
    #[serde(rename = "SystemChrome.setSystemUIChangeListener")]
    SystemChromeSetSystemUIChangeListener,
    /// Outgoing. The user has changed the visibility of
    /// the system overlays. This is relevant when using [SystemUiMode]s
    /// through [SystemChrome.setEnabledSystemUIMode].
    /// The boolean indicates whether the system overlays are visible
    /// (meaning that the application is not in fullscreen).
    #[serde(rename = "SystemChrome.systemUIChange")]
    SystemChromeSystemUIChange((bool)),
    /// Restores the system overlays to the last settings provided via
    /// [SystemChromeSetEnabledSystemUIOverlays]. May be used when the platform force
    /// enables/disables UI elements.
    ///
    /// For example, when the Android keyboard disables hidden status and navigation bars,
    /// this can be called to re-disable the bars when the keyboard is closed.
    ///
    /// On Android, the system UI cannot be changed until 1 second after the previous
    /// change. This is to prevent malware from permanently hiding navigation buttons.
    #[serde(rename = "SystemChrome.restoreSystemUIOverlays")]
    SystemChromeRestoreSystemUIOverlays,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum ClipboardFormat {
    #[serde(rename = "text/plain")]
    TextPlain,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum HapticFeedbackType {
    #[serde(rename = "HapticFeedbackType.lightImpact")]
    LightImpact,
    #[serde(rename = "HapticFeedbackType.mediumImpact")]
    MediumImpact,
    #[serde(rename = "HapticFeedbackType.heavyImpact")]
    HeavyImpact,
    #[serde(rename = "HapticFeedbackType.selectionClick")]
    SelectionClick,
}

/// Specifies a particular device orientation.
///
/// To determine which values correspond to which orientations, first position
/// the device in its default orientation (this is the orientation that the
/// system first uses for its boot logo, or the orientation in which the
/// hardware logos or markings are upright, or the orientation in which the
/// cameras are at the top). If this is a portrait orientation, then this is
/// [portraitUp]. Otherwise, it's [landscapeLeft]. As you rotate the device by
/// 90 degrees in a counter-clockwise direction around the axis that pierces the
/// screen, you step through each value in this enum in the order given.
///
/// For a device with a landscape default orientation, the orientation obtained
/// by rotating the device 90 degrees clockwise from its default orientation is
/// [portraitUp].
///
/// Used by [SystemChrome.setPreferredOrientations].
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum DeviceOrientation {
    /// If the device shows its boot logo in portrait, then the boot logo is shown
    /// in [portraitUp]. Otherwise, the device shows its boot logo in landscape
    /// and this orientation is obtained by rotating the device 90 degrees
    /// clockwise from its boot orientation.
    PortraitUp,
    /// The orientation that is 90 degrees clockwise from [PortraitUp].
    ///
    /// If the device shows its boot logo in landscape, then the boot logo is
    /// shown in [LandscapeLeft].
    LandscapeLeft,
    /// The orientation that is 180 degrees from [PortraitUp].
    PortraitDown,
    /// The orientation that is 90 degrees counterclockwise from [PortraitUp].
    LandscapeRight,
}

/// Specifies a system overlay at a particular location.
///
/// Used by [SystemChrome.setEnabledSystemUIOverlays].
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum SystemUiOverlay {
    /// The status bar provided by the embedder on the top of the application
    /// surface, if any.
    Top,
    /// The status bar provided by the embedder on the bottom of the application
    /// surface, if any.
    Bottom,
}

/// Describes different display configurations for both Android and iOS.
///
/// These modes mimic Android-specific display setups.
///
/// Used by [SystemChrome.setEnabledSystemUIMode].
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum SystemUiMode {
    /// Fullscreen display with status and navigation bars presentable by tapping
    /// anywhere on the display.
    ///
    /// Available starting at SDK 16 or Android J. Earlier versions of Android
    /// will not be affected by this setting.
    ///
    /// For applications running on iOS, the status bar and home indicator will be
    /// hidden for a similar fullscreen experience.
    ///
    /// Tapping on the screen displays overlays, this gesture is not received by
    /// the application.
    ///
    /// See also:
    ///
    ///   * [SystemUiChangeCallback], used to listen and respond to the change in
    ///     system overlays.
    LeanBack,

    /// Fullscreen display with status and navigation bars presentable through a
    /// swipe gesture at the edges of the display.
    ///
    /// Available starting at SDK 19 or Android K. Earlier versions of Android
    /// will not be affected by this setting.
    ///
    /// For applications running on iOS, the status bar and home indicator will be
    /// hidden for a similar fullscreen experience.
    ///
    /// A swipe gesture from the edge of the screen displays overlays. In contrast
    /// to [SystemUiMode.immersiveSticky], this gesture is not received by the
    /// application.
    ///
    /// See also:
    ///
    ///   * [SystemUiChangeCallback], used to listen and respond to the change in
    ///     system overlays.
    Immersive,

    /// Fullscreen display with status and navigation bars presentable through a
    /// swipe gesture at the edges of the display.
    ///
    /// Available starting at SDK 19 or Android K. Earlier versions of Android
    /// will not be affected by this setting.
    ///
    /// For applications running on iOS, the status bar and home indicator will be
    /// hidden for a similar fullscreen experience.
    ///
    /// A swipe gesture from the edge of the screen displays overlays. In contrast
    /// to [SystemUiMode.immersive], this gesture is received by the application.
    ///
    /// See also:
    ///
    ///   * [SystemUiChangeCallback], used to listen and respond to the change in
    ///     system overlays.
    ImmersiveSticky,

    /// Fullscreen display with status and navigation elements rendered over the
    /// application.
    ///
    /// Available starting at SDK 29 or Android 10. Earlier versions of Android
    /// will not be affected by this setting.
    ///
    /// For applications running on iOS, the status bar and home indicator will be
    /// visible.
    ///
    /// The system overlays will not disappear or reappear in this mode as they
    /// are permanently displayed on top of the application.
    ///
    /// See also:
    ///
    ///   * [SystemUiOverlayStyle], can be used to configure transparent status and
    ///     navigation bars with or without a contrast scrim.
    EdgeToEdge,

    /// Declares manually configured [SystemUiOverlay]s.
    ///
    /// When using this mode with [SystemChrome.setEnabledSystemUIMode], the
    /// preferred overlays must be set by the developer.
    ///
    /// When [SystemUiOverlay.top] is enabled, the status bar will remain visible
    /// on all platforms. Omitting this overlay will hide the status bar on iOS &
    /// Android.
    ///
    /// When [SystemUiOverlay.bottom] is enabled, the navigation bar and home
    /// indicator of Android and iOS applications will remain visible. Omitting this
    /// overlay will hide them.
    ///
    /// Omitting both overlays will result in the same configuration as
    /// [SystemUiMode.leanBack].
    Manual,
}

/// Specifies a preference for the style of the system overlays.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum SystemUiOverlayStyle {
    /// System overlays should be drawn with a light color. Intended for
    /// applications with a dark background.
    Light,
    /// System overlays should be drawn with a dark color. Intended for
    /// applications with a light background.
    Dark,
}
