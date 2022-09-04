use winit::keyboard::ModifiersState;

pub trait ActionKey {
    fn action_key(&self) -> bool;
}

impl ActionKey for ModifiersState {
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    fn action_key(&self) -> bool {
        self.control_key()
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    fn action_key(&self) -> bool {
        self.super_key()
    }
}
