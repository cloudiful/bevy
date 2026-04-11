use bevy::prelude::*;

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchCameraRequest {
    ToEntity(Entity),
    ToSlot(u8),
    CycleNext,
    CyclePrev,
}

#[derive(Message, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraSwitched {
    pub previous: Option<Entity>,
    pub current: Entity,
}
