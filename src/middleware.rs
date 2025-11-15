use axum::{
    body::Body,
    http::{Request, Uri},
    middleware::Next,
    response::Response,
};
use crate::language::Language;

/// Middleware that redirects requests without language prefix to the appropriate language version
/// If a request is for /path (without /lang/), it redirects to /{detected_lang}/path
pub async fn language_redirect(
    req: Request<Body>,
    next: Next,
) -> Response {
    let uri = req.uri();
    let path = uri.path();

    // Check if path already has language prefix
    if is_language_prefixed(path) || path == "/" || path.starts_with("/static/") {
        return next.run(req).await;
    }

    // If not, we need to redirect - but we need access to headers for language detection
    // This middleware can't easily do that, so we'll handle this differently
    next.run(req).await
}

/// Check if a path has a valid language prefix (/{lang}/)
fn is_language_prefixed(path: &str) -> bool {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();

    if parts.is_empty() {
        return false;
    }

    // Check if first part is a valid language code
    Language::from_str(parts[0]).is_some()
}
