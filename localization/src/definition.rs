use crate::{Locale, TextKey};

#[derive(Debug, Clone, Copy)]
pub struct LocaleSource {
    pub locale: &'static str,
    pub namespace: &'static str,
    pub contents: &'static str,
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
