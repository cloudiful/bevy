# cloudiful-bevy-localization

Reusable Bevy localization runtime for apps that generate their own static
locale registry at build time.

## What it provides

- `LocalizationDefinition`: static description of fallback locale, supported
  locales, locale sources, and declared keys
- `LocaleSource`: one TOML payload bound to a locale + namespace
- `Locale`: runtime locale handle with serialization support
- `TextKey`: runtime text key handle with serialization support
- `Localization`: Bevy resource for lookup, formatting, locale switching, and
  table access
- `LocalizationPlugin`: registers a definition and inserts the `Localization`
  resource
- `LocalizationLoadError`: structured load/validation failure
- `register_definition(...)`: definition registry hook used by the plugin and
  helper types
- `locale_name_key_id(...)`: helper for `common.locale_name.<locale>` key
  generation

## What it does not provide

- scanning downstream `assets/i18n`
- generating app-specific key constants
- embedding downstream `OUT_DIR` artifacts inside the crate
- editor tooling, extraction, or translation workflows

This crate expects the downstream app to generate or hand-author a static
registry and then pass that definition into the plugin.

## Usage

Generate a static registry in the downstream app and pass it into the plugin.
`LocalizationPlugin::new(...)` registers the definition first and then inserts
the `Localization` resource built from that definition.

```rust
use bevy::prelude::*;
use cloudiful_bevy_localization::{
    Locale, LocaleSource, Localization, LocalizationDefinition,
    LocalizationPlugin, TextKey,
};

const KEYS: &[TextKey] = &[TextKey::new("common.hello")];
const SOURCES: &[LocaleSource] = &[
    LocaleSource {
        locale: "en-US",
        namespace: "common",
        contents: r#"hello = "Hello""#,
    },
    LocaleSource {
        locale: "zh-CN",
        namespace: "common",
        contents: r#"hello = "你好""#,
    },
];

static LOCALIZATION: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US", "zh-CN"],
    sources: SOURCES,
    keys: KEYS,
};

fn main() {
    App::new()
        .add_plugins(LocalizationPlugin::new(&LOCALIZATION))
        .add_systems(Update, read_text);
}

fn read_text(localization: Res<Localization>) {
    assert_eq!(localization.current_locale(), Locale::new("en-US"));
    assert_eq!(localization.text(TextKey::new("common.hello")), "Hello");
}
```

At runtime, use the resource for lookup, formatting, locale switching, and
locale display text:

```rust
use bevy::prelude::*;
use cloudiful_bevy_localization::{Locale, Localization, TextKey};

fn ui_text(mut localization: ResMut<Localization>) {
    let text = localization.text(TextKey::new("common.hello"));
    let formatted = localization.format_text(TextKey::new("common.hello"), []);

    localization.set_locale(Locale::new("zh-CN"));

    let current = localization.current_locale();
    let available = localization.available_locales();
    let display = localization.locale_display_text(current);

    println!("{text} / {formatted} / {display} / {available:?}");
}
```

## Validation Rules

Definitions are validated at load time. A load fails when any of these rules are
broken:

- the fallback locale table is missing
- a listed locale has no table
- a declared `TextKey` is missing from any locale
- a `common.locale_name.<locale>` entry is missing for any supported locale
- a locale table contains keys that are not declared
- placeholder names differ from the fallback locale for the same key
- a locale source references a locale not listed in `LocalizationDefinition`
- a TOML value is not a string or nested table
- the same flattened key appears more than once

`LocaleSource.contents` TOML is flattened by namespace, so:

```toml
[nested]
label = "Nested"
```

under namespace `common` becomes `common.nested.label`.

## Runtime Behavior

- `Localization::text(...)` looks up the current locale first, then the fallback
  locale, and panics only if both are missing
- `Localization::format_text(...)` performs simple `{name}` replacement on the
  selected text
- `Localization::lookup(...)` and `lookup_id(...)` return `Option<&str>` without
  panicking
- `Localization::table(...)` returns the flattened table for one locale
- `Localization::locale_display_text(...)` looks up
  `common.locale_name.<locale>` in the current locale, then fallback locale, and
  finally returns the raw locale id

## Locale and Key Helpers

- `Locale::new(...)` and `TextKey::new(...)` create static handles
- `Locale::available()` reads available locales from the active registered
  definition
- `Locale::from_serialized(...)` normalizes case and punctuation when matching
  serialized locale ids
- `TextKey::from_id(...)` resolves an id against the active registered
  definition
