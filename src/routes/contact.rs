use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn contact(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Contacto");
    context.insert("content", "Ponte en contacto con nosotros.");

    let rendered = tera
        .render("contact.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
