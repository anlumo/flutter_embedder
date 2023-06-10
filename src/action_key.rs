use winit::event::Modifiers;

pub trait ActionKey {
    fn action_key(&self) -> bool;
}

impl ActionKey for Modifiers {
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    fn action_key(&self) -> bool {
        self.state().control_key()
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn action_key(&self) -> bool {
        self.state().super_key()
    }
}
