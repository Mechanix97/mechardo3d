use axum::Router;
use axum::extract::Extension;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::middleware as axum_middleware;
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::get;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{Level, error, info};

mod client_ip;
mod config;
mod data;
mod date_format;
mod extract;
mod json_ld;
mod language;
mod language_detection;
mod middleware;
mod models;
mod pages;
mod rate_limit;
mod responses;
mod routes;
mod sitemap;
mod state;
mod static_files;
mod translations;

use crate::config::AppConfig;
use crate::language::Language;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_max_level(log_level()).init();

    let config = AppConfig::from_env();
    let addr = config.bind_addr;

    let state = match AppState::build(config) {
        Ok(state) => state,
        Err(e) => {
            error!(
                "Failed to initialize templates: {}",
                pages::describe_error(&e as &dyn std::error::Error)
            );
            std::process::exit(1);
        }
    };

    let listener = match TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("Server listening on http://{}/", addr);

    if let Err(e) = axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    {
        error!("Server stopped: {}", e);
        std::process::exit(1);
    }
}

fn router(state: AppState) -> Router {
    Router::new()
        // Site-wide endpoints
        .route("/", get(redirect_root))
        .route("/health", get(health))
        .route("/robots.txt", get(sitemap::robots))
        .route("/sitemap.xml", get(sitemap::sitemap))
        .route("/favicon.ico", get(static_files::serve_favicon))
        .route("/static/{*path}", get(static_files::serve_static))
        // Language-prefixed pages (most specific first)
        .route("/{lang}/me", get(routes::me::me))
        .route(
            "/{lang}/contact",
            get(routes::contact::contact).post(routes::contact::contact_submit),
        )
        .route(
            "/{lang}/contact_success",
            get(routes::contact::contact_success),
        )
        .route("/{lang}/blog/{id}", get(routes::blog::blog_post))
        .route("/{lang}/blog", get(routes::blog::blog))
        .route(
            "/{lang}/ds2000/terms-of-service",
            get(routes::ds2000::terms_of_service),
        )
        .route(
            "/{lang}/ds2000/privacy-policy",
            get(routes::ds2000::privacy_policy),
        )
        .route("/{lang}/ds2000", get(routes::ds2000::ds2000))
        .route("/{lang}", get(routes::home::index))
        .fallback(not_found)
        // Applied bottom-up: the state is available to the middlewares above it.
        .layer(axum_middleware::from_fn(middleware::security_headers))
        .layer(axum_middleware::from_fn(middleware::request_log))
        .layer(Extension(state))
}

/// `GET /` - send visitors to their language.
async fn redirect_root(headers: HeaderMap) -> Response {
    let language = language_detection::detect_language(&headers);
    responses::language_redirect(&format!("/{}", language.as_str()))
}

/// `GET /health` - readiness probe for Docker and uptime checks.
async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

/// Unmatched routes.
///
/// A path that already carries a supported language renders a 404 page; one
/// that does not is redirected to its localized equivalent. Both outcomes are
/// terminal, which the previous fallback was not: it re-prefixed every path,
/// so unknown URLs bounced between redirects.
async fn not_found(
    Extension(state): Extension<AppState>,
    uri: Uri,
    headers: HeaderMap,
) -> Response {
    // `/es/` and `/es/blog/` are the same pages as `/es` and `/es/blog`.
    if let Some(path) = extract::strip_trailing_slash(uri.path()) {
        return Redirect::permanent(&extract::with_query(path, uri.query())).into_response();
    }

    if let Some(language) = Language::from_str(extract::first_segment(uri.path())) {
        return pages::not_found(&state, language);
    }

    let language = language_detection::detect_language(&headers);
    responses::language_redirect(&extract::localized_target(
        uri.path(),
        uri.query(),
        language,
    ))
}

/// Honour `RUST_LOG` (already set in docker-compose) without pulling in the
/// full `EnvFilter` machinery.
fn log_level() -> Level {
    let raw = match std::env::var("RUST_LOG") {
        Ok(raw) => raw,
        Err(_) => return Level::INFO,
    };
    parse_log_level(&raw).unwrap_or(Level::INFO)
}

fn parse_log_level(raw: &str) -> Option<Level> {
    raw.split(',')
        .filter_map(|directive| {
            let level = directive.rsplit('=').next().unwrap_or_default().trim();
            match level.to_ascii_lowercase().as_str() {
                "trace" => Some(Level::TRACE),
                "debug" => Some(Level::DEBUG),
                "info" => Some(Level::INFO),
                "warn" => Some(Level::WARN),
                "error" => Some(Level::ERROR),
                _ => None,
            }
        })
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_plain_log_levels() {
        assert_eq!(parse_log_level("debug"), Some(Level::DEBUG));
        assert_eq!(parse_log_level(" WARN "), Some(Level::WARN));
    }

    #[test]
    fn reads_targeted_log_directives() {
        assert_eq!(parse_log_level("mechardo3d=debug,info"), Some(Level::DEBUG));
        assert_eq!(parse_log_level("hyper=off"), None);
    }
}
