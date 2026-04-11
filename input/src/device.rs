use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrimaryGamepadMode {
    #[default]
    Auto,
    Manual,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PrimaryGamepad {
    selected: Option<Entity>,
    mode: PrimaryGamepadMode,
}

impl PrimaryGamepad {
    pub fn selected(self) -> Option<Entity> {
        self.selected
    }

    pub fn mode(self) -> PrimaryGamepadMode {
        self.mode
    }

    pub fn select(&mut self, entity: Entity) {
        self.selected = Some(entity);
        self.mode = PrimaryGamepadMode::Manual;
    }

    pub fn clear_manual(&mut self) {
        self.mode = PrimaryGamepadMode::Auto;
    }

    pub(crate) fn set_selected(&mut self, entity: Option<Entity>) {
        self.selected = entity;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDevice {
    KeyboardMouse,
    Gamepad(Entity),
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ActiveInputDevice {
    current: Option<InputDevice>,
}

impl ActiveInputDevice {
    pub fn current(self) -> Option<InputDevice> {
        self.current
    }

    pub(crate) fn set_current(&mut self, current: Option<InputDevice>) {
        self.current = current;
    }
}
