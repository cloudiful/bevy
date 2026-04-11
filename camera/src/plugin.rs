use crate::events::{CameraSwitched, SwitchCameraRequest};
use crate::systems::apply_switch_camera_requests;
use bevy::prelude::*;

pub struct CameraSwitchPlugin;

impl Plugin for CameraSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SwitchCameraRequest>()
            .add_message::<CameraSwitched>()
            .add_systems(Update, apply_switch_camera_requests);
    }
}
