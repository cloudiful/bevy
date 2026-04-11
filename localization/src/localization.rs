use crate::loader::load_tables;
use crate::validation::validate_tables;
use crate::{
    Locale, LocalizationDefinition, LocalizationLoadError, TextKey, definition_registry,
    locale_name_key_id,
};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Debug, Clone)]
pub struct Localization {
    definition: &'static LocalizationDefinition,
    locale: Locale,
    fallback_locale: Locale,
    tables: HashMap<&'static str, HashMap<String, String>>,
}

impl Default for Localization {
    fn default() -> Self {
        Self::load().expect("failed to load localization TOML files")
    }
}

impl Localization {
    pub fn load() -> Result<Self, LocalizationLoadError> {
        Self::from_definition(definition_registry::active_definition())
    }

    pub fn from_definition(
        definition: &'static LocalizationDefinition,
    ) -> Result<Self, LocalizationLoadError> {
        let fallback_locale = definition.fallback_locale();
        let tables = load_tables(definition)?;
        validate_tables(definition, &tables)?;

        Ok(Self {
            definition,
            locale: fallback_locale,
            fallback_locale,
            tables,
        })
    }

    pub fn text(&self, key: TextKey) -> &str {
        self.lookup(self.locale, key)
            .or_else(|| self.lookup(self.fallback_locale, key))
            .unwrap_or_else(|| panic!("missing localization key: {}", key.id()))
    }

    pub fn format_text<'a, I>(&self, key: TextKey, values: I) -> String
    where
        I: IntoIterator<Item = (&'a str, &'a str)>,
    {
        let mut text = self.text(key).to_string();
        for (name, value) in values {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }

    pub fn current_locale(&self) -> Locale {
        self.locale
    }

    pub fn locale(&self) -> Locale {
        self.current_locale()
    }

    pub fn available_locales(&self) -> Vec<Locale> {
        self.definition.locales()
    }

    pub fn set_locale(&mut self, locale: Locale) {
        self.locale = locale;
    }

    pub fn locale_display_text(&self, locale: Locale) -> &str {
        let key_id = locale_name_key_id(locale.id());
        self.lookup_id(self.locale, &key_id)
            .or_else(|| self.lookup_id(self.fallback_locale, &key_id))
            .unwrap_or(locale.id())
    }

    pub fn lookup(&self, locale: Locale, key: TextKey) -> Option<&str> {
        self.tables
            .get(locale.id())
            .and_then(|table| table.get(key.id()))
            .map(String::as_str)
    }

    pub fn lookup_id(&self, locale: Locale, key_id: &str) -> Option<&str> {
        self.tables
            .get(locale.id())
            .and_then(|table| table.get(key_id))
            .map(String::as_str)
    }

    pub fn table(&self, locale: Locale) -> &HashMap<String, String> {
        self.tables
            .get(locale.id())
            .unwrap_or_else(|| panic!("missing locale table for {}", locale.id()))
    }
}

pub struct LocalizationPlugin {
    definition: &'static LocalizationDefinition,
}

impl LocalizationPlugin {
    pub const fn new(definition: &'static LocalizationDefinition) -> Self {
        Self { definition }
    }
}

impl Plugin for LocalizationPlugin {
    fn build(&self, app: &mut App) {
        definition_registry::register_definition(self.definition);
        app.insert_resource(
            Localization::from_definition(self.definition)
                .expect("failed to load localization TOML files"),
        );
    }
}
