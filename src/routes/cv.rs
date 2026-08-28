use axum::Extension;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tracing::warn;

use crate::extract::Lang;
use crate::pages;
use crate::state::AppState;

/// `GET /{lang}/cv` - the resume PDF, from the latest release of the private
/// resume repository.
///
/// Serving it through the site rather than linking to GitHub keeps that
/// repository private while the link on `/me` still resolves to whatever was
/// published last - there is no copy of the PDF in this repo to go stale.
pub async fn cv(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    if !state.resume.enabled() {
        return pages::not_found(&state, lang);
    }

    let Some(pdf) = state.resume.pdf(lang).await else {
        warn!("CV download is configured but no PDF could be served");
        return pages::server_error(&state, lang);
    };

    let filename = state.resume.asset_name(lang);
    let disposition = HeaderValue::from_str(&format!("attachment; filename=\"{}\"", filename))
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
