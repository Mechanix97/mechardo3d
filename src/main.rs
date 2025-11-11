use axum::{Extension, Router, response::IntoResponse, routing::get};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tera::Tera;
use tokio::sync::RwLock;
use tracing::info;

mod data;
mod date_format;
mod language;
mod models;
mod routes;


// Redirect root to default language (Spanish)
async fn redirect_to_default_lang() -> impl IntoResponse {
    use axum::response::Redirect;
    Redirect::permanent(&format!("/{}", language::Language::default().as_str()))
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
        .layer(Extension(rate_limit));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Server running on http://127.0.0.1:3000/");
    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Error starting server");
}
