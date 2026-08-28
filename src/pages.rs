use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::Value;
use tera::Context;
use tracing::error;

use crate::language::Language;
use crate::responses::HtmlWithLang;
use crate::state::AppState;

const SITE_NAME: &str = "Mechardo Labs";
const DEFAULT_OG_IMAGE: &str = "static/images/og-image.png";

/// Everything the shared `<head>` needs for one page.
#[derive(Debug, Clone, Default)]
pub struct PageMeta {
    pub title: String,
    pub description: String,
    pub keywords: String,
    pub og_type: String,
    /// Social card title. `None` uses the page title plus the site name, which
    /// is what every page wants except the profile, where the person's own
    /// name should lead.
    pub og_title: Option<String>,
    /// Social card image, as a path under the static directory. `None` falls
    /// back to the site-wide card.
    pub og_image: Option<String>,
    /// Path after the language prefix, without a leading slash (`ds2000`).
    pub canonical_path: String,
    pub schema: Option<Value>,
}

impl PageMeta {
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            keywords: String::new(),
            og_type: "website".to_string(),
            og_title: None,
            og_image: None,
            canonical_path: String::new(),
            schema: None,
        }
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn keywords(mut self, keywords: impl Into<String>) -> Self {
        self.keywords = keywords.into();
        self
    }

    pub fn og_type(mut self, og_type: impl Into<String>) -> Self {
        self.og_type = og_type.into();
        self
    }

    /// Replace the whole social card title, site name included.
    pub fn og_title(mut self, og_title: impl Into<String>) -> Self {
        self.og_title = Some(og_title.into());
        self
    }

    /// Static path of the social card image (`images/og-me.png`).
    pub fn og_image(mut self, og_image: impl Into<String>) -> Self {
        self.og_image = Some(og_image.into());
        self
    }

    pub fn path(mut self, canonical_path: impl Into<String>) -> Self {
        self.canonical_path = canonical_path.into();
        self
    }

    pub fn schema(mut self, schema: Value) -> Self {
        self.schema = Some(schema);
        self
    }
}

/// Title, description and keywords for a page, read from the translations so
/// that every language gets its own copy instead of sharing English text.
pub fn page_meta(state: &AppState, lang: Language, page: &str) -> PageMeta {
    let title = state
        .translations
        .text_or(lang, &format!("page_titles.{}", page), page)
        .to_string();
    let description = state
        .translations
        .text_or(lang, &format!("meta.{}.description", page), "")
        .to_string();
    let keywords = state
        .translations
        .text_or(lang, &format!("meta.{}.keywords", page), "")
        .to_string();

    PageMeta::new(title, description).keywords(keywords)
}

#[derive(Serialize)]
struct Alternate {
    lang: &'static str,
    locale: &'static str,
    url: String,
}

/// Context shared by every page: language, translations, SEO tags and the
/// canonical/alternate URLs derived from the configured base URL.
pub fn base_context(state: &AppState, lang: Language, meta: &PageMeta) -> Context {
    let path = meta.canonical_path.trim_start_matches('/');

    let alternates: Vec<Alternate> = Language::ALL
        .iter()
        .map(|language| Alternate {
            lang: language.as_str(),
            locale: language.locale(),
            url: canonical_url(state, *language, path),
        })
        .collect();

    let mut context = Context::new();
    context.insert("lang", lang.as_str());
    context.insert("locale", lang.locale());
    context.insert("t", state.translations.for_lang(lang));
    context.insert("title", &meta.title);
    context.insert("meta_description", &meta.description);
    context.insert("meta_keywords", &meta.keywords);
    let og_title = match &meta.og_title {
        Some(title) => title.clone(),
        None => format!("{} | {}", meta.title, SITE_NAME),
    };
    context.insert("og_title", &og_title);
    context.insert("og_description", &meta.description);
    context.insert(
        "og_image",
        &state.config.url(
            meta.og_image
                .as_deref()
                .unwrap_or(DEFAULT_OG_IMAGE)
                .trim_start_matches('/'),
        ),
    );
    context.insert("og_type", &meta.og_type);
    context.insert("og_locale", lang.og_locale());
    context.insert("canonical_path", path);
    context.insert("canonical_url", &canonical_url(state, lang, path));
    context.insert(
        "x_default_url",
        &canonical_url(state, Language::default(), path),
    );
    context.insert("robots", "index, follow");
    context.insert("base_url", &state.config.base_url);
    context.insert("alternates", &alternates);

    if let Some(schema) = &meta.schema {
        let encoded = serde_json::to_string(schema).unwrap_or_default();
        context.insert("json_ld_schema", &encoded);
    }

    context
}

