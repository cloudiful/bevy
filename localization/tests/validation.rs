use cloudiful_bevy_localization::{
    LocaleSource, LocaleSourceFormat, Localization, LocalizationDefinition, TextKey,
};

#[test]
fn locale_source_constructors_and_format_names() {
    let source = LocaleSource::new("en-US", "common", LocaleSourceFormat::Yaml, "hello: Hi");

    assert_eq!(source.locale, "en-US");
    assert_eq!(source.namespace, "common");
    assert_eq!(source.format, LocaleSourceFormat::Yaml);
    assert_eq!(source.contents, "hello: Hi");

    assert_eq!(LocaleSourceFormat::Toml.as_str(), "TOML");
    assert_eq!(LocaleSourceFormat::Json.as_str(), "JSON");
    assert_eq!(LocaleSourceFormat::Yaml.as_str(), "YAML");
}

static DUPLICATE_ACROSS_SOURCES: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[
        LocaleSource::json("en-US", "common", r#"{ "hello": "Hello" }"#),
        LocaleSource::json("en-US", "common", r#"{ "hello": "Hi" }"#),
    ],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn duplicate_keys_across_sources_are_rejected() {
    let error = Localization::from_definition(&DUPLICATE_ACROSS_SOURCES)
        .expect_err("duplicate flattened key should be rejected");

    assert_eq!(error.source_format(), None);
    assert!(
        error
            .to_string()
            .contains("duplicate localization key: common.hello"),
        "unexpected error: {error}"
    );
}

static DUPLICATE_TOML_KEY: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::toml(
        "en-US",
        "common",
        "hello = \"Hello\"\nhello = \"Hi\"\n",
    )],
    keys: &[TextKey::new("common.hello")],
};

static DUPLICATE_JSON_KEY: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::json(
        "en-US",
        "common",
        r#"{ "hello": "Hello", "hello": "Hi" }"#,
    )],
    keys: &[TextKey::new("common.hello")],
};

static DUPLICATE_YAML_KEY: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::yaml(
        "en-US",
        "common",
        "hello: \"Hello\"\nhello: \"Hi\"\n",
    )],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn duplicate_keys_inside_one_source_report_a_format_parse_error() {
    let toml_error =
        Localization::from_definition(&DUPLICATE_TOML_KEY).expect_err("duplicate TOML key");
    assert_eq!(toml_error.source_format(), Some(LocaleSourceFormat::Toml));

    let json_error =
        Localization::from_definition(&DUPLICATE_JSON_KEY).expect_err("duplicate JSON key");
    assert_eq!(json_error.source_format(), Some(LocaleSourceFormat::Json));
    assert!(
        json_error
            .to_string()
            .contains("duplicate localization key"),
        "unexpected error: {json_error}"
    );

    let yaml_error =
        Localization::from_definition(&DUPLICATE_YAML_KEY).expect_err("duplicate YAML key");
    assert_eq!(yaml_error.source_format(), Some(LocaleSourceFormat::Yaml));
    assert!(
        yaml_error
            .to_string()
            .contains("duplicate localization key"),
        "unexpected error: {yaml_error}"
    );
}

static YAML_NON_STRING_KEY: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::yaml("en-US", "common", "1: value\n")],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn yaml_non_string_mapping_keys_are_rejected() {
    let error = Localization::from_definition(&YAML_NON_STRING_KEY)
        .expect_err("non-string YAML key should be rejected");

    assert_eq!(error.source_format(), None);
    assert!(
        error.to_string().contains("must be strings"),
        "unexpected error: {error}"
    );
}

static YAML_BOOL_VALUE: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::yaml("en-US", "common", "hello: true\n")],
    keys: &[TextKey::new("common.hello")],
};

static YAML_NULL_VALUE: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::yaml("en-US", "common", "hello: null\n")],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn yaml_bool_and_null_values_are_rejected_as_non_string_values() {
    for error in [
        Localization::from_definition(&YAML_BOOL_VALUE).expect_err("YAML bool should be rejected"),
        Localization::from_definition(&YAML_NULL_VALUE).expect_err("YAML null should be rejected"),
    ] {
        assert_eq!(error.source_format(), None);
        assert!(
            error.to_string().contains("must be a string or table"),
            "unexpected error: {error}"
        );
    }
}

static YAML_TAGGED_VALUE: LocalizationDefinition = LocalizationDefinition {
    fallback_locale: "en-US",
    locales: &["en-US"],
    sources: &[LocaleSource::yaml(
        "en-US",
        "common",
        "hello: !custom value\n",
    )],
    keys: &[TextKey::new("common.hello")],
};

#[test]
fn yaml_tagged_values_report_a_yaml_parse_error() {
    let error = Localization::from_definition(&YAML_TAGGED_VALUE)
        .expect_err("tagged YAML value should be rejected");

    assert_eq!(error.source_format(), Some(LocaleSourceFormat::Yaml));
    assert!(
        error.to_string().contains("localization YAML"),
        "unexpected error: {error}"
    );
}
