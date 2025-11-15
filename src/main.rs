use axum::{
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    Extension, Router, routing::get,
};
use serde_json::Value;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tera::Tera;
use tokio::sync::RwLock;
use tracing::info;

mod data;
mod date_format;
mod language;
mod language_detection;
mod models;
mod responses;
mod routes;
mod translations;

// Redirect root to detected or default language
async fn redirect_to_default_lang(headers: HeaderMap) -> impl IntoResponse {
    let detected_lang = language_detection::detect_language(&headers);
    Redirect::permanent(&format!("/{}", detected_lang.as_str()))
}

// Handlers for routes without language prefix that redirect to language-prefixed versions
async fn redirect_me(headers: HeaderMap) -> impl IntoResponse {
    let detected_lang = language_detection::detect_language(&headers);
    Redirect::temporary(&format!("/{}/me", detected_lang.as_str()))
}

async fn redirect_contact(headers: HeaderMap) -> impl IntoResponse {
    let detected_lang = language_detection::detect_language(&headers);
    Redirect::temporary(&format!("/{}/contact", detected_lang.as_str()))
}

async fn redirect_contact_success(headers: HeaderMap) -> impl IntoResponse {
    let detected_lang = language_detection::detect_language(&headers);
    Redirect::temporary(&format!("/{}/contact_success", detected_lang.as_str()))
}

async fn redirect_blog(headers: HeaderMap) -> impl IntoResponse {
    let detected_lang = language_detection::detect_language(&headers);
    Redirect::temporary(&format!("/{}/blog", detected_lang.as_str()))
}

// Fallback handler for truly unmapped routes
async fn fallback(
    axum::extract::Path(path): axum::extract::Path<String>,
    headers: HeaderMap,
) -> impl IntoResponse {
    // Ignore static files and direct to 404
    if path.starts_with("static/") {
        return Redirect::temporary("/404").into_response();
    }

    let detected_lang = language_detection::detect_language(&headers);
    Redirect::temporary(&format!("/{}/{}", detected_lang.as_str(), path)).into_response()
}

async fn serve_static(axum::extract::Path(path): axum::extract::Path<String>) -> impl IntoResponse {
    use axum::http::{StatusCode, header};
    use tokio::fs;

    let path = format!("static/{}", path);
    match fs::read(&path).await {
        Ok(content) => {
            let mime_type = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime_type.as_ref())], content).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut tera = Tera::new("templates/**/*").expect("Error initializing Tera");
    // Register the date_format filter
    tera.register_filter("date_format", date_format::date_format);
    let rate_limit: routes::contact::RateLimitState = Arc::new(RwLock::new(HashMap::new()));

    // Load translations
    let translations: Arc<HashMap<String, Value>> = Arc::new(translations::load_translations());
    info!("Loaded translations for languages: {:?}", translations.keys());

    // Redirect root to default language
    let app = Router::new()
        .route("/", get(redirect_to_default_lang))
        .route("/static/{*path}", get(serve_static))
        // Routes without language prefix (redirect to language-prefixed versions)
        .route("/me", get(redirect_me))
        .route("/contact", get(redirect_contact).post(redirect_contact))
        .route("/contact_success", get(redirect_contact_success))
        .route("/blog", get(redirect_blog))
        // Language-prefixed routes (most specific routes FIRST)
        .route("/{lang}/me", get(routes::me::me))
        .route("/{lang}/contact", get(routes::contact::contact).post(routes::contact::contact_submit))
        .route("/{lang}/contact_success", get(routes::contact::contact_success))
        .route("/{lang}/blog/{id}", get(routes::blog::blog_post))
        .route("/{lang}/blog", get(routes::blog::blog))
        .route("/{lang}/ds2000/terms-of-service", get(routes::ds2000::terms_of_service))
        .route("/{lang}/ds2000/privacy-policy", get(routes::ds2000::privacy_policy))
        .route("/{lang}/ds2000", get(routes::ds2000::ds2000))
        .route("/{lang}", get(routes::home::index))
        // Fallback for unmapped routes
        .fallback(|path: axum::extract::Path<String>, headers: HeaderMap| {
            fallback(path, headers)
        })
        .layer(Extension(tera))
        .layer(Extension(rate_limit))
        .layer(Extension(translations));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server running on http://127.0.0.1:3000/");
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Error starting server");
}
