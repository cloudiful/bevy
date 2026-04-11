use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LocalizationLoadError {
    ParseToml(toml::de::Error),
    InvalidData(String),
}

impl Display for LocalizationLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseToml(err) => write!(f, "failed to parse localization TOML: {err}"),
            Self::InvalidData(message) => write!(f, "{message}"),
        }
    }
}

impl Error for LocalizationLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParseToml(err) => Some(err),
            Self::InvalidData(_) => None,
        }
    }
}

impl From<toml::de::Error> for LocalizationLoadError {
    fn from(value: toml::de::Error) -> Self {
        Self::ParseToml(value)
    }
}
