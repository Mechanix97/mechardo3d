use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::language::Language;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LocalizedText {
    pub es: String,
    pub en: String,
}

impl LocalizedText {
    pub fn get(&self, lang: Language) -> &str {
        match lang {
            Language::English => &self.en,
            Language::Spanish => &self.es,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BlogPost {
    pub id: String,
    pub title: LocalizedText,
    pub summary: Option<LocalizedText>,
    /// Directory under `templates/blog/` holding the post body, when the post
    /// has one. Posts without a route fall back to their summary.
    pub route: Option<String>,
    pub thumbnail: Option<String>,
    #[serde(deserialize_with = "deserialize_date")]
    pub date: DateTime<Utc>,
}

impl BlogPost {
    pub fn get_title(&self, lang: Language) -> &str {
        self.title.get(lang)
    }

    pub fn get_summary(&self, lang: Language) -> Option<&str> {
        self.summary.as_ref().map(|summary| summary.get(lang))
    }
}

/// A blog post already resolved to a single language, ready for templates.
#[derive(Serialize, Clone, Debug)]
pub struct BlogPostView {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub route: Option<String>,
    pub thumbnail: Option<String>,
    pub date: DateTime<Utc>,
}

impl BlogPostView {
    pub fn new(post: &BlogPost, lang: Language) -> Self {
        Self {
            id: post.id.clone(),
            title: post.get_title(lang).to_string(),
            summary: post.get_summary(lang).map(|s| s.to_string()),
            route: post.route.clone(),
            thumbnail: post.thumbnail.as_deref().map(normalize_asset_path),
            date: post.date,
        }
    }

    pub fn from_posts(posts: &[BlogPost], lang: Language) -> Vec<Self> {
        posts.iter().map(|post| Self::new(post, lang)).collect()
    }
}

/// Turn an asset path from the data file into a usable URL path.
///
/// Historical entries use Windows-style separators (`\static\images\x.png`),
/// which only work because browsers rewrite backslashes in URLs.
fn normalize_asset_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/') || normalized.starts_with("http") {
        normalized
    } else {
        format!("/{}", normalized)
    }
}

fn deserialize_date<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let naive_date = NaiveDate::parse_from_str(&s, "%d-%m-%Y").map_err(serde::de::Error::custom)?;
    let datetime = naive_date
        .and_hms_opt(0, 0, 0)
        .ok_or_else(|| serde::de::Error::custom("Invalid date"))?
        .and_utc();
    Ok(datetime)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BlogPost {
        serde_json::from_str(
            r#"{
                "id": "42",
                "title": { "es": "Hola", "en": "Hello" },
                "summary": { "es": "Resumen", "en": "Summary" },
                "thumbnail": "\\static\\images\\thumb.png",
                "route": "iot-alarm",
                "date": "18-08-2025"
            }"#,
        )
        .expect("sample post should deserialize")
    }

    #[test]
    fn deserializes_day_first_dates() {
        let post = sample();
        assert_eq!(post.date.format("%Y-%m-%d").to_string(), "2025-08-18");
    }

    #[test]
    fn resolves_localized_fields() {
        let post = sample();
        assert_eq!(post.get_title(Language::English), "Hello");
        assert_eq!(post.get_title(Language::Spanish), "Hola");
        assert_eq!(post.get_summary(Language::English), Some("Summary"));
    }

    #[test]
    fn normalizes_windows_style_asset_paths() {
        let view = BlogPostView::new(&sample(), Language::English);
        assert_eq!(view.thumbnail.as_deref(), Some("/static/images/thumb.png"));
    }

    #[test]
    fn keeps_absolute_asset_paths_untouched() {
        assert_eq!(normalize_asset_path("/static/a.png"), "/static/a.png");
        assert_eq!(
            normalize_asset_path("https://cdn.example.com/a.png"),
            "https://cdn.example.com/a.png"
        );
    }

    #[test]
    fn rejects_unparseable_dates() {
        let result: Result<BlogPost, _> = serde_json::from_str(
            r#"{
                "id": "1",
                "title": { "es": "a", "en": "a" },
                "date": "2025-08-18"
            }"#,
        );
        assert!(result.is_err());
    }
}
