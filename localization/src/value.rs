use serde::de::{
    Deserialize, Deserializer, Error as DeError, IgnoredAny, MapAccess, SeqAccess, Visitor,
};
use std::collections::HashSet;
use std::fmt;

/// Format-neutral representation of a parsed locale payload.
///
/// Every supported serialization format is lowered into this shape so the
/// flattening logic stays identical across TOML, JSON, and YAML.
#[derive(Debug)]
pub(crate) enum LocaleValue {
    Text(String),
    Table(Vec<(LocaleKey, LocaleValue)>),
    /// A scalar, array, or tagged value that is not a string or table.
    Other,
}

#[derive(Debug)]
pub(crate) enum LocaleKey {
    Text(String),
    /// A non-string mapping key, only reachable through YAML payloads.
    Other,
}

impl LocaleValue {
    pub(crate) fn from_toml(value: toml::Value) -> Self {
        match value {
            toml::Value::String(text) => Self::Text(text),
            toml::Value::Table(table) => Self::Table(
                table
                    .into_iter()
                    .map(|(key, value)| (LocaleKey::Text(key), Self::from_toml(value)))
                    .collect(),
            ),
            _ => Self::Other,
        }
    }
}

/// Deserializes JSON and YAML payloads into the format-neutral tree.
///
/// Mapping keys are visited instead of collected into a `HashMap`, so duplicate
/// keys inside a single object are surfaced as a parse error for the source
/// format rather than silently overwriting the earlier value.
impl<'de> Deserialize<'de> for LocaleValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(LocaleValueVisitor)
    }
}

struct LocaleValueVisitor;

impl<'de> Visitor<'de> for LocaleValueVisitor {
    type Value = LocaleValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string or a mapping of string keys")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Text(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Text(value))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut entries = Vec::new();
        let mut seen = HashSet::new();

        while let Some((key, value)) = map.next_entry::<LocaleKey, LocaleValue>()? {
            if let LocaleKey::Text(text) = &key
                && !seen.insert(text.clone())
            {
                return Err(A::Error::custom(format!(
                    "duplicate localization key: {text}"
                )));
            }

            entries.push((key, value));
        }

        Ok(LocaleValue::Table(entries))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(LocaleValue::Other)
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_i128<E>(self, _value: i128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_u128<E>(self, _value: u128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_f32<E>(self, _value: f32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleValue::Other)
    }
}

impl<'de> Deserialize<'de> for LocaleKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(LocaleKeyVisitor)
    }
}

struct LocaleKeyVisitor;

impl<'de> Visitor<'de> for LocaleKeyVisitor {
    type Value = LocaleKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a string mapping key")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Text(value.to_owned()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Text(value))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(LocaleKey::Other)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_entry::<IgnoredAny, IgnoredAny>()?.is_some() {}
        Ok(LocaleKey::Other)
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_i64<E>(self, _value: i64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_u64<E>(self, _value: u64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_i128<E>(self, _value: i128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_u128<E>(self, _value: u128) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_f32<E>(self, _value: f32) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Ok(LocaleKey::Other)
    }
}
