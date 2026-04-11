use crate::action_state::{ActionState, InputSettings};
use crate::bindings::{InputAction, InputMap};
use crate::device::{ActiveInputDevice, PrimaryGamepad};
use crate::systems::{sync_primary_gamepad, update_action_state};
use bevy::input::ButtonInput;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use std::marker::PhantomData;

pub struct InputBindingsPlugin<A: InputAction> {
    marker: PhantomData<A>,
}

impl<A: InputAction> Default for InputBindingsPlugin<A> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
        }
    }
}

impl<A: InputAction> Plugin for InputBindingsPlugin<A> {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<InputMap<A>>()
            .init_resource::<ActionState<A>>()
            .init_resource::<InputSettings>()
            .init_resource::<PrimaryGamepad>()
            .init_resource::<ActiveInputDevice>()
            .add_systems(PreUpdate, (sync_primary_gamepad, update_action_state::<A>));
    }
}
