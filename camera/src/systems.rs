use crate::components::SwitchableCamera;
use crate::events::{CameraSwitched, SwitchCameraRequest};
use bevy::camera::Camera;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CameraCandidate {
    entity: Entity,
    switchable: SwitchableCamera,
    is_active: bool,
}

pub(crate) fn apply_switch_camera_requests(
    mut requests: MessageReader<SwitchCameraRequest>,
    mut switched_events: MessageWriter<CameraSwitched>,
    mut cameras: ParamSet<(
        Query<(Entity, &SwitchableCamera, &Camera), With<SwitchableCamera>>,
        Query<(Entity, &mut Camera), With<SwitchableCamera>>,
    )>,
) {
    for request in requests.read().copied() {
        let ordered_cameras = ordered_candidates(&cameras.p0());
        let Some(target_camera) = resolve_target(&ordered_cameras, request) else {
            continue;
        };

        let previous = previous_active_camera(&ordered_cameras);
        if previous == Some(target_camera) && active_camera_count(&ordered_cameras) == 1 {
            continue;
        }

        let changed = activate_camera(&mut cameras.p1(), target_camera);
        if !changed {
            continue;
        }

        switched_events.write(CameraSwitched {
            previous,
            current: target_camera,
        });
    }
}

fn ordered_candidates(
    cameras: &Query<(Entity, &SwitchableCamera, &Camera), With<SwitchableCamera>>,
) -> Vec<CameraCandidate> {
    let mut ordered_cameras = cameras
        .iter()
        .map(|(entity, switchable, camera)| CameraCandidate {
            entity,
            switchable: *switchable,
            is_active: camera.is_active,
        })
        .collect::<Vec<_>>();

    ordered_cameras.sort_unstable_by_key(|candidate| {
        (
            candidate.switchable.order_key,
            candidate.switchable.slot.unwrap_or(u8::MAX),
            candidate.entity.index(),
        )
    });

    ordered_cameras
}

fn resolve_target(
    ordered_cameras: &[CameraCandidate],
    request: SwitchCameraRequest,
) -> Option<Entity> {
    match request {
        SwitchCameraRequest::ToEntity(entity) => ordered_cameras
            .iter()
            .find(|candidate| candidate.entity == entity)
            .map(|candidate| candidate.entity),
        SwitchCameraRequest::ToSlot(slot) => ordered_cameras
            .iter()
            .find(|candidate| candidate.switchable.slot == Some(slot))
            .map(|candidate| candidate.entity),
        SwitchCameraRequest::CycleNext => cycle_target(ordered_cameras, 1),
        SwitchCameraRequest::CyclePrev => cycle_target(ordered_cameras, -1),
    }
}

fn cycle_target(ordered_cameras: &[CameraCandidate], direction: isize) -> Option<Entity> {
    if ordered_cameras.is_empty() {
        return None;
    }

    let current_index = ordered_cameras
        .iter()
        .position(|candidate| candidate.is_active)
        .unwrap_or(0);

    if active_camera_count(ordered_cameras) == 0 {
        return Some(ordered_cameras[0].entity);
    }

    let len = ordered_cameras.len() as isize;
    let target_index = (current_index as isize + direction).rem_euclid(len) as usize;

    Some(ordered_cameras[target_index].entity)
}

fn active_camera_count(ordered_cameras: &[CameraCandidate]) -> usize {
    ordered_cameras
        .iter()
        .filter(|candidate| candidate.is_active)
        .count()
}

fn previous_active_camera(ordered_cameras: &[CameraCandidate]) -> Option<Entity> {
    let mut active_cameras = ordered_cameras
        .iter()
        .filter(|candidate| candidate.is_active)
        .map(|candidate| candidate.entity);
    let first = active_cameras.next()?;

    if active_cameras.next().is_some() {
        return None;
    }

    Some(first)
}

