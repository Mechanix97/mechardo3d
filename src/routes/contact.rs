use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn contact(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("titulo", "Contacto");
    context.insert("mensaje", "Ponte en contacto con nosotros.");

    let rendered = tera
        .render("contact.html", &context)
        .expect("Error al renderizar la plantilla");
    Html(rendered)
}
