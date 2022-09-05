use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum MouseCursor {
    ActivateSystemCursor { device: i32, kind: String },
}
