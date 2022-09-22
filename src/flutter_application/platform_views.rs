use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    flutter_application::compositor::PlatformViewMutation,
    flutter_bindings::{
        FlutterPlatformViewMutation,
        FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeClipRect,
        FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeClipRoundedRect,
        FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeOpacity,
        FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeTransformation,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct FlutterSize {
    width: f64,
    height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "method", content = "args", rename_all = "camelCase")]
pub(super) enum PlatformViewMessage {
    Create(PlatformViewData),
    // ClearFocus {
    //     id: i64,
    // },
    // PointerEvent {
    //     id: i64,
    //     event: Value,
    // },
    Dispose(i32),
}

pub trait PlatformView: Send + 'static {
    fn render(&mut self, mutations: &[PlatformViewMutation]);
    fn clear_focus(&mut self) {}
    fn pointer_event(&mut self, _event: Value) {}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlatformViewData {
    id: i32,
    view_type: String,
    size: Option<FlutterSize>,
}

#[derive(Default)]
pub(super) struct PlatformViewsHandler {
    registered_view_types:
        HashMap<String, Box<dyn Fn(&PlatformViewData) -> Option<Box<dyn PlatformView>>>>,
    views: HashMap<i32, (PlatformViewData, Box<dyn PlatformView>)>,
}

impl PlatformViewsHandler {
    pub(super) fn handle_platform_views_message(
        &mut self,
        message: PlatformViewMessage,
    ) -> Option<Vec<u8>> {
        match message {
            PlatformViewMessage::Create(view_data) => {
                if let Some(view) = self
                    .registered_view_types
                    .get(&view_data.view_type)
                    .and_then(|generator| generator(&view_data))
                {
                    self.views.insert(view_data.id, (view_data, view));
                    Some(serde_json::to_vec(&Value::Array(vec![Value::Bool(true)])).unwrap())
                } else {
                    Some(serde_json::to_vec(&Value::Array(vec![Value::Bool(false)])).unwrap())
                }
            }
            PlatformViewMessage::Dispose(id) => {
                self.views.remove(&id);
                Some(serde_json::to_vec(&Value::Array(vec![Value::Bool(true)])).unwrap())
            }
        }
    }

    pub(super) fn render_platform_view(
        &mut self,
        id: i32,
        mutations: &[*const FlutterPlatformViewMutation],
    ) {
        let mutations: Vec<_> = mutations
            .iter()
            .filter_map(|mutation| {
                let mutation = unsafe { &**mutation };
                if mutation.type_
                    == FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeOpacity
                {
                    Some(PlatformViewMutation::Opacity(unsafe {
                        mutation.__bindgen_anon_1.opacity
                    }))
                } else if mutation.type_
                    == FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeClipRect
                {
                    Some(PlatformViewMutation::ClipRect(unsafe {
                        mutation.__bindgen_anon_1.clip_rect
                    }))
                } else if mutation.type_
                    == FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeClipRoundedRect
                {
                    Some(PlatformViewMutation::ClipRoundedRect(unsafe {
                        mutation.__bindgen_anon_1.clip_rounded_rect
                    }))
                } else if mutation.type_
                    == FlutterPlatformViewMutationType_kFlutterPlatformViewMutationTypeTransformation
                {
                    Some(PlatformViewMutation::Transformation(unsafe {
                        mutation.__bindgen_anon_1.transformation
                    }))
                } else {
                    None
                }
            })
            .collect();
        if let Some((_, view)) = self.views.get_mut(&id) {
            view.render(&mutations);
        } else {
            log::error!("Unknown platform view with identifier {id}");
        }
    }

    pub fn register_platform_view_type(
        &mut self,
        view_type: &str,
        generator: impl Fn(&PlatformViewData) -> Option<Box<dyn PlatformView>> + 'static,
    ) {
        self.registered_view_types
            .insert(view_type.to_owned(), Box::new(generator));
    }

    pub fn unregister_platform_view_type(&mut self, view_type: &str) {
        self.registered_view_types.remove(&view_type.to_owned());
    }
}
