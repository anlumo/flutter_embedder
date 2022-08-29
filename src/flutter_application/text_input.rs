use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
pub(super) enum TextInput {
    /// Establishes a new transaction. The arguments is
    /// a [List] whose first value is an integer representing a previously
    /// unused transaction identifier, and the second is a [String] with a
    /// JSON-encoded object with five keys, as obtained from
    /// [TextInputConfiguration.toJson]. This method must be invoked before any
    /// others (except `TextInput.hide`). See [TextInput.attach].
    #[serde(rename = "TextInput.setClient")]
    SetClient(u64, String),
    /// Show the keyboard. See [TextInputConnection.show].
    #[serde(rename = "TextInput.show")]
    Show,
    /// Update the value in the text editing
    /// control. The argument is a [String] with a JSON-encoded object with
    /// seven keys, as obtained from [TextEditingValue.toJSON]. See
    /// [TextInputConnection.setEditingState].
    #[serde(rename = "TextInput.setEditingState")]
    SetEditingState(String),
    /// End the current transaction. The next method
    /// called must be `TextInput.setClient` (or `TextInput.hide`). See
    /// [TextInputConnection.close].
    #[serde(rename = "TextInput.clearClient")]
    ClearClient,
    /// Hide the keyboard. Unlike the other methods, this can
    /// be called at any time. See [TextInputConnection.close].
    #[serde(rename = "TextInput.hide")]
    Hide,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) enum TextAffinity {
    #[serde(rename = "TextAffinity.downstream")]
    Downstream,
    #[serde(rename = "TextAffinity.upstream")]
    Upstream,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub(super) struct TextEditingValue {
    text: String,
    selection_base: Option<u64>,
    selection_extent: Option<u64>,
    selection_affinity: Option<TextAffinity>,
    selection_is_directional: Option<bool>,
    composing_base: Option<u64>,
    composing_extent: Option<u64>,
}

/// An action the user has requested the text input control to perform.
///
/// Each action represents a logical meaning, and also configures the soft
/// keyboard to display a certain kind of action button. The visual appearance
/// of the action button might differ between versions of the same OS.
///
/// Despite the logical meaning of each action, choosing a particular
/// [TextInputAction] does not necessarily cause any specific behavior to
/// happen, other than changing the focus when appropriate. It is up to the
/// developer to ensure that the behavior that occurs when an action button is
/// pressed is appropriate for the action button chosen.
///
/// For example: If the user presses the keyboard action button on iOS when it
/// reads "Emergency Call", the result should not be a focus change to the next
/// TextField. This behavior is not logically appropriate for a button that says
/// "Emergency Call".
///
/// See [EditableText] for more information about customizing action button
/// behavior.
///
/// Most [TextInputAction]s are supported equally by both Android and iOS.
/// However, there is not a complete, direct mapping between Android's IME input
/// types and iOS's keyboard return types. Therefore, some [TextInputAction]s
/// are inappropriate for one of the platforms. If a developer chooses an
/// inappropriate [TextInputAction] when running in debug mode, an error will be
/// thrown. If the same thing is done in release mode, then instead of sending
/// the inappropriate value, Android will use "unspecified" on the platform
/// side and iOS will use "default" on the platform side.
///
/// See also:
///
///  * [TextInput], which configures the platform's keyboard setup.
///  * [EditableText], which invokes callbacks when the action button is pressed.
//
// This class has been cloned to `flutter_driver/lib/src/common/action.dart` as `TextInputAction`,
// and must be kept in sync.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum TextInputAction {
    /// Logical meaning: There is no relevant input action for the current input
    /// source, e.g., [TextField].
    ///
    /// Android: Corresponds to Android's "IME_ACTION_NONE". The keyboard setup
    /// is decided by the OS. The keyboard will likely show a return key.
    ///
    /// iOS: iOS does not have a keyboard return type of "none." It is
    /// inappropriate to choose this [TextInputAction] when running on iOS.
    None,

    /// Logical meaning: Let the OS decide which action is most appropriate.
    ///
    /// Android: Corresponds to Android's "IME_ACTION_UNSPECIFIED". The OS chooses
    /// which keyboard action to display. The decision will likely be a done
    /// button or a return key.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyDefault". The title displayed in
    /// the action button is "return".
    Unspecified,

    /// Logical meaning: The user is done providing input to a group of inputs
    /// (like a form). Some kind of finalization behavior should now take place.
    ///
    /// Android: Corresponds to Android's "IME_ACTION_DONE". The OS displays a
    /// button that represents completion, e.g., a checkmark button.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyDone". The title displayed in the
    /// action button is "Done".
    Done,

    /// Logical meaning: The user has entered some text that represents a
    /// destination, e.g., a restaurant name. The "go" button is intended to take
    /// the user to a part of the app that corresponds to this destination.
    ///
    /// Android: Corresponds to Android's "IME_ACTION_GO". The OS displays a
    /// button that represents taking "the user to the target of the text they
    /// typed", e.g., a right-facing arrow button.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyGo". The title displayed in the
    /// action button is "Go".
    Go,

    /// Logical meaning: Execute a search query.
    ///
    /// Android: Corresponds to Android's "IME_ACTION_SEARCH". The OS displays a
    /// button that represents a search, e.g., a magnifying glass button.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeySearch". The title displayed in the
    /// action button is "Search".
    Search,

    /// Logical meaning: Sends something that the user has composed, e.g., an
    /// email or a text message.
    ///
    /// Android: Corresponds to Android's "IME_ACTION_SEND". The OS displays a
    /// button that represents sending something, e.g., a paper plane button.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeySend". The title displayed in the
    /// action button is "Send".
    Send,

    /// Logical meaning: The user is done with the current input source and wants
    /// to move to the next one.
    ///
    /// Moves the focus to the next focusable item in the same [FocusScope].
    ///
    /// Android: Corresponds to Android's "IME_ACTION_NEXT". The OS displays a
    /// button that represents moving forward, e.g., a right-facing arrow button.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyNext". The title displayed in the
    /// action button is "Next".
    Next,

    /// Logical meaning: The user wishes to return to the previous input source
    /// in the group, e.g., a form with multiple [TextField]s.
    ///
    /// Moves the focus to the previous focusable item in the same [FocusScope].
    ///
    /// Android: Corresponds to Android's "IME_ACTION_PREVIOUS". The OS displays a
    /// button that represents moving backward, e.g., a left-facing arrow button.
    ///
    /// iOS: iOS does not have a keyboard return type of "previous." It is
    /// inappropriate to choose this [TextInputAction] when running on iOS.
    Previous,

    /// Logical meaning: In iOS apps, it is common for a "Back" button and
    /// "Continue" button to appear at the top of the screen. However, when the
    /// keyboard is open, these buttons are often hidden off-screen. Therefore,
    /// the purpose of the "Continue" return key on iOS is to make the "Continue"
    /// button available when the user is entering text.
    ///
    /// Historical context aside, [TextInputAction.continueAction] can be used any
    /// time that the term "Continue" seems most appropriate for the given action.
    ///
    /// Android: Android does not have an IME input type of "continue." It is
    /// inappropriate to choose this [TextInputAction] when running on Android.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyContinue". The title displayed in the
    /// action button is "Continue". This action is only available on iOS 9.0+.
    ///
    /// The reason that this value has "Action" post-fixed to it is because
    /// "continue" is a reserved word in Dart, as well as many other languages.
    ContinueAction,

    /// Logical meaning: The user wants to join something, e.g., a wireless
    /// network.
    ///
    /// Android: Android does not have an IME input type of "join." It is
    /// inappropriate to choose this [TextInputAction] when running on Android.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyJoin". The title displayed in the
    /// action button is "Join".
    Join,

    /// Logical meaning: The user wants routing options, e.g., driving directions.
    ///
    /// Android: Android does not have an IME input type of "route." It is
    /// inappropriate to choose this [TextInputAction] when running on Android.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyRoute". The title displayed in the
    /// action button is "Route".
    Route,

    /// Logical meaning: Initiate a call to emergency services.
    ///
    /// Android: Android does not have an IME input type of "emergencyCall." It is
    /// inappropriate to choose this [TextInputAction] when running on Android.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyEmergencyCall". The title displayed
    /// in the action button is "Emergency Call".
    EmergencyCall,

    /// Logical meaning: Insert a newline character in the focused text input,
    /// e.g., [TextField].
    ///
    /// Android: Corresponds to Android's "IME_ACTION_NONE". The OS displays a
    /// button that represents a new line, e.g., a carriage return button.
    ///
    /// iOS: Corresponds to iOS's "UIReturnKeyDefault". The title displayed in the
    /// action button is "return".
    ///
    /// The term [TextInputAction.newline] exists in Flutter but not in Android
    /// or iOS. The reason for introducing this term is so that developers can
    /// achieve the common result of inserting new lines without needing to
    /// understand the various IME actions on Android and return keys on iOS.
    /// Thus, [TextInputAction.newline] is a convenience term that alleviates the
    /// need to understand the underlying platforms to achieve this common behavior.
    Newline,
}

