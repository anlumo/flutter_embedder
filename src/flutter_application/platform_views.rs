use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FlutterSize {
    width: f64,
    height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args", rename_all = "camelCase")]
pub(super) enum PlatformViewMessage {
    Create(PlatformView),
    // ClearFocus {
    //     id: i64,
    // },
    // PointerEvent {
    //     id: i64,
    //     event: Value,
    // },
    Dispose(i32),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlatformView {
    id: i32,
    view_type: String,
    size: Option<FlutterSize>,
}

#[derive(Default)]
pub(super) struct PlatformViewsHandler {
    views: HashMap<i32, PlatformView>,
}

impl PlatformViewsHandler {
    pub(super) fn handle_platform_views_message(
        &mut self,
        message: PlatformViewMessage,
    ) -> Option<Vec<u8>> {
        match message {
            PlatformViewMessage::Create(view) => {
                self.views.insert(view.id, view);
                Some(serde_json::to_vec(&Value::Array(vec![Value::Bool(true)])).unwrap())
            }
            PlatformViewMessage::Dispose(id) => {
                self.views.remove(&id);
                Some(serde_json::to_vec(&Value::Array(vec![Value::Bool(true)])).unwrap())
            }
        }
    }
}
