use bevy::prelude::*;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SwitchableCamera {
    pub slot: Option<u8>,
    pub order_key: i32,
}
