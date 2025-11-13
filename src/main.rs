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
mod routes;
mod translations;

// Redirect root to detected or default language
async fn redirect_to_default_lang(headers: HeaderMap) -> impl IntoResponse {
    // Detect language from Accept-Language header
    let detected_lang = language_detection::detect_from_accept_language(&headers);
    Redirect::permanent(&format!("/{}", detected_lang.as_str()))
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
        // Language-prefixed routes
        .route("/{lang}", get(routes::home::index))
        .route("/{lang}/ds2000", get(routes::ds2000::ds2000))
        .route(
            "/{lang}/ds2000/terms-of-service",
            get(routes::ds2000::terms_of_service),
        )
        .route(
            "/{lang}/ds2000/privacy-policy",
            get(routes::ds2000::privacy_policy),
        )
        .route("/{lang}/blog", get(routes::blog::blog))
        .route("/{lang}/blog/{id}", get(routes::blog::blog_post))
        .route(
            "/{lang}/contact",
            get(routes::contact::contact).post(routes::contact::contact_submit),
        )
        .route("/{lang}/contact_success", get(routes::contact::contact_success))
        .route("/{lang}/me", get(routes::me::me))
        .route("/static/{*path}", get(serve_static))
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
