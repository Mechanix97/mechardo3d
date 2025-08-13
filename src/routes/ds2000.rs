use axum::{Extension, response::Html};
use tera::{Context, Tera};

pub async fn ds2000(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "DS2000");

    let rendered = tera
        .render("ds2000/ds2000.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn privacy_policy(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "DS2000");

    let rendered = tera
        .render("ds2000/privacy_policy.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn terms_of_service(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "DS2000");

    let rendered = tera
        .render("ds2000/terms_of_service.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
