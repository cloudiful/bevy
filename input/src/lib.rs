#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]

mod action_state;
mod bindings;
mod device;
mod plugin;
mod systems;

/// Per-action runtime state produced by [`InputBindingsPlugin`].
pub use action_state::{ActionData, ActionState, InputSettings};
/// Generic input action traits, bindings, and mapping storage.
pub use bindings::{InputAction, InputBinding, InputButton, InputMap};
/// Active-device and primary-gamepad tracking resources.
pub use device::{ActiveInputDevice, InputDevice, PrimaryGamepad, PrimaryGamepadMode};
/// Plugin that wires input resources and action-state updates.
pub use plugin::InputBindingsPlugin;
