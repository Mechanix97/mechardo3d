use axum::Extension;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tracing::warn;

use crate::data::release_asset::ReleaseAssetStore;
use crate::extract::Lang;
use crate::language::Language;
use crate::pages;
use crate::state::AppState;

/// `GET /{lang}/cv` - the resume PDF, from the latest release of the private
/// resume repository.
pub async fn cv(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    serve(&state, &state.resume, lang).await
}

/// `GET /{lang}/tpp` - the capstone report, same mechanism.
pub async fn report(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    serve(&state, &state.report, lang).await
}

/// Stream a release asset as a download.
///
/// Serving these through the site rather than linking to GitHub keeps the
/// repositories private while the links still resolve to whatever was
/// published last - there is no copy in this repo to go stale.
async fn serve(state: &AppState, store: &ReleaseAssetStore, lang: Language) -> Response {
    if !store.enabled() {
        return pages::not_found(state, lang);
    }

    let Some(pdf) = store.pdf(lang).await else {
        warn!("A download is configured but no PDF could be served");
        return pages::server_error(state, lang);
    };

    let disposition = HeaderValue::from_str(&format!(
        "attachment; filename=\"{}\"",
        store.download_name(lang)
    ))
    .unwrap_or_else(|_| HeaderValue::from_static("attachment"));

    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/pdf"),
            ),
            (header::CONTENT_DISPOSITION, disposition),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=3600"),
            ),
        ],
        pdf.to_vec(),
    )
        .into_response()
}
