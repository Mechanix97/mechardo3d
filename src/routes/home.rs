use crate::data::blog_data::get_posts;
use crate::language::Language;
use crate::translations::get_translations_for_lang;
use axum::{extract::Path, Extension, response::Html};
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
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);

    // Obtener posts del blog
    let posts_view = match get_posts() {
        Ok(mut posts) => {
            // Ordenar por fecha descendente y tomar los 3 más recientes
            posts.sort_by(|a, b| b.date.cmp(&a.date));
            posts.into_iter().take(3).map(|post| BlogPostView {
                id: post.id.clone(),
                title: post.get_title(language.as_str()).to_string(),
                summary: post.get_summary(language.as_str()).map(|s| s.to_string()),
                route: post.route.clone(),
                thumbnail: post.thumbnail.clone(),
                date: post.date,
            }).collect::<Vec<BlogPostView>>()
        }
        Err(e) => {
            eprintln!("Error loading blog posts from data/blog_posts.json: {}", e);
            Vec::new() // Usar lista vacía para no romper el renderizado
        }
    };

    // Get translations for current language
    let t = get_translations_for_lang(&translations, language.as_str());

    // Configurar el contexto de Tera
    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", if language == Language::English { "Home" } else { "Inicio" });
    context.insert("posts", &posts_view);
    context.insert("t", &t);

    // Renderizar la plantilla
    match tera.render("index.html", &context) {
        Ok(rendered) => Html(rendered),
        Err(e) => {
            eprintln!("Error rendering index.html: {}", e);
            Html(format!(
                "<html><body><h1>Error rendering page</h1><p>{}</p></body></html>",
                e
            ))
        }
    }
}
