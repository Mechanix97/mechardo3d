use axum::{Extension, Router, response::IntoResponse, routing::get};
use chrono::{DateTime, Datelike, Utc};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tera::{Result as TeraResult, Tera, Value};
use tokio::sync::RwLock;
use tracing::info;

mod data;
mod models;
mod routes;

fn date_format(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    let date_str = value
        .as_str()
        .ok_or_else(|| tera::Error::msg("Expected a string for date"))?;
    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| tera::Error::msg(format!("Invalid date format: {}", e)))?
        .with_timezone(&Utc);

    let month = match date.month() {
        1 => "enero",
        2 => "febrero",
        3 => "marzo",
        4 => "abril",
        5 => "mayo",
        6 => "junio",
        7 => "julio",
        8 => "agosto",
        9 => "septiembre",
        10 => "octubre",
        11 => "noviembre",
        12 => "diciembre",
        _ => "desconocido",
    };

    let formatted = format!("{:02} de {} de {}", date.day(), month, date.year());
    Ok(Value::String(formatted))
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
    // Registrar el filtro date_format
    tera.register_filter("date_format", date_format);
    let rate_limit: routes::contact::RateLimitState = Arc::new(RwLock::new(HashMap::new()));

    let app = Router::new()
        .route("/", get(routes::home::index))
        .route("/ds2000", get(routes::ds2000::ds2000))
        .route(
            "/ds2000/terms-of-service",
            get(routes::ds2000::terms_of_service),
        )
        .route(
            "/ds2000/privacy-policy",
            get(routes::ds2000::privacy_policy),
        )
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
