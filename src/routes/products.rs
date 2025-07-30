use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn products(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Productos");
    context.insert("content", "Aquí encontrarás todos nuestros productos.");

    let rendered = tera
        .render("products.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
