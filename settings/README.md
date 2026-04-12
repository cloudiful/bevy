# cloudiful-bevy-settings

Reusable Bevy settings framework for apps that keep their own action enums,
field keys, localization layer, and UI messages.

## What it provides

- `SettingActionHandler<Action>`: app settings resource contract for validating
  and applying a narrowed settings action
- `RequestedSettingAction<Action>`: adapter trait for app-defined UI/request
  messages
- `SettingFieldSpec<AppAction, FieldKey, TextKey>`: generic field descriptor
  with label metadata and a control definition
- `SettingControlSpec<Action, FieldKey>`: toggle, stepper, and select control
  variants
- `SettingSliderSpec<Action>`: slider metadata for stepper-style controls
- `SettingSelectOption<Action>`: select option descriptor with disabled-state
  support
- `SettingFieldSource<AppAction, Context, FieldKey, TextKey>`: trait for
  producing field specs from app context
- `SettingSystemSet`: shared `ApplyActions` and `SyncUi` schedule sets
- `apply_setting_action(...)`: immediate helper for applying one app action to
  persistent settings
- `change_setting(...)`: message-driven bridge from UI/request messages into
  persistent settings updates
- `register_setting_systems(...)`: helper that orders change systems before UI
  sync systems

## What it does not provide

- app-specific settings resources
- app-specific action enums, field keys, or text-key enums
- widget rendering, button/slider/select UI components, or layout
- persistence bootstrapping beyond using `bevy-persistent`
- localized label assembly helpers or any localization runtime

This crate stays narrow on purpose: your app owns settings data, UI messages,
field identities, and localization. This crate only coordinates schema/action
flow and the system ordering around persistent settings updates.

## Minimal end-to-end example

```rust,no_run
use bevy::prelude::*;
use bevy_persistent::{Persistent, StorageFormat};
use cloudiful_bevy_settings::{
    RequestedSettingAction, SettingActionHandler, change_setting,
    register_setting_systems,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
enum AppAction {
    ToggleFullscreen,
    SetScale(f32),
    Ignore,
}

#[derive(Debug, Clone, Copy)]
enum SettingsAction {
    ToggleFullscreen,
    SetScale(f32),
}

impl TryFrom<AppAction> for SettingsAction {
    type Error = ();

    fn try_from(value: AppAction) -> Result<Self, Self::Error> {
        match value {
            AppAction::ToggleFullscreen => Ok(Self::ToggleFullscreen),
            AppAction::SetScale(scale) => Ok(Self::SetScale(scale)),
            AppAction::Ignore => Err(()),
        }
    }
}

#[derive(Resource, Debug, Default, Serialize, Deserialize)]
struct AppSettings {
    fullscreen: bool,
    ui_scale: f32,
}

impl SettingActionHandler<SettingsAction> for AppSettings {
    fn can_apply(&self, action: SettingsAction) -> bool {
        match action {
            SettingsAction::ToggleFullscreen => true,
            SettingsAction::SetScale(scale) => (0.5..=2.0).contains(&scale),
        }
    }

    fn apply(&mut self, action: SettingsAction) {
        match action {
            SettingsAction::ToggleFullscreen => {
                self.fullscreen = !self.fullscreen;
            }
            SettingsAction::SetScale(scale) => {
                self.ui_scale = scale;
            }
        }
    }
}

#[derive(Message, Debug, Clone, Copy)]
struct UiSettingRequest {
    action: AppAction,
}

impl RequestedSettingAction<AppAction> for UiSettingRequest {
    fn action(&self) -> AppAction {
        self.action
    }
}

fn sync_ui() {}

fn build_settings_resource() -> Persistent<AppSettings> {
    let path = std::env::temp_dir().join("cloudiful-bevy-settings-example.toml");

    Persistent::<AppSettings>::builder()
        .name("app settings")
        .format(StorageFormat::Toml)
        .path(&path)
        .default(AppSettings {
            fullscreen: false,
            ui_scale: 1.0,
        })
        .build()
        .unwrap()
}

fn main() {
    let mut app = App::new();
    app.add_message::<UiSettingRequest>();
    app.insert_resource(build_settings_resource());

    register_setting_systems(
        &mut app,
        change_setting::<AppSettings, SettingsAction, UiSettingRequest, AppAction>,
        sync_ui,
    );
}
```

## Schema Types

`SettingFieldSpec` is the generic description your UI layer can consume. Each
field has:

- `label`: already-resolved display text
- `label_key`: optional app-defined text key for localization-aware UIs
- `control`: one `SettingControlSpec` variant

`SettingControlSpec` models the common settings control shapes:

- `Toggle { text, action }`
- `Stepper { key, value, decrease_action, increase_action, slider }`
- `Select { key, value, options }`

Helpers on `SettingFieldSpec` keep construction compact:

- `toggle(...)`
- `stepper(...)`
- `select(...)`
- `with_label_key(...)`
- `with_slider(...)`

Use `SettingSliderSpec::new(...)` when a stepper also needs slider metadata such
as range, precision, suffix text, and the `fn(f32) -> Action` mapper.

## System Behavior

`register_setting_systems(...)` configures two ordered update sets:

1. `SettingSystemSet::ApplyActions`
2. `SettingSystemSet::SyncUi`

Put message-reading or direct action-application systems in `ApplyActions`.
Put systems that rebuild labels, refresh view state, or mirror settings back
into UI resources in `SyncUi`.

`change_setting(...)` reads Bevy messages, extracts the app action via
`RequestedSettingAction`, narrows it with `TryFrom`, checks `can_apply`, and
then updates the `Persistent<T>` resource.

`apply_setting_action(...)` is the same logic without message plumbing. Use it
when a system already has the action value in hand.
