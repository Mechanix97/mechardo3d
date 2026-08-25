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

    let meta =
        page_meta(&state, lang, "home").schema(json_ld::organization_schema(&state.config, lang));

    let mut context = pages::base_context(&state, lang, &meta);
    context.insert("posts", &recent);

    pages::render(&state, "index.html", &context, lang)
}
