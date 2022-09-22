use crate::flutter_application::{PlatformView, PlatformViewData, PlatformViewMutation};

pub struct TestPlatformView;

impl TestPlatformView {
    pub fn new(_data: &PlatformViewData) -> Self {
        Self
    }
}

impl PlatformView for TestPlatformView {
    fn render(&mut self, mutations: &[PlatformViewMutation]) {
        log::debug!("Render test platform view: {mutations:?}");
    }
}
