use axum::{Extension, Router, response::IntoResponse, routing::get};
use std::net::SocketAddr;
use tera::Tera;
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
        Err(_) => (StatusCode::NOT_FOUND, "Archivo no encontrado").into_response(),
    }
}

#[tokio::main]
async fn main() {
    // Inicializa Tera con las plantillas
    let tera = Tera::new("templates/**/*").expect("Error al inicializar Tera");

    // Configura el router con todas las rutas
    let app = Router::new()
        .route("/", get(routes::home::index))
        .route("/products", get(routes::products::products))
        .route("/blog", get(routes::blog::blog))
        .route("/blog/{id}", get(routes::blog::blog_post))
        .route("/contact", get(routes::contact::contact))
        .route("/me", get(routes::about::about))
        .route("/static/{*path}", get(serve_static))
        .layer(Extension(tera));

    // Inicia el servidor
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Servidor corriendo en http://{}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .expect("Error al iniciar el servidor");
}
