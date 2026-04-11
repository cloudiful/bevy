use crate::action_state::{ActionData, ActionState, InputSettings};
use crate::bindings::{InputAction, InputBinding, InputButton, InputMap};
use crate::device::{ActiveInputDevice, InputDevice, PrimaryGamepad, PrimaryGamepadMode};
use bevy::input::ButtonInput;
use bevy::input::gamepad::Gamepad;
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;

pub(crate) fn sync_primary_gamepad(
    mut primary: ResMut<PrimaryGamepad>,
    gamepads: Query<Entity, With<Gamepad>>,
) {
    let mut connected = gamepads.iter().collect::<Vec<_>>();
    connected.sort_unstable_by_key(|entity| entity.index());

    if let Some(selected) = primary.selected() {
        if connected.contains(&selected) {
            return;
        }
    }

    match primary.mode() {
        PrimaryGamepadMode::Auto => primary.set_selected(connected.into_iter().next()),
        PrimaryGamepadMode::Manual => primary.set_selected(None),
    }
}

pub(crate) fn update_action_state<A: InputAction>(
    map: Res<InputMap<A>>,
    settings: Res<InputSettings>,
    keyboard: Res<ButtonInput<KeyCode>>,
    primary: Res<PrimaryGamepad>,
    gamepads: Query<&Gamepad>,
    mut active_device: ResMut<ActiveInputDevice>,
    mut state: ResMut<ActionState<A>>,
) {
    let primary_gamepad = primary
        .selected()
        .and_then(|entity| gamepads.get(entity).ok());
    let mut keyboard_active = false;
    let mut gamepad_active = false;

    for action in map.actions() {
        let previous = state.data(action);
        let (value, action_keyboard_active, action_gamepad_active) = action_value(
            map.bindings(action),
            &keyboard,
            primary_gamepad,
            settings.axis_activity_threshold,
        );
        let pressed = value.abs() >= settings.action_press_threshold;
        let next = ActionData {
            pressed,
            just_pressed: !previous.pressed && pressed,
            just_released: previous.pressed && !pressed,
            value,
        };

        keyboard_active |= action_keyboard_active;
        gamepad_active |= action_gamepad_active;
        state.set(action, next);
    }

    if keyboard_active {
        active_device.set_current(Some(InputDevice::KeyboardMouse));
    } else if gamepad_active {
        active_device.set_current(primary.selected().map(InputDevice::Gamepad));
    }
}

fn action_value(
    bindings: &[InputBinding],
    keyboard: &ButtonInput<KeyCode>,
    gamepad: Option<&Gamepad>,
    axis_activity_threshold: f32,
) -> (f32, bool, bool) {
    let mut value = 0.0;
    let mut keyboard_active = false;
    let mut gamepad_active = false;

    for binding in bindings {
        let (binding_value, binding_keyboard_active, binding_gamepad_active) =
            binding_value(*binding, keyboard, gamepad, axis_activity_threshold);
        value = merge_value(value, binding_value);
        keyboard_active |= binding_keyboard_active;
        gamepad_active |= binding_gamepad_active;
    }

    (value, keyboard_active, gamepad_active)
}

fn binding_value(
    binding: InputBinding,
    keyboard: &ButtonInput<KeyCode>,
    gamepad: Option<&Gamepad>,
    axis_activity_threshold: f32,
) -> (f32, bool, bool) {
    match binding {
        InputBinding::Button(button) => {
            let pressed = button_pressed(button, keyboard, gamepad);
            let value = if pressed { 1.0 } else { 0.0 };
            let device_flags = button_flags(button, pressed);
            (value, device_flags.0, device_flags.1)
        }
        InputBinding::GamepadAxis(axis) => {
            let value = gamepad.and_then(|pad| pad.get(axis)).unwrap_or(0.0);
            (value, false, value.abs() >= axis_activity_threshold)
        }
        InputBinding::ButtonAxis { negative, positive } => {
            let negative_pressed = button_pressed(negative, keyboard, gamepad);
            let positive_pressed = button_pressed(positive, keyboard, gamepad);
            let value = match (negative_pressed, positive_pressed) {
                (true, false) => -1.0,
                (false, true) => 1.0,
                _ => 0.0,
            };
            let negative_flags = button_flags(negative, negative_pressed);
            let positive_flags = button_flags(positive, positive_pressed);
            (
                value,
                negative_flags.0 || positive_flags.0,
                negative_flags.1 || positive_flags.1,
            )
        }
    }
}