fn canonical_url(state: &AppState, lang: Language, path: &str) -> String {
    if path.is_empty() {
        state.config.url(lang.as_str())
    } else {
        state.config.url(&format!("{}/{}", lang.as_str(), path))
    }
}

/// Render a template, turning failures into a 500 page instead of a panic.
pub fn render(state: &AppState, template: &str, context: &Context, lang: Language) -> Response {
    render_with_status(state, template, context, lang, StatusCode::OK)
}

pub fn render_with_status(
    state: &AppState,
    template: &str,
    context: &Context,
    lang: Language,
    status: StatusCode,
) -> Response {
    match state.tera.render(template, context) {
        Ok(html) => HtmlWithLang::new(html, lang)
            .with_status(status)
            .with_secure_cookie(state.config.cookie_secure)
            .into_response(),
        Err(e) => {
            error!(
                "Error rendering {}: {}",
                template,
                describe_error(&e as &dyn std::error::Error)
            );
            server_error(state, lang)
        }
    }
}

/// Localized 404 page.
pub fn not_found(state: &AppState, lang: Language) -> Response {
    let meta = PageMeta::new(
        state
            .translations
            .text_or(lang, "page_titles.not_found", "404"),
        state
            .translations
            .text_or(lang, "errors.not_found_message", "Page not found."),
    );
    let mut context = base_context(state, lang, &meta);
    context.insert("robots", "noindex, follow");

    match state.tera.render("404.html", &context) {
        Ok(html) => HtmlWithLang::new(html, lang)
            .with_status(StatusCode::NOT_FOUND)
            .with_secure_cookie(state.config.cookie_secure)
            .into_response(),
        Err(e) => {
            error!(
                "Error rendering 404.html: {}",
                describe_error(&e as &dyn std::error::Error)
            );
            minimal_page(
                state,
                lang,
                StatusCode::NOT_FOUND,
                "page_titles.not_found",
                "errors.not_found_message",
            )
        }
    }
}

/// Localized 500 page. Deliberately does not go through Tera: it is the
/// fallback used when rendering itself is what failed.
pub fn server_error(state: &AppState, lang: Language) -> Response {
    minimal_page(
        state,
        lang,
        StatusCode::INTERNAL_SERVER_ERROR,
        "page_titles.server_error",
        "errors.server_error_message",
    )
}

fn minimal_page(
    state: &AppState,
    lang: Language,
    status: StatusCode,
    title_key: &str,
    message_key: &str,
) -> Response {
    let title = state.translations.text_or(lang, title_key, "Error");
    let message = state
        .translations
        .text_or(lang, message_key, "Something went wrong.");
    let home = format!("/{}", lang.as_str());

    let html = format!(
        "<!DOCTYPE html><html lang=\"{lang}\"><head><meta charset=\"utf-8\">\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\
         <meta name=\"robots\" content=\"noindex\">\
         <title>{title} | Mechardo Labs</title></head>\
         <body><h1>{title}</h1><p>{message}</p><p><a href=\"{home}\">Mechardo Labs</a></p></body></html>",
        lang = escape_html(lang.as_str()),
        title = escape_html(title),
        message = escape_html(message),
        home = escape_html(&home),
    );

    HtmlWithLang::new(html, lang)
        .with_status(status)
        .with_secure_cookie(state.config.cookie_secure)
        .into_response()
}

fn escape_html(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Flatten an error and its sources into one log line.
pub fn describe_error(error: &dyn std::error::Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(inner) = source {
        message.push_str(&format!(": {}", inner));
        source = inner.source();
    }
    message
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_html_entities() {
        assert_eq!(
            escape_html("<script>&\"x\""),
            "&lt;script&gt;&amp;&quot;x&quot;"
        );
    }

    #[test]
    fn page_meta_defaults_to_a_website() {
        let meta = PageMeta::new("Title", "Description").path("blog");
        assert_eq!(meta.og_type, "website");
        assert_eq!(meta.canonical_path, "blog");
        assert!(meta.schema.is_none());
    }
}
