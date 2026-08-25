use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[serde(rename = "en")]
    English,
    #[default]
    #[serde(rename = "es")]
    Spanish,
}

impl Language {
    /// Every language the site is published in, in menu order.
    pub const ALL: [Language; 2] = [Language::Spanish, Language::English];

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Spanish => "es",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "en" => Some(Language::English),
            "es" => Some(Language::Spanish),
            _ => None,
        }
    }

    /// BCP 47 tag, e.g. for `hreflang` and JSON-LD `inLanguage`.
    pub fn locale(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Spanish => "es-ES",
        }
    }

    /// Open Graph locale, e.g. `og:locale`.
    pub fn og_locale(&self) -> &'static str {
        match self {
            Language::English => "en_US",
            Language::Spanish => "es_ES",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_language_codes() {
        for language in Language::ALL {
            assert_eq!(Language::from_str(language.as_str()), Some(language));
        }
    }

    #[test]
    fn rejects_unsupported_codes() {
        assert_eq!(Language::from_str("fr"), None);
        assert_eq!(Language::from_str("EN"), None);
        assert_eq!(Language::from_str(""), None);
    }

    #[test]
    fn defaults_to_spanish() {
        assert_eq!(Language::default(), Language::Spanish);
        assert_eq!(Language::default().as_str(), "es");
    }

    #[test]
    fn exposes_locales() {
        assert_eq!(Language::English.locale(), "en-US");
        assert_eq!(Language::Spanish.og_locale(), "es_ES");
    }
}
