use axum::Extension;
use axum::response::Response;
use tracing::warn;

use crate::extract::Lang;
use crate::json_ld;
use crate::models::blog_post::BlogPostView;
use crate::pages::{self, page_meta};
use crate::state::AppState;

/// Blog posts previewed on the home page.
const RECENT_POSTS: usize = 3;

pub async fn index(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let posts = match state.blog.posts() {
        Ok(posts) => posts,
        Err(e) => {
            // The rest of the page is still worth serving.
            warn!("Home page rendered without blog posts: {}", e);
            Default::default()
        }
    };

    let recent: Vec<BlogPostView> = posts
        .iter()
        .take(RECENT_POSTS)
        .map(|post| BlogPostView::new(post, lang))
        .collect();

    // The home page's <title> leads with the brand and its tagline instead of
    // the generic "Inicio"/"Home" plus " | Mechardo Labs" every other page
    // gets - it's the one result Google shows for the site itself, so a
    // one-word title says nothing a visitor doesn't already know.
    let home_title = state
        .translations
        .text_or(lang, "page_titles.home_full", "Mechardo Labs")
        .to_string();

    let meta = page_meta(&state, lang, "home")
        .title(home_title.as_str())
        .full_title()
        .og_title(home_title)
        .schema(json_ld::organization_schema(&state.config, lang));

    let mut context = pages::base_context(&state, lang, &meta);
    context.insert("posts", &recent);

    pages::render(&state, "index.html", &context, lang)
}
