mod components;
mod events;
#[cfg(feature = "input_bindings")]
mod input;
mod plugin;
mod systems;

pub use components::SwitchableCamera;
pub use events::{CameraSwitched, SwitchCameraRequest};
#[cfg(feature = "input_bindings")]
pub use input::{
    CameraGamepadBindings, CameraInputBindings, CameraInputBindingsPlugin,
    CameraSlotGamepadBinding, CameraSlotKeyBinding,
};
pub use plugin::CameraSwitchPlugin;
