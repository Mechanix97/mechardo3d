use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn about(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("titulo", "Acerca de Mí");
    context.insert("mensaje", "Soy el creador de este sitio, ¡bienvenido!");

    let rendered = tera
        .render("about.html", &context)
        .expect("Error al renderizar la plantilla");
    Html(rendered)
}
