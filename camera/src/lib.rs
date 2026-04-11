mod components;
mod events;
mod plugin;
mod systems;

pub use components::SwitchableCamera;
pub use events::{CameraSwitched, SwitchCameraRequest};
pub use plugin::CameraSwitchPlugin;
