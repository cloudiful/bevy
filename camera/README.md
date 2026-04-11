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

## Stable ordering

Candidates always sort by:

1. `order_key`
2. `slot.unwrap_or(u8::MAX)`
3. entity index

That same ordering drives cycle behavior and repeated-slot tie breaking.

## Usage

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
