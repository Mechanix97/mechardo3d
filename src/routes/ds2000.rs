use crate::language::Language;
use crate::responses::HtmlWithLang;
use crate::translations::get_translations_for_lang;
use axum::{extract::Path, Extension};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Context, Tera};

pub async fn ds2000(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");
    context.insert("t", &t);

    let rendered = tera
        .render("ds2000/ds2000.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}

pub async fn privacy_policy(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");
    context.insert("t", &t);

    let rendered = tera
        .render("ds2000/privacy_policy.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}

pub async fn terms_of_service(
    Path(lang): Path<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");
    context.insert("t", &t);

    let rendered = tera
        .render("ds2000/terms_of_service.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}
