use cloudiful_bevy_localization::{
    Locale, LocaleSource, LocaleSourceFormat, Localization, LocalizationDefinition, TextKey,
};

const MIXED_KEYS: &[TextKey] = &[
    TextKey::new("common.hello"),
    TextKey::new("common.nested.label"),
];

static MIXED_DEFINITION: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US", "zh-CN", "ja-JP"],
    sources: &[
        LocaleSource::toml(
            "en-US",
            "common",
            r#"
hello = "Hello {name}"
[locale_name]
en_us = "English"
zh_cn = "Chinese"
ja_jp = "Japanese"
[nested]
label = "Nested"
"#,
        ),
        LocaleSource::yaml(
            "zh-CN",
            "common",
            r#"
hello: "你好 {name}"
locale_name:
  en_us: "英语"
  zh_cn: "中文"
  ja_jp: "日语"
nested:
  label: "嵌套"
"#,
        ),
        LocaleSource::json(
            "ja-JP",
            "common",
            r#"{
  "hello": "こんにちは {name}",
  "locale_name": { "en_us": "英語", "zh_cn": "中国語", "ja_jp": "日本語" },
  "nested": { "label": "ネスト" }
}"#,
        ),
    ],
    keys: MIXED_KEYS,
};

#[test]
fn all_formats_flatten_into_the_same_tables() {
    let localization =
        Localization::from_definition(&MIXED_DEFINITION).expect("mixed formats should load");

    assert_eq!(
        localization.lookup_id(Locale::new("en-US"), "common.hello"),
        Some("Hello {name}")
    );
    assert_eq!(
        localization.lookup_id(Locale::new("zh-CN"), "common.hello"),
        Some("你好 {name}")
    );
    assert_eq!(
        localization.lookup_id(Locale::new("ja-JP"), "common.hello"),
        Some("こんにちは {name}")
    );
    assert_eq!(
        localization.lookup_id(Locale::new("en-US"), "common.nested.label"),
        Some("Nested")
    );
    assert_eq!(
        localization.lookup_id(Locale::new("zh-CN"), "common.nested.label"),
        Some("嵌套")
    );
    assert_eq!(
        localization.lookup_id(Locale::new("ja-JP"), "common.nested.label"),
        Some("ネスト")
    );
    assert_eq!(
        localization.lookup_id(Locale::new("ja-JP"), "common.locale_name.ja_jp"),
        Some("日本語")
    );
}

static BAD_TOML: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Toml,
        contents: "hello = \"unterminated",
    }],
    keys: &[TextKey::new("common.hello")],
};

static BAD_JSON: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Json,
        contents: r#"{ "hello": }"#,
    }],
    keys: &[TextKey::new("common.hello")],
};

static BAD_YAML: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Yaml,
        contents: "hello: [unterminated",
    }],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn parse_errors_identify_the_source_format() {
    let toml_error = Localization::from_definition(&BAD_TOML).expect_err("bad TOML should fail");
    assert_eq!(toml_error.source_format(), Some(LocaleSourceFormat::Toml));
    assert!(toml_error.to_string().contains("TOML"));

    let json_error = Localization::from_definition(&BAD_JSON).expect_err("bad JSON should fail");
    assert_eq!(json_error.source_format(), Some(LocaleSourceFormat::Json));
    assert!(json_error.to_string().contains("JSON"));

    let yaml_error = Localization::from_definition(&BAD_YAML).expect_err("bad YAML should fail");
    assert_eq!(yaml_error.source_format(), Some(LocaleSourceFormat::Yaml));
    assert!(yaml_error.to_string().contains("YAML"));
}

static SCALAR_TOML: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Toml,
        contents: "hello = 42",
    }],
    keys: &[TextKey::new("common.hello")],
};

static ARRAY_JSON: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Json,
        contents: r#"{ "hello": ["a", "b"] }"#,
    }],
    keys: &[TextKey::new("common.hello")],
};

static MAPPING_YAML: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Yaml,
        contents: "hello: 42",
    }],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn non_string_values_remain_rejected_for_every_format() {
    for err in [
        Localization::from_definition(&SCALAR_TOML).expect_err("TOML integer should be rejected"),
        Localization::from_definition(&ARRAY_JSON).expect_err("JSON array should be rejected"),
        Localization::from_definition(&MAPPING_YAML).expect_err("YAML scalar should be rejected"),
    ] {
        assert!(
            err.to_string().contains("must be a string or table"),
            "unexpected error: {err}"
        );
    }
}

static UNKNOWN_KEY_JSON: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource {
        locale: "en-US",
        namespace: "common",
        format: LocaleSourceFormat::Json,
        contents: r#"{
  "hello": "Hello",
  "locale_name": { "en_us": "English" },
  "extra": "unexpected"
}"#,
    }],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn unknown_keys_are_rejected_for_json_sources() {
    let err = Localization::from_definition(&UNKNOWN_KEY_JSON)
        .expect_err("unknown JSON key should be rejected");
    assert!(err.to_string().contains("unknown localization keys"));
    assert!(err.to_string().contains("common.extra"));
}
