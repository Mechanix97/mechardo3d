use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value};
use tracing::{info, warn};

use crate::language::Language;

/// All translations, indexed by language code.
///
/// Each language directory (`translations/{lang}/*.json`) is merged into a
/// single JSON object that is handed to Tera as the `t` context variable.
/// Adding a new module file is enough for it to be picked up - there is no
/// hard-coded list of modules.
#[derive(Debug)]
pub struct Translations {
    by_lang: HashMap<String, Value>,
    empty: Value,
}

impl Default for Translations {
    fn default() -> Self {
        Self {
            by_lang: HashMap::new(),
            empty: Value::Object(Map::new()),
        }
    }
}

impl Translations {
    pub fn load(dir: &Path) -> Self {
        let mut by_lang = HashMap::new();

        for language in Language::ALL {
            let lang_dir = dir.join(language.as_str());
            match load_language(&lang_dir) {
                Ok(merged) => {
                    by_lang.insert(language.as_str().to_string(), merged);
                }
                Err(e) => warn!("Failed to load translations from {:?}: {}", lang_dir, e),
            }
        }

        info!("Loaded translations for languages: {:?}", by_lang.keys());

        Self {
            by_lang,
            ..Self::default()
        }
    }

    /// Translation tree for a language, falling back to the default language.
    pub fn for_lang(&self, lang: Language) -> &Value {
        self.by_lang
            .get(lang.as_str())
            .or_else(|| self.by_lang.get(Language::default().as_str()))
            .unwrap_or(&self.empty)
    }

    /// Look up a dotted key such as `page_titles.home`.
    pub fn text(&self, lang: Language, path: &str) -> Option<&str> {
        let mut node = self.for_lang(lang);
        for key in path.split('.') {
            node = node.get(key)?;
        }
        node.as_str()
    }

    /// Look up a dotted key, falling back to `default` when it is missing.
    pub fn text_or<'a>(&'a self, lang: Language, path: &str, default: &'a str) -> &'a str {
        self.text(lang, path).unwrap_or(default)
    }
}

/// Read and merge every `*.json` file in a language directory.
fn load_language(dir: &Path) -> Result<Value, std::io::Error> {
    let mut files: Vec<_> = fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect();
    // Sorted so that merge order (and therefore collision reporting) is stable.
    files.sort();

    let mut merged = Map::new();
    for path in files {
        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(e) => {
                warn!("Failed to read {:?}: {}", path, e);
                continue;
            }
        };

        match serde_json::from_str::<Value>(&contents) {
            Ok(Value::Object(obj)) => {
                for (key, value) in obj {
                    if merged.insert(key.clone(), value).is_some() {
                        warn!(
                            "Duplicate translation key {:?} while loading {:?}",
                            key, path
                        );
                    }
                }
            }
            Ok(_) => warn!("Translation file {:?} is not a JSON object", path),
            Err(e) => warn!("Failed to parse {:?}: {}", path, e),
        }
    }

    Ok(Value::Object(merged))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn translations() -> Translations {
        Translations::load(Path::new("translations"))
    }

    fn key_paths(value: &Value, prefix: &str, out: &mut BTreeSet<String>) {
        if let Value::Object(map) = value {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", prefix, key)
                };
                out.insert(path.clone());
                key_paths(child, &path, out);
            }
        }
    }

    #[test]
    fn loads_every_language() {
        let translations = translations();
        for language in Language::ALL {
            let tree = translations.for_lang(language);
            assert!(
                tree.as_object().is_some_and(|module| !module.is_empty()),
                "missing translations for {}",
                language.as_str()
            );
        }
    }

    #[test]
    fn languages_have_the_same_keys() {
        let translations = translations();
        let mut english = BTreeSet::new();
        let mut spanish = BTreeSet::new();
        key_paths(translations.for_lang(Language::English), "", &mut english);
        key_paths(translations.for_lang(Language::Spanish), "", &mut spanish);

        let only_english: Vec<_> = english.difference(&spanish).collect();
        let only_spanish: Vec<_> = spanish.difference(&english).collect();
        assert!(
            only_english.is_empty() && only_spanish.is_empty(),
            "translation keys out of sync: only in en {:?}, only in es {:?}",
            only_english,
            only_spanish
        );
    }

    #[test]
    fn looks_up_dotted_keys() {
        let translations = translations();
        assert_eq!(
            translations.text(Language::English, "page_titles.home"),
            Some("Home")
        );
        assert_eq!(
            translations.text(Language::English, "page_titles.nope"),
            None
        );
        assert_eq!(
            translations.text_or(Language::English, "page_titles.nope", "Fallback"),
            "Fallback"
        );
    }

    #[test]
    fn empty_store_never_panics() {
        let empty = Translations::default();
        assert!(empty.for_lang(Language::English).is_object());
        assert_eq!(empty.text(Language::English, "page_titles.home"), None);
    }
}
