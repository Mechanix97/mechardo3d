
use axum::{response::Html, Extension};
use tera::{Tera, Context};

pub async fn index(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("titulo", "Bienvenido a Mi Sitio");
    context.insert("mensaje", "Explora nuestros productos, blog, contacto y más sobre mí.");

    let rendered = tera
        .render("index.html", &context)
        .expect("Error al renderizar la plantilla");
    Html(rendered)
}
