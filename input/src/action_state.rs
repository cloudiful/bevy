use crate::bindings::InputAction;
use bevy::prelude::Resource;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ActionData {
    pub pressed: bool,
    pub just_pressed: bool,
    pub just_released: bool,
    pub value: f32,
}

impl Default for ActionData {
    fn default() -> Self {
        Self {
            pressed: false,
            just_pressed: false,
            just_released: false,
            value: 0.0,
        }
    }
}

#[derive(Resource, Debug, Clone)]
pub struct ActionState<A: InputAction> {
    actions: HashMap<A, ActionData>,
}

impl<A: InputAction> Default for ActionState<A> {
    fn default() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }
}

impl<A: InputAction> ActionState<A> {
    pub fn pressed(&self, action: A) -> bool {
        self.data(action).pressed
    }

    pub fn just_pressed(&self, action: A) -> bool {
        self.data(action).just_pressed
    }

    pub fn just_released(&self, action: A) -> bool {
        self.data(action).just_released
    }

    pub fn value(&self, action: A) -> f32 {
        self.data(action).value
    }

    pub fn data(&self, action: A) -> ActionData {
        self.actions.get(&action).copied().unwrap_or_default()
    }

    pub(crate) fn set(&mut self, action: A, data: ActionData) {
        self.actions.insert(action, data);
    }
}

#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct InputSettings {
    pub action_press_threshold: f32,
    pub axis_activity_threshold: f32,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            action_press_threshold: 0.5,
            axis_activity_threshold: 0.2,
        }
    }
}