fn button_pressed(
    button: InputButton,
    keyboard: &ButtonInput<KeyCode>,
    gamepad: Option<&Gamepad>,
) -> bool {
    match button {
        InputButton::Key(key) => keyboard.pressed(key),
        InputButton::Gamepad(button) => gamepad.is_some_and(|pad| pad.pressed(button)),
    }
}

fn button_flags(button: InputButton, pressed: bool) -> (bool, bool) {
    match button {
        InputButton::Key(_) => (pressed, false),
        InputButton::Gamepad(_) => (false, pressed),
    }
}

fn merge_value(current: f32, next: f32) -> f32 {
    if next.abs() > current.abs() {
        next
    } else {
        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InputBindingsPlugin;
    use bevy::input::gamepad::{GamepadAxis, GamepadButton};

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum TestAction {
        Jump,
        MoveX,
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(InputBindingsPlugin::<TestAction>::default());
        app
    }

    #[test]
    fn keyboard_binding_updates_action_state() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<InputMap<TestAction>>()
            .bind_key(TestAction::Jump, KeyCode::Space);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Space);
        app.update();

        let state = app.world().resource::<ActionState<TestAction>>();
        assert!(state.pressed(TestAction::Jump));
        assert!(state.just_pressed(TestAction::Jump));
        assert_eq!(
            app.world().resource::<ActiveInputDevice>().current(),
            Some(InputDevice::KeyboardMouse)
        );
    }

    #[test]
    fn rebind_changes_trigger_key() {
        let mut app = test_app();
        {
            let mut map = app.world_mut().resource_mut::<InputMap<TestAction>>();
            map.bind_key(TestAction::Jump, KeyCode::Space);
            assert!(map.rebind_button(
                TestAction::Jump,
                InputButton::Key(KeyCode::Space),
                InputButton::Key(KeyCode::Enter),
            ));
        }

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Enter);
        app.update();

        assert!(
            app.world()
                .resource::<ActionState<TestAction>>()
                .pressed(TestAction::Jump)
        );
    }

    #[test]
    fn primary_gamepad_defaults_to_first_connected() {
        let mut app = test_app();
        let second = app.world_mut().spawn(Gamepad::default()).id();
        let first = app.world_mut().spawn(Gamepad::default()).id();

        app.update();

        let selected = app.world().resource::<PrimaryGamepad>().selected();
        let expected = if first.index() < second.index() {
            first
        } else {
            second
        };
        assert_eq!(selected, Some(expected));
    }

    #[test]
    fn gamepad_axis_updates_value_and_active_device() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<InputMap<TestAction>>()
            .bind_gamepad_axis(TestAction::MoveX, GamepadAxis::LeftStickX);
        let gamepad = app.world_mut().spawn(Gamepad::default()).id();

        app.update();
        app.world_mut()
            .resource_mut::<PrimaryGamepad>()
            .select(gamepad);
        app.world_mut()
            .get_mut::<Gamepad>(gamepad)
            .unwrap()
            .analog_mut()
            .set(GamepadAxis::LeftStickX, 0.75);
        app.update();

        let state = app.world().resource::<ActionState<TestAction>>();
        assert_eq!(state.value(TestAction::MoveX), 0.75);
        assert!(state.pressed(TestAction::MoveX));
        assert_eq!(
            app.world().resource::<ActiveInputDevice>().current(),
            Some(InputDevice::Gamepad(gamepad))
        );
    }

    #[test]
    fn manual_primary_gamepad_is_respected() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<InputMap<TestAction>>()
            .bind_gamepad_button(TestAction::Jump, GamepadButton::South);
        let first = app.world_mut().spawn(Gamepad::default()).id();
        let second = app.world_mut().spawn(Gamepad::default()).id();

        app.world_mut()
            .resource_mut::<PrimaryGamepad>()
            .select(second);
        app.update();

        app.world_mut()
            .get_mut::<Gamepad>(first)
            .unwrap()
            .digital_mut()
            .press(GamepadButton::South);
        app.update();
        assert!(
            !app.world()
                .resource::<ActionState<TestAction>>()
                .pressed(TestAction::Jump)
        );

        app.world_mut()
            .get_mut::<Gamepad>(second)
            .unwrap()
            .digital_mut()
            .press(GamepadButton::South);
        app.update();
        assert!(
            app.world()
                .resource::<ActionState<TestAction>>()
                .pressed(TestAction::Jump)
        );
    }
}
