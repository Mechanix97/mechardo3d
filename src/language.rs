use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[serde(rename = "es")]
    Spanish,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
        }
    }

    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Spanish => "Español",
        }
    }

    #[allow(dead_code)]
    pub fn flag_emoji(&self) -> &'static str {
        match self {
            Language::English => "🇬🇧",
            Language::Spanish => "🇪🇸",
        }
    }

    /// Returns all supported languages in order
    #[allow(dead_code)]
    pub fn all() -> &'static [Language] {
        &[Language::Spanish, Language::English]
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            _ => None,
        }
    }

    pub fn default() -> Self {
        Language::Spanish
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::default()
    }
}


