use crate::{LocalizationDefinition, LocalizationLoadError};
use std::collections::HashMap;
use toml::Value;

pub(crate) fn load_tables(
    definition: &'static LocalizationDefinition,
) -> Result<HashMap<&'static str, HashMap<String, String>>, LocalizationLoadError> {
    let mut tables = definition
        .locales()
        .into_iter()
        .map(|locale| (locale.id(), HashMap::new()))
        .collect::<HashMap<_, _>>();

    for source in definition.sources {
        let value = toml::from_str::<Value>(source.contents)?;
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

fn flatten_locale_value(
    prefix: &str,
    value: &Value,
    texts: &mut HashMap<String, String>,
) -> Result<(), LocalizationLoadError> {
    match value {
        Value::String(text) => {
            let previous = texts.insert(prefix.to_string(), text.clone());
            if previous.is_some() {
                return Err(LocalizationLoadError::InvalidData(format!(
                    "duplicate localization key: {prefix}"
                )));
            }
        }
        Value::Table(table) => {
            for (key, nested_value) in table {
                let nested_prefix = format!("{prefix}.{key}");
                flatten_locale_value(&nested_prefix, nested_value, texts)?;
            }
        }
        _ => {
            return Err(LocalizationLoadError::InvalidData(format!(
                "localization value at '{prefix}' must be a string or table"
            )));
        }
    }

    Ok(())
}
