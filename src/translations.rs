use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// Load translations from JSON files
/// Returns a HashMap with language codes as keys and translation JSON as values
pub fn load_translations() -> HashMap<String, Value> {
    let mut translations = HashMap::new();

    // Load English translations
    if let Ok(en_content) = fs::read_to_string("translations/en.json") {
        if let Ok(en_json) = serde_json::from_str::<Value>(&en_content) {
            translations.insert("en".to_string(), en_json);
        } else {
            eprintln!("Failed to parse translations/en.json");
        }
    } else {
        eprintln!("Failed to read translations/en.json");
    }

    // Load Spanish translations
    if let Ok(es_content) = fs::read_to_string("translations/es.json") {
        if let Ok(es_json) = serde_json::from_str::<Value>(&es_content) {
            translations.insert("es".to_string(), es_json);
        } else {
            eprintln!("Failed to parse translations/es.json");
        }
    } else {
        eprintln!("Failed to read translations/es.json");
    }

    translations
}

/// Get translations for a specific language
/// Returns the translation object that can be passed to Tera context as "t"
pub fn get_translations_for_lang(translations: &HashMap<String, Value>, lang: &str) -> Value {
    translations
        .get(lang)
        .cloned()
        .unwrap_or_else(|| {
            // Fallback to Spanish if language not found
            translations
                .get("es")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()))
        })
}
