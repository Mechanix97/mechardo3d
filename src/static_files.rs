use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use axum::Extension;
use axum::extract::Path as AxumPath;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tokio::fs;
use tracing::debug;

use crate::state::AppState;

const LONG_CACHE_EXTENSIONS: [&str; 10] = [
    "png", "jpg", "jpeg", "avif", "webp", "gif", "svg", "ico", "woff", "woff2",
];

/// `GET /static/{*path}`
pub async fn serve_static(
    AxumPath(path): AxumPath<String>,
    Extension(state): Extension<AppState>,
    headers: HeaderMap,
) -> Response {
    serve(&state, &path, &headers).await
}

/// `GET /favicon.ico` - browsers ask for it at the root.
pub async fn serve_favicon(Extension(state): Extension<AppState>, headers: HeaderMap) -> Response {
    serve(&state, "images/favicon/favicon.ico", &headers).await
}

async fn serve(state: &AppState, requested: &str, headers: &HeaderMap) -> Response {
    let Some(path) = safe_join(&state.config.static_dir, requested) else {
        debug!("Rejected static path: {:?}", requested);
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(metadata) = fs::metadata(&path).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !metadata.is_file() {
        return StatusCode::NOT_FOUND.into_response();
    }

    let etag = etag_for(&metadata);
    if let Some(etag) = etag.as_deref()
        && request_matches_etag(headers, etag)
    {
        return (StatusCode::NOT_MODIFIED, cache_headers(&path, Some(etag))).into_response();
    }

    match fs::read(&path).await {
        Ok(contents) => (
            StatusCode::OK,
            cache_headers(&path, etag.as_deref()),
            contents,
        )
            .into_response(),
        Err(e) => {
            debug!("Failed to read {}: {}", path.display(), e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}

/// Resolve a request path inside `root`, or refuse it.
///
/// Path traversal was possible here: the requested path used to be
/// concatenated onto `static/`, so `../` sequences escaped the directory and
/// could read any file the process could, including `secrets/`.
pub fn safe_join(root: &Path, requested: &str) -> Option<PathBuf> {
    let mut path = root.to_path_buf();
    let mut segments = 0;

    for segment in requested.split(['/', '\\']) {
        if segment.is_empty() || segment == "." {
            continue;
        }
        // Rejects `..` along with dotfiles, which are never public assets.
        if segment.starts_with('.') || segment.contains('\0') {
            return None;
        }
        path.push(segment);
        segments += 1;
    }

    if segments == 0 { None } else { Some(path) }
}

fn etag_for(metadata: &std::fs::Metadata) -> Option<String> {
    let modified = metadata.modified().ok()?;
    let seconds = modified.duration_since(UNIX_EPOCH).ok()?.as_secs();
    Some(format!("\"{:x}-{:x}\"", metadata.len(), seconds))
}

fn request_matches_etag(headers: &HeaderMap, etag: &str) -> bool {
    headers
        .get(header::IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(',')
                .any(|candidate| candidate.trim().trim_start_matches("W/") == etag)
        })
}

fn cache_headers(path: &Path, etag: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();

    let mime = mime_guess::from_path(path).first_or_octet_stream();
    if let Ok(value) = HeaderValue::from_str(mime.as_ref()) {
        headers.insert(header::CONTENT_TYPE, value);
    }

    headers.insert(header::CACHE_CONTROL, cache_control(path));

    if let Some(etag) = etag
        && let Ok(value) = HeaderValue::from_str(etag)
    {
        headers.insert(header::ETAG, value);
    }

    headers
}

fn cache_control(path: &Path) -> HeaderValue {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    if LONG_CACHE_EXTENSIONS.contains(&extension.as_str()) {
        HeaderValue::from_static("public, max-age=2592000")
    } else {
        HeaderValue::from_static("public, max-age=3600, must-revalidate")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_normal_paths() {
        assert_eq!(
            safe_join(Path::new("static"), "js/contact.js"),
            Some(PathBuf::from("static/js/contact.js"))
        );
        assert_eq!(
            safe_join(Path::new("static"), "/images/logo.png"),
            Some(PathBuf::from("static/images/logo.png"))
        );
    }

    #[test]
    fn refuses_traversal() {
        for attempt in [
            "../secrets/recaptcha.env",
            "..%2fsecrets",
            "images/../../secrets/recaptcha.env",
            "..\\secrets\\recaptcha.env",
            "./../../etc/passwd",
        ] {
            let joined = safe_join(Path::new("static"), attempt);
            assert!(
                joined.is_none() || joined.as_ref().is_some_and(|p| p.starts_with("static")),
                "escaped the static directory: {}",
                attempt
            );
            assert!(
                !format!("{:?}", joined).contains(".."),
                "kept a traversal segment: {}",
                attempt
            );
        }
    }

    #[test]
    fn refuses_dotfiles_and_empty_paths() {
        assert_eq!(safe_join(Path::new("static"), ".env"), None);
        assert_eq!(safe_join(Path::new("static"), ""), None);
        assert_eq!(safe_join(Path::new("static"), "///"), None);
    }

    #[test]
    fn caches_images_longer_than_code() {
        assert_eq!(
            cache_control(Path::new("static/images/logo.png")),
            HeaderValue::from_static("public, max-age=2592000")
        );
        assert_eq!(
            cache_control(Path::new("static/style.css")),
            HeaderValue::from_static("public, max-age=3600, must-revalidate")
        );
    }

    #[test]
    fn matches_etags_including_weak_ones() {
        let mut headers = HeaderMap::new();
        headers.insert(header::IF_NONE_MATCH, HeaderValue::from_static("\"a-b\""));
        assert!(request_matches_etag(&headers, "\"a-b\""));
        assert!(!request_matches_etag(&headers, "\"c-d\""));

        let mut weak = HeaderMap::new();
        weak.insert(
            header::IF_NONE_MATCH,
            HeaderValue::from_static("W/\"a-b\", \"z\""),
        );
        assert!(request_matches_etag(&weak, "\"a-b\""));
    }
}
