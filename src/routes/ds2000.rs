use crate::language::Language;
use axum::{extract::Path, Extension, response::Html};
use tera::{Context, Tera};

pub async fn ds2000(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");

    let rendered = tera
        .render("ds2000/ds2000.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn privacy_policy(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");

    let rendered = tera
        .render("ds2000/privacy_policy.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn terms_of_service(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");

    let rendered = tera
        .render("ds2000/terms_of_service.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
