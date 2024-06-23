use std::{collections::HashMap, sync::OnceLock};

use crate::asset;

static TRANSLATIONS: OnceLock<HashMap<String, String>> = OnceLock::new();

pub fn load_translations() -> asset::Result<()> {
    // TODO use correct locale
    let translations = asset::load_yaml_file("lang", "en.yaml")?;
    TRANSLATIONS.get_or_init(|| translations);
    Ok(())
}

pub fn tr(key: &str) -> &str {
    let translations = TRANSLATIONS.get().expect("translations not loaded");
    if let Some(value) = translations.get(key) {
        value
    } else {
        eprintln!("Missing translation for {}", key);
        key
    }
}
