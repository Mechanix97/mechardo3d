use crate::language::Language;
use crate::translations::get_translations_for_lang;
use axum::{extract::Path, Extension, response::Html};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

pub async fn me(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> Html<String> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", if language == Language::English { "About me" } else { "Sobre mí" });
    context.insert("t", &t);

    let rendered = tera
        .render("me.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
