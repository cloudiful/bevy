#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]

mod action;
mod schema;
mod systems;

/// Traits for app-defined settings handlers and request messages.
pub use action::{RequestedSettingAction, SettingActionHandler};
/// Generic settings schema descriptors and field-source traits.
pub use schema::{
    SettingControlSpec, SettingFieldSource, SettingFieldSpec, SettingSelectOption,
    SettingSliderSpec,
};
/// Shared settings system set and helper systems.
pub use systems::{
    SettingSystemSet, apply_setting_action, change_setting, register_setting_systems,
};
