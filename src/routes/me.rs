use axum::Extension;
use axum::response::Response;

use crate::extract::Lang;
use crate::json_ld;
use crate::pages::{self, page_meta};
use crate::state::AppState;

pub async fn me(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let meta = page_meta(&state, lang, "about")
        .og_type("profile")
        .path("me")
        .schema(json_ld::person_schema(&state.config, lang));

    let mut context = pages::base_context(&state, lang, &meta);
    // Absent when no GitHub token is configured, which is what hides the
    // download button rather than rendering a link that cannot resolve.
    if state.resume.enabled() {
        context.insert("cv_url", &format!("/{}/cv", lang.as_str()));
    }

    pages::render(&state, "me.html", &context, lang)
}
