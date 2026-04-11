use crate::events::{CameraSwitched, SwitchCameraRequest};
use crate::systems::apply_switch_camera_requests;
use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CameraSwitchSystemSet {
    EmitRequests,
    ApplyRequests,
}

pub struct CameraSwitchPlugin;

impl Plugin for CameraSwitchPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SwitchCameraRequest>()
            .add_message::<CameraSwitched>()
            .configure_sets(
                Update,
                (
                    CameraSwitchSystemSet::EmitRequests,
                    CameraSwitchSystemSet::ApplyRequests.after(CameraSwitchSystemSet::EmitRequests),
                ),
            )
            .add_systems(
                Update,
                apply_switch_camera_requests.in_set(CameraSwitchSystemSet::ApplyRequests),
            );
    }
}
