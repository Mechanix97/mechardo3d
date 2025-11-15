use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// Load translations from modular JSON files organized by language
/// Returns a HashMap with language codes as keys and merged translation JSON as values
pub fn load_translations() -> HashMap<String, Value> {
    let mut translations = HashMap::new();

    // Load English translations
    if let Ok(en_merged) = load_language_translations("en") {
        translations.insert("en".to_string(), en_merged);
    } else {
        eprintln!("Failed to load English translations");
    }

    // Load Spanish translations
    if let Ok(es_merged) = load_language_translations("es") {
        translations.insert("es".to_string(), es_merged);
    } else {
        eprintln!("Failed to load Spanish translations");
    }

    translations
}

/// Load and merge all translation files for a specific language
fn load_language_translations(lang: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut merged = serde_json::Map::new();

    // Files to load in order
    let files = vec!["common", "home", "contact", "blog", "ds2000", "about"];

    for file in files {
        let path = format!("translations/{}/{}.json", lang, file);
        match fs::read_to_string(&path) {
            Ok(content) => {
                match serde_json::from_str::<Value>(&content) {
                    Ok(Value::Object(obj)) => {
                        // Merge this file's content into the main object
                        for (key, value) in obj {
                            merged.insert(key, value);
                        }
                    }
                    Ok(_) => eprintln!("Translation file {} is not a JSON object", path),
                    Err(e) => eprintln!("Failed to parse {}: {}", path, e),
                }
            }
            Err(e) => eprintln!("Failed to read {}: {}", path, e),
        }
    }

    Ok(Value::Object(merged))
}

/// Get translations for a specific language
/// Returns the translation object that can be passed to Tera context as "t"
pub fn get_translations_for_lang(translations: &HashMap<String, Value>, lang: &str) -> Value {
    translations.get(lang).cloned().unwrap_or_else(|| {
        // Fallback to Spanish if language not found
        translations
            .get("es")
            .cloned()
            .unwrap_or(Value::Object(serde_json::Map::new()))
    })
}
