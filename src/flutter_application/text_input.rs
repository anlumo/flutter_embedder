use serde::{Deserialize, Serialize};

// TODO: all of these method names need their enum name as a prefix
// (like `TextInput.setClient`)
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "method", content = "args")]
pub(super) enum TextInput {
    /// Establishes a new transaction. The arguments is
    /// a [List] whose first value is an integer representing a previously
    /// unused transaction identifier, and the second is a [String] with a
    /// JSON-encoded object with five keys, as obtained from
    /// [TextInputConfiguration.toJson]. This method must be invoked before any
    /// others (except `TextInput.hide`). See [TextInput.attach].
    SetClient(u64, String),
    /// Show the keyboard. See [TextInputConnection.show].
    Show,
    /// Update the value in the text editing
    /// control. The argument is a [String] with a JSON-encoded object with
    /// seven keys, as obtained from [TextEditingValue.toJSON]. See
    /// [TextInputConnection.setEditingState].
    SetEditingState(String),
    /// End the current transaction. The next method
    /// called must be `TextInput.setClient` (or `TextInput.hide`). See
    /// [TextInputConnection.close].
    ClearClient,
    /// Hide the keyboard. Unlike the other methods, this can
    /// be called at any time. See [TextInputConnection.close].
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

// TODO: all of these method names need their enum name as a prefix
// (like `TextInputClient.updateEditingState`)

/// The following incoming methods are defined for this channel (registered
/// using [MethodChannel.setMethodCallHandler]). In each case, the first argument
/// is a transaction identifier. Calls for stale transactions should be ignored.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase", tag = "method", content = "args")]
pub(super) enum TextInputClient {
    UpdateEditingState(u64, TextEditingValue),
    UpdateEditingStateWithTag(u64, serde_json::Map<String, serde_json::Value>),
    PerformAction(u64, String),
    RequestExistingInputState,
    OnConnectionClosed(u64),
}
