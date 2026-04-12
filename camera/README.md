# cloudiful-bevy-camera

Reusable Bevy camera switching core.

## What it provides

- `CameraSwitchPlugin`: registers request handling and switch events
- `SwitchableCamera`: marker/config component for cameras managed by core
- `SwitchCameraRequest`: external switch request API
- `CameraSwitched`: emitted when active switchable camera changes

## What it does not provide

- camera spawning or setup helpers
- keyboard or gamepad bindings
- settings, rebind, or device-selection logic
- UI focus gating or project-specific business rules

This crate stays narrow on purpose: project code marks cameras with
`SwitchableCamera` and sends `SwitchCameraRequest` values.

An optional `input_bindings` feature adds a minimal Bevy-native input layer.
It stays generic on purpose:

- keyboard via `ButtonInput<KeyCode>`
- optional native gamepad bindings via Bevy `Gamepad`
- no control-settings resource
- no rebind workflow
- no primary-device selection
- no UI/business-state gating

## Participation Rules

Only entities that have both:

- `SwitchableCamera`
- `Camera`

participate in switching. A request targeting a plain `Camera`, a plain
`SwitchableCamera`, or any unrelated entity is ignored.

## Stable ordering

Candidates always sort by:

1. `order_key`
2. `slot.unwrap_or(u8::MAX)`
3. entity index

That same ordering drives cycle behavior and repeated-slot tie breaking.

## Request Semantics

`SwitchCameraRequest` supports four operations:

- `ToEntity(Entity)`: switch to the matching switchable camera entity
- `ToSlot(u8)`: switch to the first ordered camera with that slot
- `CycleNext`: move forward through the ordered camera list
- `CyclePrev`: move backward through the ordered camera list

Behavior details:

- missing targets are a no-op
- cycling with no active camera selects the first ordered candidate
- switching to the already uniquely active camera is a no-op
- if multiple cameras are active, the next valid switch collapses them down to
  one active camera
- `CameraSwitched.previous` is `Some(entity)` only when there was exactly one
  active camera before the switch
- when a switch request succeeds, the target camera becomes the only active
  switchable camera

## Usage

### Core only

```rust
use bevy::prelude::*;
use cloudiful_bevy_camera::{
    CameraSwitchPlugin, SwitchCameraRequest, SwitchableCamera,
};

App::new().add_plugins((DefaultPlugins, CameraSwitchPlugin));

fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        Camera::default(),
        SwitchableCamera {
            slot: Some(1),
            order_key: 10,
        },
    ));
}

fn request_camera(mut requests: MessageWriter<SwitchCameraRequest>) {
    requests.write(SwitchCameraRequest::ToSlot(1));
}
```

Listen for successful switches:

```rust
use bevy::prelude::*;
use cloudiful_bevy_camera::CameraSwitched;

fn observe_switches(mut switched: MessageReader<CameraSwitched>) {
    for event in switched.read() {
        println!("camera changed: {:?} -> {:?}", event.previous, event.current);
    }
}
```

### `input_bindings` feature

```rust
# #[cfg(feature = "input_bindings")]
# {
use bevy::prelude::*;
use cloudiful_bevy_camera::{
    CameraGamepadBindings, CameraInputBindings, CameraInputBindingsPlugin,
    CameraSwitchPlugin, SwitchableCamera,
};

App::new()
    .add_plugins((
        DefaultPlugins,
        CameraSwitchPlugin,
        CameraInputBindingsPlugin,
    ))
    .insert_resource(
        CameraInputBindings::default()
            .bind_slot(KeyCode::Digit1, 1)
            .bind_slot(KeyCode::Digit2, 2)
            .bind_next(KeyCode::KeyE)
            .bind_prev(KeyCode::KeyQ)
            .with_gamepad(
                CameraGamepadBindings::default()
                    .bind_slot(GamepadButton::South, 1)
                    .bind_slot(GamepadButton::East, 2)
                    .bind_next(GamepadButton::RightTrigger)
                    .bind_prev(GamepadButton::LeftTrigger),
            ),
    );

fn spawn_camera(commands: &mut Commands) {
    commands.spawn((
        Camera::default(),
        SwitchableCamera {
            slot: Some(1),
            order_key: 10,
        },
    ));
}
# }
```

Feature-disabled builds still compile only the core switching API.

`CameraInputBindingsPlugin` emits `SwitchCameraRequest` messages from generic
keyboard/gamepad bindings before the core switch application runs.

Input-layer details:

- keyboard slot bindings use the first matching `just_pressed` key
- `next` and `prev` are checked after direct slot bindings
- gamepad checks run only if `CameraInputBindings.gamepad` is configured
- gamepad bindings match any connected gamepad; this feature does not pick a
  primary device
