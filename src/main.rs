use axum::{Extension, Router, response::IntoResponse, routing::get};
use std::net::SocketAddr;
use tera::Tera;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod data;
mod models;
mod routes;

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

    let tera = Tera::new("templates/**/*").expect("Error initializing Tera");
    let rate_limit: routes::contact::RateLimitState = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/", get(routes::home::index))
        .route("/products", get(routes::products::products))
        .route("/blog", get(routes::blog::blog))
        .route("/blog/{:id}", get(routes::blog::blog_post))
        .route(
            "/contact",
            get(routes::contact::contact).post(routes::contact::contact_submit),
        )
        .route("/contact_success", get(routes::contact::contact_success))
        .route("/me", get(routes::me::me))
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
