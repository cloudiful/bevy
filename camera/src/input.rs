use crate::events::SwitchCameraRequest;
use crate::plugin::CameraSwitchSystemSet;
use bevy::input::ButtonInput;
use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraSlotKeyBinding {
    pub slot: u8,
    pub key: KeyCode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CameraSlotGamepadBinding {
    pub slot: u8,
    pub button: GamepadButton,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CameraGamepadBindings {
    pub slots: Vec<CameraSlotGamepadBinding>,
    pub next: Vec<GamepadButton>,
    pub prev: Vec<GamepadButton>,
}

impl CameraGamepadBindings {
    pub fn bind_slot(mut self, button: GamepadButton, slot: u8) -> Self {
        self.slots.push(CameraSlotGamepadBinding { slot, button });
        self
    }

    pub fn bind_next(mut self, button: GamepadButton) -> Self {
        self.next.push(button);
        self
    }

    pub fn bind_prev(mut self, button: GamepadButton) -> Self {
        self.prev.push(button);
        self
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Default)]
pub struct CameraInputBindings {
    pub slots: Vec<CameraSlotKeyBinding>,
    pub next: Vec<KeyCode>,
    pub prev: Vec<KeyCode>,
    pub gamepad: Option<CameraGamepadBindings>,
}

impl CameraInputBindings {
    pub fn bind_slot(mut self, key: KeyCode, slot: u8) -> Self {
        self.slots.push(CameraSlotKeyBinding { slot, key });
        self
    }

    pub fn bind_next(mut self, key: KeyCode) -> Self {
        self.next.push(key);
        self
    }

    pub fn bind_prev(mut self, key: KeyCode) -> Self {
        self.prev.push(key);
        self
    }

    pub fn with_gamepad(mut self, gamepad: CameraGamepadBindings) -> Self {
        self.gamepad = Some(gamepad);
        self
    }
}

pub struct CameraInputBindingsPlugin;

impl Plugin for CameraInputBindingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<CameraInputBindings>()
            .add_systems(
                Update,
                emit_switch_camera_requests_from_input.in_set(CameraSwitchSystemSet::EmitRequests),
            );
    }
}

fn emit_switch_camera_requests_from_input(
    bindings: Res<CameraInputBindings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut requests: MessageWriter<SwitchCameraRequest>,
) {
    let Some(request) = resolve_input_request(&bindings, &keyboard, &gamepads) else {
        return;
    };

    requests.write(request);
}

fn resolve_input_request(
    bindings: &CameraInputBindings,
    keyboard: &ButtonInput<KeyCode>,
    gamepads: &Query<&Gamepad>,
) -> Option<SwitchCameraRequest> {
    if let Some(slot) = bindings
        .slots
        .iter()
        .find(|binding| keyboard.just_pressed(binding.key))
        .map(|binding| binding.slot)
    {
        return Some(SwitchCameraRequest::ToSlot(slot));
    }

    if keyboard.any_just_pressed(bindings.next.iter().copied()) {
        return Some(SwitchCameraRequest::CycleNext);
    }

    if keyboard.any_just_pressed(bindings.prev.iter().copied()) {
        return Some(SwitchCameraRequest::CyclePrev);
    }

    let Some(gamepad) = &bindings.gamepad else {
        return None;
    };

    if let Some(slot) = gamepad
        .slots
        .iter()
        .find(|binding| any_gamepad_just_pressed(gamepads, [binding.button]))
        .map(|binding| binding.slot)
    {
        return Some(SwitchCameraRequest::ToSlot(slot));
    }

    if any_gamepad_just_pressed(gamepads, gamepad.next.iter().copied()) {
        return Some(SwitchCameraRequest::CycleNext);
    }

    if any_gamepad_just_pressed(gamepads, gamepad.prev.iter().copied()) {
        return Some(SwitchCameraRequest::CyclePrev);
    }

    None
}

fn any_gamepad_just_pressed(
    gamepads: &Query<&Gamepad>,
    buttons: impl IntoIterator<Item = GamepadButton>,
) -> bool {
    let buttons = buttons.into_iter().collect::<Vec<_>>();
    !buttons.is_empty()
        && gamepads
            .iter()
            .any(|gamepad| gamepad.any_just_pressed(buttons.iter().copied()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CameraSwitchPlugin, SwitchableCamera};
    use bevy::camera::Camera;

    fn test_app(bindings: CameraInputBindings) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins((CameraSwitchPlugin, CameraInputBindingsPlugin))
            .insert_resource(bindings);
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

    fn active_camera(app: &mut App) -> Option<Entity> {
        let world = app.world_mut();
        let mut query = world.query::<(Entity, &Camera)>();
        query
            .iter(world)
            .find_map(|(entity, camera)| camera.is_active.then_some(entity))
    }

    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
    }

    fn press_gamepad_button(app: &mut App, entity: Entity, button: GamepadButton) {
        let mut gamepad = app.world_mut().get_mut::<Gamepad>(entity).unwrap();
        gamepad.digital_mut().press(button);
    }

    #[test]
    fn keyboard_slot_binding_emits_to_slot_request() {
        let mut app = test_app(CameraInputBindings::default().bind_slot(KeyCode::Digit2, 2));
        spawn_switchable_camera(&mut app, Some(1), 10, true);
        let second = spawn_switchable_camera(&mut app, Some(2), 20, false);

        press_key(&mut app, KeyCode::Digit2);
        app.update();

        assert_eq!(active_camera(&mut app), Some(second));
    }

    #[test]
    fn keyboard_next_binding_cycles_forward() {
        let mut app = test_app(CameraInputBindings::default().bind_next(KeyCode::KeyE));
        spawn_switchable_camera(&mut app, Some(1), 10, true);
        let second = spawn_switchable_camera(&mut app, Some(2), 20, false);

        press_key(&mut app, KeyCode::KeyE);
        app.update();

        assert_eq!(active_camera(&mut app), Some(second));
    }

    #[test]
    fn keyboard_prev_binding_cycles_backward() {
        let mut app = test_app(CameraInputBindings::default().bind_prev(KeyCode::KeyQ));
        let first = spawn_switchable_camera(&mut app, Some(1), 10, false);
        spawn_switchable_camera(&mut app, Some(2), 20, true);

        press_key(&mut app, KeyCode::KeyQ);
        app.update();

        assert_eq!(active_camera(&mut app), Some(first));
    }

    #[test]
    fn gamepad_binding_cycles_forward() {
        let bindings = CameraInputBindings::default()
            .with_gamepad(CameraGamepadBindings::default().bind_next(GamepadButton::RightTrigger));
        let mut app = test_app(bindings);
        spawn_switchable_camera(&mut app, Some(1), 10, true);
        let second = spawn_switchable_camera(&mut app, Some(2), 20, false);
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();

        press_gamepad_button(&mut app, gamepad, GamepadButton::RightTrigger);
        app.update();

        assert_eq!(active_camera(&mut app), Some(second));
    }
}
