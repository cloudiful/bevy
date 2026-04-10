# bevy_shell_outline

Small reusable Bevy outline helper crate for geometry-shell outlines.

## What it provides

- `GpmoOutlinePlugin`: initializes default outline assets
- `OutlineStyle`: configurable color, emissive strength, and shell scale
- `OutlineAssets`: default cached material/style resource
- `OutlineShell`: marker component for spawned outline meshes
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
use bevy_shell_outline::GpmoOutlinePlugin;

App::new().add_plugins((DefaultPlugins, GpmoOutlinePlugin::default()));
```

Spawn an outline child:

```rust
use bevy::prelude::*;
use bevy_shell_outline::{OutlineAssets, spawn_default_outline_mesh};

fn attach_outline(
    parent: &mut ChildSpawnerCommands,
    outline_assets: &OutlineAssets,
    mesh: Handle<Mesh>,
) {
    spawn_default_outline_mesh(parent, outline_assets, mesh, Visibility::Hidden);
}
```

Then toggle the outline child's `Visibility`.

## Technique

This crate uses a geometry-shell outline:

1. draw the normal mesh
2. draw a second mesh with front-face culling
3. scale that second mesh slightly larger
4. only the outside silhouette remains visible
