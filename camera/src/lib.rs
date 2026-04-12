#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]

mod components;
mod events;
#[cfg(feature = "input_bindings")]
mod input;
mod plugin;
mod systems;

/// Marker/config component for cameras managed by [`CameraSwitchPlugin`].
pub use components::SwitchableCamera;
/// Switch request and switch event messages emitted by [`CameraSwitchPlugin`].
pub use events::{CameraSwitched, SwitchCameraRequest};
#[cfg(feature = "input_bindings")]
#[cfg_attr(docsrs, doc(cfg(feature = "input_bindings")))]
/// Generic Bevy-native input bindings for emitting [`SwitchCameraRequest`] values.
pub use input::{
    CameraGamepadBindings, CameraInputBindings, CameraInputBindingsPlugin,
    CameraSlotGamepadBinding, CameraSlotKeyBinding,
};
/// Plugin that applies [`SwitchCameraRequest`] values and emits [`CameraSwitched`].
pub use plugin::CameraSwitchPlugin;
