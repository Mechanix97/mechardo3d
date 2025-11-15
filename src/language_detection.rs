use crate::language::Language;
use axum::http::HeaderMap;

/// Detect language from cookie first, then fallback to Accept-Language header
pub fn detect_language(headers: &HeaderMap) -> Language {
    // First, try to get language from cookie
    #[allow(clippy::collapsible_if)]
    if let Some(cookie_header) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie in cookie_str.split(';') {
                let cookie = cookie.trim();
                if let Some(lang) = cookie.strip_prefix("language=") {
                    if let Some(language) = Language::from_str(lang) {
                        return language;
                    }
                }
            }
        }
    }

    // Fallback to Accept-Language header
    detect_from_accept_language(headers)
}

/// Parse Accept-Language header and return the best matching language
/// Supports formats like: "en-US,en;q=0.9,es;q=0.8"
pub fn detect_from_accept_language(headers: &HeaderMap) -> Language {
    let accept_language = headers
        .get("accept-language")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if accept_language.is_empty() {
        return Language::default();
    }

    // Parse Accept-Language header
    // Format: "en-US,en;q=0.9,es;q=0.8" or "es-ES,es;q=0.9,en;q=0.8"
    let mut languages: Vec<(String, f32)> = Vec::new();

    for part in accept_language.split(',') {
        let part = part.trim();
        if let Some((lang, qvalue)) = part.split_once(';') {
            let lang = lang.trim().to_lowercase();
            // Extract q value (quality/priority)
            let q = qvalue
                .trim()
                .strip_prefix("q=")
                .and_then(|q| q.parse::<f32>().ok())
                .unwrap_or(1.0);

            // Extract base language (e.g., "en-US" -> "en")
            let base_lang = lang.split('-').next().unwrap_or(&lang).to_string();
            languages.push((base_lang, q));
        } else {
            // No q value, default to 1.0
            let lang = part.trim().to_lowercase();
            let base_lang = lang.split('-').next().unwrap_or(&lang).to_string();
            languages.push((base_lang, 1.0));
        }
    }

    // Sort by quality (q value) descending
    languages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Find the first supported language
    for (lang, _) in languages {
        if let Some(language) = Language::from_str(&lang) {
            return language;
        }
    }

    // Default to Spanish if no match found
    Language::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_detect_english() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            HeaderValue::from_static("en-US,en;q=0.9"),
        );
        assert_eq!(detect_from_accept_language(&headers), Language::English);
    }

    #[test]
    fn test_detect_spanish() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            HeaderValue::from_static("es-ES,es;q=0.9"),
        );
        assert_eq!(detect_from_accept_language(&headers), Language::Spanish);
    }

    #[test]
    fn test_detect_with_quality() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept-language",
            HeaderValue::from_static("fr-FR,fr;q=0.9,es;q=0.8,en;q=0.7"),
        );
        // Should return Spanish (first supported language)
        assert_eq!(detect_from_accept_language(&headers), Language::Spanish);
    }

    #[test]
    fn test_default_on_empty() {
        let headers = HeaderMap::new();
        assert_eq!(detect_from_accept_language(&headers), Language::default());
    }
}
