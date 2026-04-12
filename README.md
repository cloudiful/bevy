# cloudiful-bevy

Reusable Bevy crates collected in a single workspace.

This repository is not a single umbrella runtime crate. Each workspace member is
published and consumed independently, while sharing a common Bevy baseline:
`0.18.1`.

## Crates

| Crate | Purpose | When to use it |
| --- | --- | --- |
| [`cloudiful-bevy-camera`](./camera/README.md) | Camera switching core, with optional generic input bindings | You need stable camera slot/entity switching without coupling it to game-specific state |
| [`cloudiful-bevy-input`](./input/README.md) | Generic action-to-input mapping and action state runtime | You want your game to own the action enum while sharing keyboard/gamepad handling |
| [`cloudiful-bevy-localization`](./localization/README.md) | Localization runtime for app-provided static registries | You generate locale/key definitions in your own app and need runtime lookup + validation |
| [`cloudiful-bevy-outline`](./outline/README.md) | Geometry-shell outline helpers for 3D meshes | You want a small reusable backface-shell outline setup instead of a full render pass |
| [`cloudiful-bevy-settings`](./cloudiful-bevy-settings/README.md) | Generic settings action/schema helpers wired for Bevy systems | You keep app-specific settings types and UI, but want reusable settings flow infrastructure |

## Workspace Usage

Run the full test suite:

```sh
cargo test --workspace
```

Build docs for one crate:

```sh
cargo doc -p cloudiful-bevy-input --no-deps
```

Run the outline example:

```sh
cargo run -p cloudiful-bevy-outline --example basic
```

Entry points:

- workspace manifest: [`Cargo.toml`](./Cargo.toml)
- camera docs: [`camera/README.md`](./camera/README.md)
- input docs: [`input/README.md`](./input/README.md)
- localization docs: [`localization/README.md`](./localization/README.md)
- outline docs: [`outline/README.md`](./outline/README.md)
- settings docs: [`cloudiful-bevy-settings/README.md`](./cloudiful-bevy-settings/README.md)
