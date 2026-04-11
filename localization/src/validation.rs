use crate::{Locale, LocalizationDefinition, LocalizationLoadError, locale_name_key_id};
use std::collections::{BTreeSet, HashMap};

pub(crate) fn validate_tables(
    definition: &'static LocalizationDefinition,
    tables: &HashMap<&'static str, HashMap<String, String>>,
) -> Result<(), LocalizationLoadError> {
    let locales = definition.locales();
    let fallback = definition.fallback_locale();

    if !tables.contains_key(fallback.id()) {
        return Err(LocalizationLoadError::InvalidData(format!(
            "missing fallback locale table for {}",
            fallback.id()
        )));
    }

    let expected_keys = expected_key_ids(definition, &locales);

    for locale in &locales {
        let Some(table) = tables.get(locale.id()) else {
            return Err(LocalizationLoadError::InvalidData(format!(
                "missing locale table for {}",
                locale.id()
            )));
        };

        for key in definition.keys {
            if !table.contains_key(key.id()) {
                return Err(LocalizationLoadError::InvalidData(format!(
                    "missing localization key '{}' for locale {:?}",
                    key.id(),
                    locale
                )));
            }
        }

        for display_locale in &locales {
            let locale_name_id = locale_name_key_id(display_locale.id());
            if !table.contains_key(&locale_name_id) {
                return Err(LocalizationLoadError::InvalidData(format!(
                    "missing locale name key '{}' for locale {:?}",
                    locale_name_id, locale
                )));
            }
        }

        let actual_keys = table.keys().cloned().collect::<BTreeSet<_>>();
        let unexpected = actual_keys
            .difference(&expected_keys)
            .cloned()
            .collect::<Vec<_>>();
        if !unexpected.is_empty() {
            return Err(LocalizationLoadError::InvalidData(format!(
                "locale {:?} contains unknown localization keys: {:?}",
                locale, unexpected
            )));
        }
    }

    let fallback_table = tables.get(fallback.id()).expect("validated above");
    for key in definition.keys {
        let fallback_placeholders =
            placeholders(fallback_table.get(key.id()).expect("validated above"));

        for locale in &locales {
            let table = tables.get(locale.id()).expect("validated above");
            let locale_placeholders = placeholders(table.get(key.id()).expect("validated above"));

            if locale_placeholders != fallback_placeholders {
                return Err(LocalizationLoadError::InvalidData(format!(
                    "placeholder mismatch for key '{}' in locale {:?}",
                    key.id(),
                    locale
                )));
            }
        }
    }

    Ok(())
}

fn expected_key_ids(
    definition: &'static LocalizationDefinition,
    locales: &[Locale],
) -> BTreeSet<String> {
    let mut expected = definition
        .keys
        .iter()
        .map(|key| key.id().to_string())
        .collect::<BTreeSet<_>>();

    for locale in locales {
        expected.insert(locale_name_key_id(locale.id()));
    }

    expected
}

pub(crate) fn placeholders(text: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            continue;
        }

        let mut name = String::new();
        while let Some(&next) = chars.peek() {
            chars.next();
            if next == '}' {
                break;
            }
            name.push(next);
        }

        if !name.is_empty() {
            names.insert(name);
        }
    }

    names
}
