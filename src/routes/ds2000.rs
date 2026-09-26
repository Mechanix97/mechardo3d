use axum::Extension;
use axum::response::Response;
use serde_json::json;

use crate::extract::Lang;
use crate::json_ld;
use crate::language::Language;
use crate::pages::{self, page_meta};
use crate::state::AppState;

/// Breadcrumb label for the DS2000 pages, matching the nav link text in
/// `base.html` rather than `page_titles.ds2000` (which carries the longer,
/// SEO-oriented `<title>` copy).
const DS2000_CRUMB: &str = "DS2000";

pub async fn ds2000(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let meta = page_meta(&state, lang, "ds2000").path("ds2000");

    let schema = json_ld::ds2000_schema(&state.config, lang, &meta.title, &meta.description);
    let breadcrumbs = json_ld::breadcrumbs(
        &state.config,
        lang,
        &[(home_crumb(&state, lang), ""), (DS2000_CRUMB, "ds2000")],
    );
    let meta = meta.og_type("product").schema(json!([schema, breadcrumbs]));

    let context = pages::base_context(&state, lang, &meta);
    pages::render(&state, "ds2000/ds2000.html", &context, lang)
}

pub async fn privacy_policy(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    legal_page(
        &state,
        lang,
        "privacy_policy",
        "ds2000/privacy-policy",
        "ds2000/privacy_policy.html",
    )
}

pub async fn terms_of_service(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    legal_page(
        &state,
        lang,
        "terms_of_service",
        "ds2000/terms-of-service",
        "ds2000/terms_of_service.html",
    )
}

/// The DS2000's terms of service and privacy policy describe the site's own
/// legal terms, not the device itself, so they get a plain `WebPage` (like
/// any other content page) rather than `ds2000_schema` - and a breadcrumb one
/// level deeper, under DS2000.
fn legal_page(
    state: &AppState,
    lang: Language,
    page: &str,
    path: &str,
    template: &str,
) -> Response {
    let meta = page_meta(state, lang, page).path(path);

    let schema = json_ld::webpage_schema(
        &state.config,
        lang,
        &meta.title,
        &meta.description,
        "WebPage",
        &format!("{}/{}", lang.as_str(), path),
    );
    let breadcrumbs = json_ld::breadcrumbs(
        &state.config,
        lang,
        &[
            (home_crumb(state, lang), ""),
            (DS2000_CRUMB, "ds2000"),
            (meta.title.as_str(), path),
        ],
    );
    let meta = meta.schema(json!([schema, breadcrumbs]));

    let context = pages::base_context(state, lang, &meta);
    pages::render(state, template, &context, lang)
}

fn home_crumb(state: &AppState, lang: Language) -> &str {
    state.translations.text_or(lang, "page_titles.home", "Home")
}
