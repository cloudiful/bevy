use crate::LocaleSourceFormat;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum LocalizationLoadError {
    ParseToml(toml::de::Error),
    ParseJson(serde_json::Error),
    ParseYaml(yaml_serde::Error),
    InvalidData(String),
}

impl LocalizationLoadError {
    /// Returns the source format when this error came from parsing a payload.
    ///
    /// Validation and flattening failures return `None` because they are not
    /// tied to a single format.
    pub const fn source_format(&self) -> Option<LocaleSourceFormat> {
        match self {
            Self::ParseToml(_) => Some(LocaleSourceFormat::Toml),
            Self::ParseJson(_) => Some(LocaleSourceFormat::Json),
            Self::ParseYaml(_) => Some(LocaleSourceFormat::Yaml),
            Self::InvalidData(_) => None,
        }
    }
}

impl Display for LocalizationLoadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseToml(err) => write!(f, "failed to parse localization TOML: {err}"),
            Self::ParseJson(err) => write!(f, "failed to parse localization JSON: {err}"),
            Self::ParseYaml(err) => write!(f, "failed to parse localization YAML: {err}"),
            Self::InvalidData(message) => write!(f, "{message}"),
        }
    }
}

impl Error for LocalizationLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ParseToml(err) => Some(err),
            Self::ParseJson(err) => Some(err),
            Self::ParseYaml(err) => Some(err),
            Self::InvalidData(_) => None,
        }
    }
}

impl From<toml::de::Error> for LocalizationLoadError {
    fn from(value: toml::de::Error) -> Self {
        Self::ParseToml(value)
    }
}

impl From<serde_json::Error> for LocalizationLoadError {
    fn from(value: serde_json::Error) -> Self {
        Self::ParseJson(value)
    }
}

impl From<yaml_serde::Error> for LocalizationLoadError {
    fn from(value: yaml_serde::Error) -> Self {
        Self::ParseYaml(value)
    }
}
