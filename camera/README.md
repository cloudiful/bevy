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

## Stable ordering

Candidates always sort by:

1. `order_key`
2. `slot.unwrap_or(u8::MAX)`
3. entity index

That same ordering drives cycle behavior and repeated-slot tie breaking.

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

### `input_bindings` feature

```rust
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
```

Feature-disabled builds still compile only the core switching API.
