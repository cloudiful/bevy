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

```rust
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
        info!("jump");
    }

    let movement = actions.value(Action::MoveX);
    if movement != 0.0 {
        info!("move_x = {movement}");
    }

    info!("active device: {:?}", active_device.current());
}
```

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
