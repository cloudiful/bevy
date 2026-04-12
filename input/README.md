# cloudiful-bevy-input

Reusable Bevy input abstraction for game-defined actions.

## What it provides

- `InputBindingsPlugin<A>`: generic plugin for a game-defined action enum
- `InputMap<A>`: action -> keyboard/gamepad bindings
- `ActionState<A>`: `pressed`, `just_pressed`, `just_released`, and analog `value`
- `PrimaryGamepad`: auto or manual primary gamepad selection
- `ActiveInputDevice`: tracks whether keyboard/mouse or the primary gamepad is active
- rebind helpers on `InputMap<A>`

## What it does not provide

- camera, inventory, pad, menu, placement, or any other game logic
- a built-in action enum
- UI for controls menus
- save/load for user settings
- project-specific focus gating or business rules

This crate stays generic on purpose: the game defines actions, bindings, persistence, and meaning.

## Minimal demo

```rust,no_run
use bevy::input::gamepad::{GamepadAxis, GamepadButton};
use bevy::input::keyboard::KeyCode;
use bevy::prelude::*;
use cloudiful_bevy_input::{
    ActionState, ActiveInputDevice, InputBindingsPlugin, InputButton, InputMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Action {
    Jump,
    MoveX,
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, InputBindingsPlugin::<Action>::default()))
        .add_systems(Startup, setup_bindings)
        .add_systems(Update, read_input)
        .run();
}

fn setup_bindings(mut map: ResMut<InputMap<Action>>) {
    map.bind_key(Action::Jump, KeyCode::Space)
        .bind_gamepad_button(Action::Jump, GamepadButton::South)
        .bind_button_axis(
            Action::MoveX,
            InputButton::Key(KeyCode::KeyA),
            InputButton::Key(KeyCode::KeyD),
        )
        .bind_gamepad_axis(Action::MoveX, GamepadAxis::LeftStickX);
}

fn read_input(
    actions: Res<ActionState<Action>>,
    active_device: Res<ActiveInputDevice>,
) {
    if actions.just_pressed(Action::Jump) {
        println!("jump");
    }

    let movement = actions.value(Action::MoveX);
    if movement != 0.0 {
        println!("move_x = {movement}");
    }

    println!("active device: {:?}", active_device.current());
}
```

## Runtime Semantics

`ActionState<A>` stores four values per action:

- `pressed`
- `just_pressed`
- `just_released`
- `value`

`pressed` is threshold-based, not button-only. By default an action becomes
pressed when `value.abs() >= 0.5`.

`value` is the strongest absolute contribution from the action's bindings. If an
action is bound to multiple inputs, the runtime keeps whichever binding has the
largest absolute value for that frame.

`InputSettings` controls those thresholds:

- `action_press_threshold`: defaults to `0.5`
- `axis_activity_threshold`: defaults to `0.2`

The axis activity threshold only affects device-activity tracking. It does not
clamp the stored action value.

## Rebinding

```rust
use bevy::input::keyboard::KeyCode;
use cloudiful_bevy_input::{InputButton, InputMap};

# #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
# enum Action { Jump }
fn rebind_jump(map: &mut InputMap<Action>) {
    map.rebind_button(
        Action::Jump,
        InputButton::Key(KeyCode::Space),
        InputButton::Key(KeyCode::Enter),
    );
}
```

`rebind_button(...)` also updates `ButtonAxis` bindings when either side matches
the old button. `rebind_gamepad_axis(...)` swaps a bound analog axis in place.

## Primary Gamepad Selection

`PrimaryGamepad` tracks which connected gamepad should drive gamepad bindings.

- default mode is auto
- auto mode selects the connected gamepad with the lowest entity index
- `PrimaryGamepad::select(entity)` switches to manual mode
- `PrimaryGamepad::clear_manual()` returns to auto mode
- if the manually selected gamepad disappears, the selected gamepad becomes
  `None`

Manual selection example:

```rust
use bevy::prelude::Entity;
use cloudiful_bevy_input::PrimaryGamepad;

fn choose_primary(primary: &mut PrimaryGamepad, gamepad: Entity) {
    primary.select(gamepad);
}

fn reset_primary(primary: &mut PrimaryGamepad) {
    primary.clear_manual();
}
```

## Active Device Tracking

`ActiveInputDevice` records the last active source that crossed the configured
activity threshold:

- `InputDevice::KeyboardMouse`
- `InputDevice::Gamepad(Entity)`

Keyboard/mouse activity wins first in frames where both keyboard and gamepad
activity are detected. If no binding is active, the previous value is retained.
