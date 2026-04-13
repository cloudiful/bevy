# cloudiful-bevy-outline

Small reusable Bevy outline helper crate for geometry-shell outlines.

## What it provides

- `GpmoOutlinePlugin`: initializes default outline assets
- `GpmoOutlinePlugin::with_default_style(...)`: installs a custom default
  outline style
- `OutlineStyle`: configurable color, emissive strength, and shell scale
- `OutlineAssets`: default cached material/style resource
- `OutlineShell`: marker component for spawned outline meshes
- `create_outline_material(...)`: build a custom outline material from an
  `OutlineStyle`
- `outline_shell_transform(...)`: build the transform used for shell scaling
- `spawn_outline_mesh(...)`: attach an outline mesh with a custom material/style
- `spawn_default_outline_mesh(...)`: attach an outline mesh using the plugin's default style

## What it does not provide

- screen-space outlines
- stencil-based outline passes
- automatic mesh duplication for arbitrary scenes

This crate intentionally stays simple: it supports the common "draw a slightly enlarged backface-only shell" outline technique.

## Usage

Add the plugin:

```rust
use bevy::prelude::*;
use cloudiful_bevy_outline::{GpmoOutlinePlugin, OutlineAssets};

let mut app = App::new();
app.init_resource::<Assets<StandardMaterial>>()
    .add_plugins(GpmoOutlinePlugin::default());
app.update();

assert!(app.world().contains_resource::<OutlineAssets>());
```

Spawn an outline child with the default material/style resource:

```rust
use bevy::prelude::*;
use cloudiful_bevy_outline::{OutlineAssets, spawn_default_outline_mesh};

fn attach_outline(
    parent: &mut ChildSpawnerCommands,
    outline_assets: &OutlineAssets,
    mesh: Handle<Mesh>,
) {
    spawn_default_outline_mesh(parent, outline_assets, mesh, Visibility::Hidden);
}
```

Then toggle the outline child's `Visibility`.

The full runnable example lives at [`examples/basic.rs`](./examples/basic.rs).

## Default Style Flow

`GpmoOutlinePlugin` creates one default material at startup and stores it in
`OutlineAssets`.

- use `OutlineAssets` when many outline shells can share one default look
- use `GpmoOutlinePlugin::with_default_style(...)` when the whole app should
  start with a different default style
- use `create_outline_material(...)` when one outline needs its own material
- use `outline_shell_transform(...)` when you need the shell scale without
  spawning through the helper

Custom style example:

```rust
use bevy::prelude::*;
use cloudiful_bevy_outline::{GpmoOutlinePlugin, OutlineAssets, OutlineStyle};

let mut app = App::new();
app.init_resource::<Assets<StandardMaterial>>()
    .add_plugins(GpmoOutlinePlugin::with_default_style(OutlineStyle {
        color: Color::srgb(1.0, 0.6, 0.2),
        emissive_strength: 3.0,
        scale: Vec3::splat(1.12),
    }));
app.update();

let outline_assets = app.world().resource::<OutlineAssets>();
assert_eq!(outline_assets.default_style().emissive_strength, 3.0);
```

## Technique

This crate uses a geometry-shell outline:

1. draw the normal mesh
2. draw a second mesh with front-face culling
3. scale that second mesh slightly larger
4. only the outside silhouette remains visible
