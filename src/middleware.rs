use std::time::Instant;

use axum::extract::{Extension, Request};
use axum::http::{HeaderName, HeaderValue, header};
use axum::middleware::Next;
use axum::response::Response;
use tracing::{info, warn};

use crate::state::AppState;

/// Baseline security headers for every response.
///
/// `Content-Security-Policy` is opt-in through configuration because the pages
/// load reCAPTCHA and Plausible and use inline bootstrap scripts; a policy has
/// to be written for that mix before it is turned on.
pub async fn security_headers(
    Extension(state): Extension<AppState>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=(), interest-cohort=()"),
    );

    if state.config.hsts_enabled {
        headers.insert(
            header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        );
    }

    if let Some(policy) = &state.config.content_security_policy
        && let Ok(value) = HeaderValue::from_str(policy)
    {
        headers.insert(header::CONTENT_SECURITY_POLICY, value);
    }

    response
}

/// One structured log line per request.
pub async fn request_log(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let started = Instant::now();

    let response = next.run(request).await;

    let status = response.status().as_u16();
    let latency_ms = started.elapsed().as_millis() as u64;

    if response.status().is_server_error() {
        warn!(%method, %path, status, latency_ms, "request failed");
    } else {
        info!(%method, %path, status, latency_ms, "request");
    }

    response
}
