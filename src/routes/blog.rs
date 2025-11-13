use crate::data::blog_data::get_posts;
use crate::language::Language;
use crate::translations::get_translations_for_lang;
use axum::{Extension, extract::Path, response::Html};
use chrono::{DateTime, Utc};
use rand::rng;
use rand::seq::IteratorRandom;
use serde::{Serialize};
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
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut posts = get_posts().expect("Error loading posts");
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    let t = get_translations_for_lang(&translations, language.as_str());

    // Convert posts to localized view
    let posts_view: Vec<BlogPostView> = posts.iter().map(|post| BlogPostView {
        id: post.id.clone(),
        title: post.get_title(language.as_str()).to_string(),
        summary: post.get_summary(language.as_str()).map(|s| s.to_string()),
        route: post.route.clone(),
        thumbnail: post.thumbnail.clone(),
        date: post.date,
    }).collect();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "Blog");
    context.insert("posts", &posts_view);
    context.insert("t", &t);

    let rendered = tera
        .render("blog.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn blog_post(
    Path((lang, id)): Path<(String, String)>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let posts = get_posts().expect("Error loading posts");
    let post: &crate::models::blog_post::BlogPost = posts
        .iter()
        .find(|p: &&crate::models::blog_post::BlogPost| p.id == id)
        .expect("Post not found");

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

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", &post_view.title);
    context.insert("post", &post_view);
    context.insert("t", &t);

    if let Some(post_content_route) = &post.route {
        let content = fs::read_to_string(format!("templates/blog/{}", post_content_route))
            .expect("Post content not found");
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

    let rendered = tera
        .render("blog_post.html", &context)
        .expect("Error rendering post");
    Html(rendered)
}
