mod definition;
mod definition_registry;
mod error;
mod loader;
mod locale;
mod localization;
mod text_key;
mod validation;

pub use definition::{LocaleSource, LocalizationDefinition};
pub use definition_registry::register_definition;
pub use error::LocalizationLoadError;
pub use locale::Locale;
pub use localization::{Localization, LocalizationPlugin};
pub use text_key::TextKey;

pub fn locale_name_key_id(locale_id: &str) -> String {
    format!(
        "common.locale_name.{}",
        locale_id.replace('-', "_").to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE_KEYS: &[TextKey] = &[
        TextKey::new("common.greeting"),
        TextKey::new("common.nested.label"),
    ];
    const BASE_SOURCES: &[LocaleSource] = &[
        LocaleSource {
            locale: "en-US",
            namespace: "common",
            contents: r#"
greeting = "Hello {name}"
[locale_name]
en_us = "English"
zh_cn = "Chinese"
[nested]
label = "Nested"
"#,
        },
        LocaleSource {
            locale: "zh-CN",
            namespace: "common",
            contents: r#"
greeting = "你好 {name}"
[locale_name]
en_us = "英语"
zh_cn = "中文"
[nested]
label = "嵌套"
"#,
        },
    ];
    static BASE_DEFINITION: LocalizationDefinition = LocalizationDefinition {
        fallback_locale: "en-US",
        locales: &["en-US", "zh-CN"],
        sources: BASE_SOURCES,
        keys: BASE_KEYS,
    };

    #[test]
    fn locale_files_are_flattened() {
        let localization =
            Localization::from_definition(&BASE_DEFINITION).expect("base definition should load");

        assert_eq!(
            localization.lookup_id(Locale::new("en-US"), "common.nested.label"),
            Some("Nested")
        );
    }

    #[test]
    fn missing_keys_fail_validation() {
        const KEYS: &[TextKey] = &[TextKey::new("common.greeting"), TextKey::new("common.bye")];
        static DEFINITION: LocalizationDefinition = LocalizationDefinition {
            fallback_locale: "en-US",
            locales: &["en-US"],
            sources: &[LocaleSource {
                locale: "en-US",
                namespace: "common",
                contents: r#"
greeting = "Hello"
[locale_name]
en_us = "English"
"#,
            }],
            keys: KEYS,
        };

        let err = Localization::from_definition(&DEFINITION).expect_err("missing key should fail");
        assert!(
            err.to_string()
                .contains("missing localization key 'common.bye'")
        );
    }

    #[test]
    fn placeholder_mismatch_fails_validation() {
        static DEFINITION: LocalizationDefinition = LocalizationDefinition {
            fallback_locale: "en-US",
            locales: &["en-US", "zh-CN"],
            sources: &[
                LocaleSource {
                    locale: "en-US",
                    namespace: "common",
                    contents: r#"
greeting = "Hello {name}"
[locale_name]
en_us = "English"
zh_cn = "Chinese"
[nested]
label = "Nested"
"#,
                },
                LocaleSource {
                    locale: "zh-CN",
                    namespace: "common",
                    contents: r#"
greeting = "你好 {user}"
[locale_name]
en_us = "英语"
zh_cn = "中文"
[nested]
label = "嵌套"
"#,
                },
            ],
            keys: BASE_KEYS,
        };

        let err = Localization::from_definition(&DEFINITION)
            .expect_err("placeholder mismatch should fail");
        assert!(err.to_string().contains("placeholder mismatch"));
    }

    #[test]
    fn text_lookup_falls_back_to_fallback_locale() {
        let mut localization =
            Localization::from_definition(&BASE_DEFINITION).expect("base definition should load");

        localization.set_locale(Locale::new("fr-FR"));

        assert_eq!(
            localization.text(TextKey::new("common.greeting")),
            "Hello {name}"
        );
        assert_eq!(
            localization.format_text(TextKey::new("common.greeting"), [("name", "Alex")]),
            "Hello Alex"
        );
    }
}
