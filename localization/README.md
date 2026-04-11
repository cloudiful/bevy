# cloudiful-bevy-localization

Reusable Bevy localization runtime for apps that generate their own static locale registry at build time.

Scope:
- locale/text key types
- TOML table loading and flattening
- key completeness and placeholder validation
- runtime lookup, formatting, and locale switching
- Bevy `Plugin` and `Resource` wiring

Non-goals:
- scanning downstream `assets/i18n`
- generating app-specific key constants
- embedding downstream `OUT_DIR` artifacts inside the crate

## Usage

Generate a static registry in the downstream app and pass it into the plugin:

```rust
use cloudiful_bevy_localization::{
    LocaleSource, LocalizationDefinition, LocalizationPlugin, TextKey,
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

app.add_plugins(LocalizationPlugin::new(&LOCALIZATION));
```

At runtime, use the resource:

```rust
fn ui_text(localization: Res<cloudiful_bevy_localization::Localization>) {
    let text = localization.text(TextKey::new("common.hello"));
    println!("{text}");
}
```
