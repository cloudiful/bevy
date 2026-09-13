use crate::value::{LocaleKey, LocaleValue};
use crate::{LocaleSource, LocaleSourceFormat, LocalizationDefinition, LocalizationLoadError};
use std::collections::HashMap;

pub(crate) fn load_tables(
    definition: &'static LocalizationDefinition,
) -> Result<HashMap<&'static str, HashMap<String, String>>, LocalizationLoadError> {
    let mut tables = definition
        .locales()
        .into_iter()
        .map(|locale| (locale.id(), HashMap::new()))
        .collect::<HashMap<_, _>>();

    for source in definition.sources {
        let value = parse_source(source)?;
        let table = tables.get_mut(source.locale).ok_or_else(|| {
            LocalizationLoadError::InvalidData(format!(
                "locale source '{}' is not listed in available locales",
                source.locale
            ))
        })?;
        flatten_locale_value(source.namespace, &value, table)?;
    }

    Ok(tables)
}

fn parse_source(source: &LocaleSource) -> Result<LocaleValue, LocalizationLoadError> {
    Ok(match source.format {
        LocaleSourceFormat::Toml => {
            LocaleValue::from_toml(toml::from_str::<toml::Value>(source.contents)?)
        }
        LocaleSourceFormat::Json => serde_json::from_str::<LocaleValue>(source.contents)?,
        LocaleSourceFormat::Yaml => yaml_serde::from_str::<LocaleValue>(source.contents)?,
    })
}

fn flatten_locale_value(
    prefix: &str,
    value: &LocaleValue,
    texts: &mut HashMap<String, String>,
) -> Result<(), LocalizationLoadError> {
    match value {
        LocaleValue::Text(text) => {
            let previous = texts.insert(prefix.to_string(), text.clone());
            if previous.is_some() {
                return Err(LocalizationLoadError::InvalidData(format!(
                    "duplicate localization key: {prefix}"
                )));
            }
        }
        LocaleValue::Table(entries) => {
            for (key, nested_value) in entries {
                let LocaleKey::Text(key) = key else {
                    return Err(LocalizationLoadError::InvalidData(format!(
                        "localization keys at '{prefix}' must be strings"
                    )));
                };
                let nested_prefix = format!("{prefix}.{key}");
                flatten_locale_value(&nested_prefix, nested_value, texts)?;
            }
        }
        LocaleValue::Other => {
            return Err(LocalizationLoadError::InvalidData(format!(
                "localization value at '{prefix}' must be a string or table"
            )));
        }
    }

    Ok(())
}
