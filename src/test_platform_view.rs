use crate::flutter_application::{PlatformView, PlatformViewData, PlatformViewMutation};

pub struct TestPlatformView;

impl TestPlatformView {
    pub fn new(_data: &PlatformViewData) -> Self {
        Self
    }
}

impl PlatformView for TestPlatformView {
    fn render(&mut self, offset: (f64, f64), size: (f64, f64), mutations: &[PlatformViewMutation]) {
        log::debug!("Render test platform view at {offset:?} size {size:?}: {mutations:?}");
    }
}