fn activate_camera(
    cameras: &mut Query<(Entity, &mut Camera), With<SwitchableCamera>>,
    target_camera: Entity,
) -> bool {
    let mut changed = false;

    for (entity, mut camera) in cameras.iter_mut() {
        let should_be_active = entity == target_camera;
        if camera.is_active != should_be_active {
            camera.is_active = should_be_active;
            changed = true;
        }
    }

    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::CameraSwitchPlugin;
    use bevy::ecs::message::Messages;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(CameraSwitchPlugin);
        app
    }

    fn spawn_switchable_camera(
        app: &mut App,
        slot: Option<u8>,
        order_key: i32,
        is_active: bool,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Camera {
                    is_active,
                    ..default()
                },
                SwitchableCamera { slot, order_key },
            ))
            .id()
    }

    fn active_cameras(app: &mut App) -> Vec<Entity> {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &Camera, &SwitchableCamera)>();
        let mut active_cameras = query
            .iter(world)
            .filter_map(|(entity, camera, _)| camera.is_active.then_some(entity))
            .collect::<Vec<_>>();
        active_cameras.sort_unstable_by_key(|entity| entity.index());
        active_cameras
    }

    fn send_request(app: &mut App, request: SwitchCameraRequest) {
        app.world_mut().write_message(request);
        app.update();
    }

    fn drain_switched_events(app: &mut App) -> Vec<CameraSwitched> {
        app.world_mut()
            .resource_mut::<Messages<CameraSwitched>>()
            .drain()
            .collect()
    }

    #[test]
    fn to_slot_activates_target_and_deactivates_others() {
        let mut app = test_app();
        let first = spawn_switchable_camera(&mut app, Some(1), 20, true);
        let second = spawn_switchable_camera(&mut app, Some(2), 10, false);

        send_request(&mut app, SwitchCameraRequest::ToSlot(2));

        assert_eq!(active_cameras(&mut app), vec![second]);
        assert_eq!(
            drain_switched_events(&mut app),
            vec![CameraSwitched {
                previous: Some(first),
                current: second,
            }]
        );
    }

    #[test]
    fn cycle_next_uses_stable_order() {
        let mut app = test_app();
        let third = spawn_switchable_camera(&mut app, Some(3), 30, false);
        let first = spawn_switchable_camera(&mut app, Some(1), 10, true);
        let second = spawn_switchable_camera(&mut app, Some(2), 20, false);

        send_request(&mut app, SwitchCameraRequest::CycleNext);

        assert_eq!(active_cameras(&mut app), vec![second]);
        assert_eq!(
            drain_switched_events(&mut app),
            vec![CameraSwitched {
                previous: Some(first),
                current: second,
            }]
        );

        send_request(&mut app, SwitchCameraRequest::CycleNext);

        assert_eq!(active_cameras(&mut app), vec![third]);
    }

    #[test]
    fn cycle_prev_uses_stable_order() {
        let mut app = test_app();
        let third = spawn_switchable_camera(&mut app, Some(3), 30, true);
        let first = spawn_switchable_camera(&mut app, Some(1), 10, false);
        let second = spawn_switchable_camera(&mut app, Some(2), 20, false);

        send_request(&mut app, SwitchCameraRequest::CyclePrev);

        assert_eq!(active_cameras(&mut app), vec![second]);
        assert_eq!(
            drain_switched_events(&mut app),
            vec![CameraSwitched {
                previous: Some(third),
                current: second,
            }]
        );

        send_request(&mut app, SwitchCameraRequest::CyclePrev);

        assert_eq!(active_cameras(&mut app), vec![first]);
    }

    #[test]
    fn cycle_without_active_camera_selects_first_candidate() {
        let mut app = test_app();
        let first = spawn_switchable_camera(&mut app, Some(2), 10, false);
        spawn_switchable_camera(&mut app, Some(1), 20, false);

        send_request(&mut app, SwitchCameraRequest::CyclePrev);

        assert_eq!(active_cameras(&mut app), vec![first]);
        assert_eq!(
            drain_switched_events(&mut app),
            vec![CameraSwitched {
                previous: None,
                current: first,
            }]
        );
    }

    #[test]
    fn to_missing_slot_keeps_current_state() {
        let mut app = test_app();
        let first = spawn_switchable_camera(&mut app, Some(1), 10, true);
        spawn_switchable_camera(&mut app, Some(2), 20, false);

        send_request(&mut app, SwitchCameraRequest::ToSlot(9));

        assert_eq!(active_cameras(&mut app), vec![first]);
        assert!(drain_switched_events(&mut app).is_empty());
    }

    #[test]
    fn to_entity_ignores_non_switchable_or_non_camera_entities() {
        let mut app = test_app();
        let active = spawn_switchable_camera(&mut app, Some(1), 10, true);
        let plain_camera = app.world_mut().spawn(Camera::default()).id();
        let plain_switchable = app
            .world_mut()
            .spawn((SwitchableCamera::default(), Transform::default()))
            .id();

        send_request(&mut app, SwitchCameraRequest::ToEntity(plain_camera));
        send_request(&mut app, SwitchCameraRequest::ToEntity(plain_switchable));

        assert_eq!(active_cameras(&mut app), vec![active]);
        assert!(drain_switched_events(&mut app).is_empty());
    }

    #[test]
    fn multiple_active_cameras_collapse_on_next_valid_switch() {
        let mut app = test_app();
        spawn_switchable_camera(&mut app, Some(1), 10, true);
        spawn_switchable_camera(&mut app, Some(2), 20, true);
        let target = spawn_switchable_camera(&mut app, Some(3), 30, false);

        send_request(&mut app, SwitchCameraRequest::ToSlot(3));

        assert_eq!(active_cameras(&mut app), vec![target]);
        assert_eq!(
            drain_switched_events(&mut app),
            vec![CameraSwitched {
                previous: None,
                current: target,
            }]
        );
    }

    #[test]
    fn switching_to_same_unique_active_camera_emits_no_event() {
        let mut app = test_app();
        let first = spawn_switchable_camera(&mut app, Some(1), 10, true);
        spawn_switchable_camera(&mut app, Some(2), 20, false);

        send_request(&mut app, SwitchCameraRequest::ToEntity(first));

        assert_eq!(active_cameras(&mut app), vec![first]);
        assert!(drain_switched_events(&mut app).is_empty());
    }
}
