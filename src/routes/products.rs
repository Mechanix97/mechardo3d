use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn products(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("titulo", "Nuestros Productos");
    context.insert("mensaje", "Aquí encontrarás todos nuestros productos.");

    let rendered = tera
        .render("products.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
