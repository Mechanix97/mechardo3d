use crate::json_ld;
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

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" { "es_ES" } else { "en_US" };

    // Generate JSON-LD schema
    let schema = json_ld::person_schema(language.as_str());
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", title);
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags
    context.insert("meta_description", "About Lucas Rack - Rust Developer, Blockchain Engineer, and Electronics enthusiast. Experience with Ethereum, Smart Contracts, and P2P protocols.");
    context.insert("meta_keywords", "lucas rack, rust developer, blockchain engineer, software engineer, ethereum");
    context.insert("og_title", title);
    context.insert("og_description", "About Lucas Rack - Rust Developer, Blockchain Engineer, and Electronics enthusiast. Experience with Ethereum, Smart Contracts, and P2P protocols.");
    context.insert("og_type", "profile");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", "me");

    let rendered = tera
        .render("me.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}
