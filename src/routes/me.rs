use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn me(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Acerca de Mí");

    let rendered = tera
        .render("me.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
