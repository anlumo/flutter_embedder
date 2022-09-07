use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::flutter_bindings::FlutterEngine;

pub(super) struct Platform;

impl Platform {
    pub(super) fn handle_message(
        _engine: FlutterEngine,
        message: /*PlatformMessage*/ &Value,
    ) -> Option<Vec<u8>> {
        log::debug!("Platform message: {message:?}");
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
