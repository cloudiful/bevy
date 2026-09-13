use crate::{Locale, TextKey};

/// Serialization format of a [`LocaleSource`] payload.
///
/// Format selection is explicit; the loader never guesses from payload text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocaleSourceFormat {
    Toml,
    Json,
    Yaml,
}

impl LocaleSourceFormat {
    /// Human-readable name used in error messages.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Toml => "TOML",
            Self::Json => "JSON",
            Self::Yaml => "YAML",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LocaleSource {
    pub locale: &'static str,
    pub namespace: &'static str,
    pub format: LocaleSourceFormat,
    pub contents: &'static str,
}

impl LocaleSource {
    /// Creates a source with an explicit [`LocaleSourceFormat`].
    pub const fn new(
        locale: &'static str,
        namespace: &'static str,
        format: LocaleSourceFormat,
        contents: &'static str,
    ) -> Self {
        Self {
            locale,
            namespace,
            format,
            contents,
        }
    }

    /// Creates a TOML source.
    pub const fn toml(
        locale: &'static str,
        namespace: &'static str,
        contents: &'static str,
    ) -> Self {
        Self::new(locale, namespace, LocaleSourceFormat::Toml, contents)
    }

    /// Creates a JSON source.
    pub const fn json(
        locale: &'static str,
        namespace: &'static str,
        contents: &'static str,
    ) -> Self {
        Self::new(locale, namespace, LocaleSourceFormat::Json, contents)
    }

    /// Creates a YAML source.
    pub const fn yaml(
        locale: &'static str,
        namespace: &'static str,
        contents: &'static str,
    ) -> Self {
        Self::new(locale, namespace, LocaleSourceFormat::Yaml, contents)
    }
}

#[derive(Debug)]
pub struct LocalizationDefinition {
    pub fallback_locale: &'static str,
    pub locales: &'static [&'static str],
    pub sources: &'static [LocaleSource],
    pub keys: &'static [TextKey],
}

impl LocalizationDefinition {
    pub fn fallback_locale(&self) -> Locale {
        Locale::new(self.fallback_locale)
    }

    pub fn locales(&self) -> Vec<Locale> {
        self.locales.iter().copied().map(Locale::new).collect()
    }
}
