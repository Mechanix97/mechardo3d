use crate::data::blog_data::get_posts;
use crate::json_ld;
use crate::language::Language;
use crate::responses::HtmlWithLang;
use crate::translations::get_translations_for_lang;
use axum::{Extension, extract::Path};
use chrono::{DateTime, Utc};
use rand::rng;
use rand::seq::IteratorRandom;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
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

pub async fn blog(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut posts = get_posts().expect("Error loading posts");
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    let t = get_translations_for_lang(&translations, language.as_str());

    // Convert posts to localized view
    let posts_view: Vec<BlogPostView> = posts
        .iter()
        .map(|post| BlogPostView {
            id: post.id.clone(),
            title: post.get_title(language.as_str()).to_string(),
            summary: post.get_summary(language.as_str()).map(|s| s.to_string()),
            route: post.route.clone(),
            thumbnail: post.thumbnail.clone(),
            date: post.date,
        })
        .collect();

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" { "es_ES" } else { "en_US" };

    // Generate JSON-LD schema
    let schema = json_ld::webpage_schema(
        "Blog",
        "Mechardo Labs Blog - Articles about Rust, Blockchain, Electronics, and Software Development.",
        "Blog",
        language.as_str()
    );
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "Blog");
    context.insert("posts", &posts_view);
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags
    context.insert("meta_description", "Mechardo Labs Blog - Articles about Rust, Blockchain, Electronics, and Software Development.");
    context.insert("meta_keywords", "blog, rust, blockchain, tutorials, software development");
    context.insert("og_title", "Blog");
    context.insert("og_description", "Mechardo Labs Blog - Articles about Rust, Blockchain, Electronics, and Software Development.");
    context.insert("og_type", "website");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", "blog");

    let rendered = tera
        .render("blog.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}

pub async fn blog_post(
    Path((lang, id)): Path<(String, String)>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> Result<HtmlWithLang, (axum::http::StatusCode, String)> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let posts = get_posts().map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error loading posts: {}", e),
        )
    })?;
    let post: &crate::models::blog_post::BlogPost = posts
        .iter()
        .find(|p: &&crate::models::blog_post::BlogPost| p.id == id)
        .ok_or_else(|| {
            (
                axum::http::StatusCode::NOT_FOUND,
                format!("Post not found: {}", id),
            )
        })?;

    let t = get_translations_for_lang(&translations, language.as_str());

    // Convert post to localized view
    let post_view = BlogPostView {
        id: post.id.clone(),
        title: post.get_title(language.as_str()).to_string(),
        summary: post.get_summary(language.as_str()).map(|s| s.to_string()),
        route: post.route.clone(),
        thumbnail: post.thumbnail.clone(),
        date: post.date,
    };

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" { "es_ES" } else { "en_US" };

    // Build canonical path for individual blog post
    let canonical_path = format!("blog/{}", id);

    // Generate JSON-LD schema for blog post
    let description = post_view.summary.as_ref()
        .map(|s| s.chars().take(155).collect::<String>())
        .unwrap_or_else(|| "Mechardo Labs Blog".to_string());
    let date_str = post_view.date.format("%Y-%m-%d").to_string();
    let schema = json_ld::blog_post_schema(
        &post_view.title,
        &description,
        &date_str,
        "Lucas Rack",
        language.as_str()
    );
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", &post_view.title);
    context.insert("post", &post_view);
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags for blog post
    let meta_description = post_view.summary.as_ref()
        .map(|s| s.chars().take(155).collect::<String>())
        .unwrap_or_else(|| "Mechardo Labs Blog - Articles about Rust, Blockchain, Electronics, and Software Development.".to_string());
    context.insert("meta_description", &meta_description);
    context.insert("meta_keywords", "blog, rust, blockchain, tutorials, software development");
    context.insert("og_title", &post_view.title);
    context.insert("og_description", &meta_description);
    context.insert("og_type", "article");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", &canonical_path);

    if let Some(post_content_route) = &post.route {
        let content = fs::read_to_string(format!(
            "templates/blog/{}/{}.html",
            post_content_route,
            language.as_str()
        ))
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Post content not found: {} - Error: {}",
                    post_content_route, e
                ),
            )
        })?;
        context.insert("content", &content);
    } else {
        context.insert("content", &post_view.summary);
    }

    // Convert related posts to localized view
    let related_posts: Vec<BlogPostView> = posts
        .iter()
        .filter(|p| p.id != id)
        .choose_multiple(&mut rng(), 2)
        .into_iter()
        .map(|p| BlogPostView {
            id: p.id.clone(),
            title: p.get_title(language.as_str()).to_string(),
            summary: p.get_summary(language.as_str()).map(|s| s.to_string()),
            route: p.route.clone(),
            thumbnail: p.thumbnail.clone(),
            date: p.date,
        })
        .collect();

    context.insert("related_posts", &related_posts);

    let rendered = tera.render("blog_post.html", &context).map_err(|e| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error rendering post: {}", e),
        )
    })?;
    Ok(HtmlWithLang::new(rendered, language))
}
