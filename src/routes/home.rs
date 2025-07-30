use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn index(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Inicio");
    context.insert(
        "content",
        "Explora nuestros productos, blog, contacto y más sobre mí.",
    );

    let rendered = tera
        .render("index.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
