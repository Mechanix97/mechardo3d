use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;

use crate::language::Language;
use crate::language_detection::detect_language;
use crate::responses::language_redirect;

/// The language taken from the first path segment.
///
/// Requests whose prefix is not a supported language are redirected to the
/// visitor's language instead of being silently rendered in Spanish under a URL
/// that does not exist (which produced duplicate content for crawlers).
pub struct Lang(pub Language);

impl<S> FromRequestParts<S> for Lang
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match Language::from_str(first_segment(parts.uri.path())) {
            Some(language) => Ok(Lang(language)),
            None => {
                let language = detect_language(&parts.headers);
                let target = localized_target(parts.uri.path(), parts.uri.query(), language);
                Err(language_redirect(&target))
            }
        }
    }
}

/// First path segment, without slashes (`/en/blog/4` -> `en`).
pub fn first_segment(path: &str) -> &str {
    path.trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or_default()
}

/// Where a request without a usable language prefix should go.
///
/// A prefix that looks like a language tag (`/fr/blog/4`) is swapped for a
/// supported one; anything else is treated as a path that simply lost its
/// prefix (`/ds2000` -> `/es/ds2000`).
pub fn localized_target(path: &str, query: Option<&str>, language: Language) -> String {
    let first = first_segment(path);
    let rest = if looks_like_language_tag(first) {
        path.trim_start_matches('/')
            .strip_prefix(first)
            .unwrap_or_default()
    } else {
        let trimmed = path.trim_start_matches('/');
        if trimmed.is_empty() { "" } else { path }
    };

    let rest = rest.trim_start_matches('/');
    let target = if rest.is_empty() {
        format!("/{}", language.as_str())
    } else {
        format!("/{}/{}", language.as_str(), rest)
    };

    with_query(&target, query)
}

/// Re-attach the query string to a rewritten path.
pub fn with_query(path: &str, query: Option<&str>) -> String {
    match query.filter(|query| !query.is_empty()) {
        Some(query) => format!("{}?{}", path, query),
        None => path.to_string(),
    }
}

/// Canonical form of a path: no trailing slash, except for the root.
///
/// Axum 0.8 does not match trailing slashes, so `/es/` - the canonical home URL
/// this site advertises - used to fall through to the fallback.
pub fn strip_trailing_slash(path: &str) -> Option<&str> {
    if path.len() > 1 && path.ends_with('/') {
        let trimmed = path.trim_end_matches('/');
        Some(if trimmed.is_empty() { "/" } else { trimmed })
    } else {
        None
    }
}

/// Top-level segments that belong to the site's own routes.
///
/// Some of them are two letters long (`me`), which would otherwise be mistaken
/// for a language tag and swapped away instead of prefixed.
const SITE_SEGMENTS: [&str; 10] = [
    "me",
    "blog",
    "contact",
    "contact_success",
    "ds2000",
    "static",
    "health",
    "robots.txt",
    "sitemap.xml",
    "favicon.ico",
];

/// `fr`, `pt-BR`, ... - a prefix a visitor meant as a language.
fn looks_like_language_tag(segment: &str) -> bool {
    if SITE_SEGMENTS.contains(&segment) {
        return false;
    }

    let (primary, region) = match segment.split_once('-') {
        Some((primary, region)) => (primary, Some(region)),
        None => (segment, None),
    };

    let valid_primary = primary.len() == 2 && primary.chars().all(|c| c.is_ascii_alphabetic());
    let valid_region = match region {
        Some(region) => region.len() == 2 && region.chars().all(|c| c.is_ascii_alphabetic()),
        None => true,
    };

    valid_primary && valid_region
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_first_segment() {
        assert_eq!(first_segment("/en/blog/4"), "en");
        assert_eq!(first_segment("/en"), "en");
        assert_eq!(first_segment("/"), "");
        assert_eq!(first_segment(""), "");
    }

    #[test]
    fn recognizes_language_tags() {
        assert!(looks_like_language_tag("fr"));
        assert!(looks_like_language_tag("pt-BR"));
        assert!(!looks_like_language_tag("ds2000"));
        assert!(!looks_like_language_tag(""));
    }

    #[test]
    fn never_mistakes_a_route_for_a_language() {
        for segment in SITE_SEGMENTS {
            assert!(
                !looks_like_language_tag(segment),
                "{} was taken for a language tag",
                segment
            );
        }
    }

    #[test]
    fn keeps_two_letter_routes_intact() {
        // `/me` is a page, not Montenegrin.
        assert_eq!(localized_target("/me", None, Language::Spanish), "/es/me");
        assert_eq!(localized_target("/me", None, Language::English), "/en/me");
    }

    #[test]
    fn swaps_unsupported_language_prefixes() {
        assert_eq!(
            localized_target("/fr/blog/4", None, Language::Spanish),
            "/es/blog/4"
        );
        assert_eq!(localized_target("/fr", None, Language::English), "/en");
    }

    #[test]
    fn prefixes_paths_that_lost_their_language() {
        assert_eq!(
            localized_target("/ds2000", None, Language::Spanish),
            "/es/ds2000"
        );
        assert_eq!(
            localized_target("/blog/4", None, Language::English),
            "/en/blog/4"
        );
    }

    #[test]
    fn redirects_the_root_to_a_language_home() {
        assert_eq!(localized_target("/", None, Language::Spanish), "/es");
        assert_eq!(localized_target("", None, Language::English), "/en");
    }

    #[test]
    fn strips_trailing_slashes() {
        assert_eq!(strip_trailing_slash("/es/"), Some("/es"));
        assert_eq!(strip_trailing_slash("/es/blog/"), Some("/es/blog"));
        assert_eq!(strip_trailing_slash("//"), Some("/"));
        assert_eq!(strip_trailing_slash("/es"), None);
        assert_eq!(strip_trailing_slash("/"), None);
    }

    #[test]
    fn keeps_the_query_string() {
        assert_eq!(
            localized_target("/blog", Some("page=2"), Language::Spanish),
            "/es/blog?page=2"
        );
    }

    #[test]
    fn targets_always_start_with_a_supported_language() {
        let cases = ["/", "/fr", "/fr/blog/4", "/ds2000", "/a/b/c", "/xx-YY/me"];
        for case in cases {
            let target = localized_target(case, None, Language::Spanish);
            assert_eq!(first_segment(&target), "es", "unexpected target {}", target);
        }
    }
}
