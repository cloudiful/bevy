use crate::LocalizationDefinition;
use std::sync::RwLock;

static ACTIVE_DEFINITION: RwLock<Option<&'static LocalizationDefinition>> = RwLock::new(None);

pub fn register_definition(definition: &'static LocalizationDefinition) {
    *ACTIVE_DEFINITION
        .write()
        .expect("localization definition registry poisoned") = Some(definition);
}

pub(crate) fn active_definition() -> &'static LocalizationDefinition {
    ACTIVE_DEFINITION
        .read()
        .expect("localization definition registry poisoned")
        .as_ref()
        .copied()
        .expect("localization definition not registered")
}
