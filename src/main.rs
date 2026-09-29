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
                "Failed to initialize application state: {}",
                pages::describe_error(e.as_ref())
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
    .with_graceful_shutdown(shutdown_signal())
    .await
    {
        error!("Server stopped: {}", e);
        std::process::exit(1);
    }

    info!("Server shut down gracefully");
}

/// Resolves once the process receives Ctrl+C or, on Unix, SIGTERM - the signal
/// `docker stop` / a compose recreate sends before escalating to SIGKILL. Letting
/// `axum::serve` drain in-flight requests on either signal avoids that escalation
/// and the abrupt connection drops that come with it.
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received Ctrl+C, shutting down"),
        _ = terminate => info!("Received SIGTERM, shutting down"),
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
        .route("/{lang}/cv", get(routes::downloads::cv))
        .route("/{lang}/tpp", get(routes::downloads::report))
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

/// Exercises the real `router()` end to end (routing, the `Lang` extractor,
/// the fallback and the middleware stack) instead of only the helper
/// functions it's built from. This is what would have caught `/cv` resolving
/// to the home page instead of `/{lang}/cv` - see #88.
#[cfg(test)]
mod router_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, header};
    use tower::ServiceExt;

    fn test_router() -> Router {
        let state = AppState::build(AppConfig::for_tests()).expect("state should build");
        router(state)
    }

    async fn get(uri: &str) -> Response {
        request(uri, None).await
    }

    async fn get_with_header(uri: &str, name: &str, value: &str) -> Response {
        request(uri, Some((name, value))).await
    }

    async fn request(uri: &str, extra_header: Option<(&str, &str)>) -> Response {
        let mut builder = Request::builder().uri(uri);
        if let Some((name, value)) = extra_header {
            builder = builder.header(name, value);
        }
        test_router()
            .oneshot(builder.body(Body::empty()).expect("valid request"))
            .await
            .expect("router should always answer")
    }

    fn location(response: &Response) -> &str {
        response
            .headers()
            .get(header::LOCATION)
            .expect("a redirect should carry a Location header")
            .to_str()
            .expect("Location should be ASCII")
    }

    #[tokio::test]
    async fn root_redirects_by_accept_language() {
        let response = get_with_header("/", "accept-language", "en").await;
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(location(&response), "/en");
        assert_eq!(
            response
                .headers()
                .get(header::VARY)
                .and_then(|v| v.to_str().ok()),
            Some("accept-language, cookie")
        );
    }

    #[tokio::test]
    async fn root_defaults_to_spanish_without_hints() {
        let response = get("/").await;
        assert_eq!(location(&response), "/es");
    }

    #[tokio::test]
    async fn unprefixed_pages_redirect_to_the_visitors_language() {
        // `/cv` is the regression case for #88: it used to lose its segment
        // and redirect to `/es` instead of `/es/cv`.
        for (path, expected) in [("/me", "/es/me"), ("/cv", "/es/cv"), ("/tpp", "/es/tpp")] {
            let response = get(path).await;
            assert_eq!(
                location(&response),
                expected,
                "unexpected redirect for {path}"
            );
        }
    }

    #[tokio::test]
    async fn unsupported_language_prefix_is_swapped() {
        let response = get("/fr/blog/4").await;
        assert_eq!(location(&response), "/es/blog/4");
    }

    #[tokio::test]
    async fn trailing_slash_is_stripped_permanently() {
        for (path, expected) in [("/es/", "/es"), ("/es/blog/", "/es/blog")] {
            let response = get(path).await;
            assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
            assert_eq!(location(&response), expected);
        }
    }

    #[tokio::test]
    async fn unknown_localized_path_renders_a_404_with_the_language_cookie() {
        let response = get("/es/no-existe").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .expect("404 should still set the language cookie")
            .to_str()
            .expect("cookie should be ASCII");
        assert!(cookie.starts_with("language=es"));
    }

    #[tokio::test]
    async fn pages_carry_the_security_headers() {
        let response = get("/es/contact").await;
        assert_eq!(response.status(), StatusCode::OK);

        let headers = response.headers();
        assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
        assert_eq!(headers.get("x-frame-options").unwrap(), "SAMEORIGIN");
        assert_eq!(
            headers.get("referrer-policy").unwrap(),
            "strict-origin-when-cross-origin"
        );
        // hsts_enabled defaults to true in AppConfig::for_tests().
        assert!(headers.get("strict-transport-security").is_some());
    }
}
