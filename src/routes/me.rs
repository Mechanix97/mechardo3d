use crate::language::Language;
use crate::responses::HtmlWithLang;
use crate::translations::get_translations_for_lang;
use axum::Extension;
use axum::extract::Path;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

pub async fn me(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let title = t
        .get("page_titles")
        .and_then(|pt| pt.get("about"))
        .and_then(|v| v.as_str())
        .unwrap_or("About Me");

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", title);
    context.insert("t", &t);

    let rendered = tera
        .render("me.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}
