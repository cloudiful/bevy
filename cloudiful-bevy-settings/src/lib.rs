#![doc = include_str!("../README.md")]

mod action;
mod schema;
mod systems;

pub use action::{RequestedSettingAction, SettingActionHandler};
pub use schema::{
    SettingControlSpec, SettingFieldSource, SettingFieldSpec, SettingSelectOption,
    SettingSliderSpec,
};
pub use systems::{
    SettingSystemSet, apply_setting_action, change_setting, register_setting_systems,
};
