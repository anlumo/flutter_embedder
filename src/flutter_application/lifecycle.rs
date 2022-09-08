use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) enum LifecycleState {
    Resumed,
    Inactive,
    Paused,
    Detached,
}