/// The following incoming methods are defined for this channel (registered
/// using [MethodChannel.setMethodCallHandler]). In each case, the first argument
/// is a transaction identifier. Calls for stale transactions should be ignored.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args")]
pub(super) enum TextInputClient {
    /// The user has changed the contents
    /// of the text control. The second argument is an object with seven keys,
    /// in the form expected by [TextEditingValue.fromJSON].
    #[serde(rename = "TextInputClient.updateEditingState")]
    UpdateEditingState(u64, TextEditingValue),
    #[serde(rename = "TextInputClient.updateEditingStateWithDeltas")]
    UpdateEditingWithDeltas(u64, serde_json::Map<String, serde_json::Value>),
    /// One or more text controls
    /// were autofilled by the platform's autofill service. The first argument
    /// (the client ID) is ignored, the second argument is a map of tags to
    /// objects in the form expected by [TextEditingValue.fromJSON]. See
    /// [AutofillScope.getAutofillClient] for details on the interpretation of
    /// the tag.
    #[serde(rename = "TextInputClient.updateEditingStateWithTag")]
    UpdateEditingStateWithTag(u64, serde_json::Map<String, serde_json::Value>),
    /// The user has triggered an action.
    #[serde(rename = "TextInputClient.performAction")]
    PerformAction(u64, TextInputAction),
    #[serde(rename = "TextInputClient.performSelectors")]
    PerformSelectors(u64, Vec<serde_json::Value>),
    /// The embedding may have
    /// lost its internal state about the current editing client, if there is
    /// one. The framework should call `TextInput.setClient` and
    /// `TextInput.setEditingState` again with its most recent information. If
    /// there is no existing state on the framework side, the call should
    /// fizzle.
    #[serde(rename = "TextInputClient.requestExistingInputState")]
    RequestExistingInputState,
    #[serde(rename = "TextInputClient.updateFloatingCursor")]
    UpdateFloatingCursor(u64, String),
    /// The text input connection closed
    /// on the platform side. For example the application is moved to
    /// background or used closed the virtual keyboard. This method informs
    /// [TextInputClient] to clear connection and finalize editing.
    /// `TextInput.clearClient` and `TextInput.hide` is not called after
    /// clearing the connection since on the platform side the connection is
    /// already finalized.
    #[serde(rename = "TextInputClient.onConnectionClosed")]
    OnConnectionClosed(u64),
    #[serde(rename = "TextInputClient.showAutocorrectionPromptRect")]
    ShowAutocorrectionPromptRect(u64),
    #[serde(rename = "TextInputClient.showToolbar")]
    ShowToolbar(u64),
    #[serde(rename = "TextInputClient.insertTextPlaceholder")]
    InsertTextPlaceholder(u64, f64, f64),
    #[serde(rename = "TextInputClient.removeTextPlaceholder")]
    RemoveTextPlaceholder(u64),
}
