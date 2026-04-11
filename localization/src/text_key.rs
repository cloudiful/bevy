use crate::definition_registry::active_definition;
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as DeError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextKey(&'static str);

impl TextKey {
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    pub const fn id(self) -> &'static str {
        self.0
    }

    pub fn from_id(id: &str) -> Option<Self> {
        active_definition()
            .keys
            .iter()
            .copied()
            .find(|key| key.id() == id)
    }
}

impl Serialize for TextKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for TextKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;
        TextKey::from_id(&id).ok_or_else(|| D::Error::custom(format!("unknown text key: {id}")))
    }
}
