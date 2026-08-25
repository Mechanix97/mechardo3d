use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use rand::rng;
use rand::seq::IteratorRandom;
use tracing::{error, warn};

use crate::extract::Lang;
use crate::json_ld;
use crate::language::Language;
use crate::models::blog_post::BlogPostView;
use crate::pages::{self, page_meta};
use crate::state::AppState;

/// How many other posts are suggested at the bottom of a post.
const RELATED_POSTS: usize = 2;
/// Search engines cut descriptions around this length.
const META_DESCRIPTION_CHARS: usize = 155;

pub async fn blog(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let posts = match state.blog.posts() {
        Ok(posts) => posts,
        Err(e) => {
            error!("Blog index unavailable: {}", e);
            return pages::server_error(&state, lang);
        }
    };

    let posts_view = BlogPostView::from_posts(&posts, lang);

    let meta = page_meta(&state, lang, "blog").path("blog");
    let schema = json_ld::webpage_schema(
        &state.config,
        lang,
        &meta.title,
        &meta.description,
        "Blog",
        &format!("{}/blog", lang.as_str()),
    );
    let meta = meta.schema(schema);

    let mut context = pages::base_context(&state, lang, &meta);
    context.insert("posts", &posts_view);

    pages::render(&state, "blog.html", &context, lang)
}

pub async fn blog_post(
    Lang(lang): Lang,
    Path((_lang, id)): Path<(String, String)>,
    Extension(state): Extension<AppState>,
) -> Response {
    let posts = match state.blog.posts() {
        Ok(posts) => posts,
        Err(e) => {
            error!("Blog post {} unavailable: {}", id, e);
            return pages::server_error(&state, lang);
        }
    };

    let Some(post) = posts.iter().find(|post| post.id == id) else {
        return pages::not_found(&state, lang);
    };

    let view = BlogPostView::new(post, lang);

    let content = match post.route.as_deref() {
        Some(route) => match post_body(&state, route, lang) {
            Some(content) => content,
            None => return pages::server_error(&state, lang),
        },
        None => view.summary.clone().unwrap_or_default(),
    };

    let canonical_path = format!("blog/{}", view.id);
    let description = truncate_chars(
        view.summary.as_deref().unwrap_or(""),
        META_DESCRIPTION_CHARS,
    );
    let schema = json_ld::blog_post_schema(
        &state.config,
        lang,
        &view,
        &description,
        &format!("{}/{}", lang.as_str(), canonical_path),
    );

    let meta = page_meta(&state, lang, "blog_post")
        .title(view.title.as_str())
        .og_type("article")
        .path(canonical_path.as_str())
        .schema(schema);
    let meta = if description.is_empty() {
        meta
    } else {
        meta.description(description)
    };

    let related: Vec<BlogPostView> = posts
        .iter()
        .filter(|candidate| candidate.id != id)
        .choose_multiple(&mut rng(), RELATED_POSTS)
        .into_iter()
        .map(|post| BlogPostView::new(post, lang))
        .collect();

    let mut context = pages::base_context(&state, lang, &meta);
    context.insert("post", &view);
    context.insert("content", &content);
    context.insert("related_posts", &related);

    pages::render(&state, "blog_post.html", &context, lang)
}

/// Post body in the requested language, falling back to the default language
/// when a translation has not been written yet.
fn post_body(state: &AppState, route: &str, lang: Language) -> Option<String> {
    match state.blog.content(route, lang) {
        Ok(content) => Some(content.to_string()),
        Err(e) => {
            warn!("Missing {} body for post {}: {}", lang.as_str(), route, e);
            if lang == Language::default() {
                return None;
            }
            match state.blog.content(route, Language::default()) {
                Ok(content) => Some(content.to_string()),
                Err(e) => {
                    error!("No body available for post {}: {}", route, e);
                    None
                }
            }
        }
    }
}

fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut truncated: String = text.chars().take(limit.saturating_sub(1)).collect();
    truncated = truncated.trim_end().to_string();
    truncated.push('…');
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_short_text_untouched() {
        assert_eq!(truncate_chars("short", 10), "short");
        assert_eq!(truncate_chars("", 10), "");
    }

    #[test]
    fn truncates_on_character_boundaries() {
        let text = "á".repeat(200);
        let truncated = truncate_chars(&text, META_DESCRIPTION_CHARS);
        assert_eq!(truncated.chars().count(), META_DESCRIPTION_CHARS);
        assert!(truncated.ends_with('…'));
    }
}
