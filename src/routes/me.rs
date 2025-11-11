use crate::language::Language;
use axum::{extract::Path, Extension, response::Html};
use tera::{Context, Tera};

pub async fn me(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", if language == Language::English { "About me" } else { "Sobre mí" });

    let rendered = tera
        .render("me.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
