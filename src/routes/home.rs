use crate::data::blog_data::get_posts;
use crate::json_ld;
use crate::language::Language;
use crate::responses::HtmlWithLang;
use crate::translations::get_translations_for_lang;
use axum::{Extension, extract::Path};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Serialize, Clone)]
struct BlogPostView {
    id: String,
    title: String,
    summary: Option<String>,
    route: Option<String>,
    thumbnail: Option<String>,
    date: DateTime<Utc>,
}

pub async fn index(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);

    // Get blog posts
    let posts_view = match get_posts() {
        Ok(mut posts) => {
            // Sort by date descending and take the 3 most recent
            posts.sort_by(|a, b| b.date.cmp(&a.date));
            posts
                .into_iter()
                .take(3)
                .map(|post| BlogPostView {
                    id: post.id.clone(),
                    title: post.get_title(language.as_str()).to_string(),
                    summary: post.get_summary(language.as_str()).map(|s| s.to_string()),
                    route: post.route.clone(),
                    thumbnail: post.thumbnail.clone(),
                    date: post.date,
                })
                .collect::<Vec<BlogPostView>>()
        }
        Err(e) => {
            eprintln!("Error loading blog posts from data/blog_posts.json: {}", e);
            Vec::new() // Use empty list to avoid breaking rendering
        }
    };

    // Get translations for current language
    let t = get_translations_for_lang(&translations, language.as_str());

    let title = t
        .get("page_titles")
        .and_then(|pt| pt.get("home"))
        .and_then(|v| v.as_str())
        .unwrap_or("Home");

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" {
        "es_ES"
    } else {
        "en_US"
    };

    // Generate JSON-LD schema
    let schema = json_ld::organization_schema(language.as_str());
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    // Configure Tera context
    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", title);
    context.insert("posts", &posts_view);
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags
    context.insert("meta_description", "Mechardo Labs - Software Engineer specialized in Rust, Blockchain, and Electronics. Explore my portfolio, projects, and blog.");
    context.insert(
        "meta_keywords",
        "software engineer, rust, blockchain, discord, electronics, developer portfolio",
    );
    context.insert("og_title", title);
    context.insert("og_description", "Mechardo Labs - Software Engineer specialized in Rust, Blockchain, and Electronics. Explore my portfolio, projects, and blog.");
    context.insert("og_type", "website");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", "");

    // Render template
    match tera.render("index.html", &context) {
        Ok(rendered) => HtmlWithLang::new(rendered, language),
        Err(e) => {
            eprintln!("Error rendering index.html: {}", e);
            HtmlWithLang::new(
                format!(
                    "<html><body><h1>Error rendering page</h1><p>{}</p></body></html>",
                    e
                ),
                language,
            )
        }
    }
}
