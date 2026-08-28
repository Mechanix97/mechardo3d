use axum::Extension;
use axum::response::Response;

use crate::extract::Lang;
use crate::json_ld;
use crate::pages::{self, page_meta};
use crate::state::AppState;

pub async fn me(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    // Shared on its own, this page should introduce the person, not the site:
    // the card leads with the name and the role rather than "About Me". Both
    // come from the translations the page already renders, so they cannot
    // drift apart from the heading.
    let name = state.translations.text_or(lang, "about.name", "Lucas Rack");
    let role = state.translations.text_or(lang, "about.role", "");
    let social_title = if role.is_empty() {
        name.to_string()
    } else {
        format!("{} — {}", name, role)
    };

    let meta = page_meta(&state, lang, "about")
        .og_type("profile")
        .og_title(social_title)
        .og_image("static/images/og-me.png")
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
