use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn about(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Acerca de Mí");
    context.insert("content", "Soy el creador de este sitio, ¡bienvenido!");

    let rendered = tera
        .render("about.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
