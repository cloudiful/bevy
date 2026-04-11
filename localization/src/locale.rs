use crate::definition_registry::active_definition;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Locale(&'static str);

impl Default for Locale {
    fn default() -> Self {
        active_definition().fallback_locale()
    }
}

impl Locale {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn id(self) -> &'static str {
        self.0
    }

    pub fn available() -> Vec<Self> {
        active_definition().locales()
    }

    pub fn from_serialized(raw: &str) -> Option<Self> {
        let normalized = canonical_locale_id(raw);

        Self::available()
            .into_iter()
            .find(|locale| canonical_locale_id(locale.id()) == normalized)
    }
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Locale::from_serialized(&raw)
            .ok_or_else(|| D::Error::custom(format!("unknown locale: {raw}")))
    }
}

fn canonical_locale_id(raw: &str) -> String {
    raw.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_lowercase())
        .collect()
}
