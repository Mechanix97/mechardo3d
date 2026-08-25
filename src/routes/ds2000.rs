use axum::Extension;
use axum::response::Response;

use crate::extract::Lang;
use crate::json_ld;
use crate::pages::{self, page_meta};
use crate::state::AppState;

pub async fn ds2000(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    render_product_page(&state, lang, "ds2000", "ds2000", "ds2000/ds2000.html")
}

pub async fn privacy_policy(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    render_product_page(
        &state,
        lang,
        "privacy_policy",
        "ds2000/privacy-policy",
        "ds2000/privacy_policy.html",
    )
}

pub async fn terms_of_service(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    render_product_page(
        &state,
        lang,
        "terms_of_service",
        "ds2000/terms-of-service",
        "ds2000/terms_of_service.html",
    )
}

fn render_product_page(
    state: &AppState,
    lang: crate::language::Language,
    page: &str,
    path: &str,
    template: &str,
) -> Response {
    let meta = page_meta(state, lang, page)
        .og_type("product")
        .path(path)
        .schema(json_ld::product_schema(&state.config, lang));

    let context = pages::base_context(state, lang, &meta);
    pages::render(state, template, &context, lang)
}
