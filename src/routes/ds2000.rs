use crate::json_ld;
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

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" { "es_ES" } else { "en_US" };

    // Generate JSON-LD schema
    let schema = json_ld::product_schema(language.as_str());
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags
    context.insert("meta_description", "DS2000 - USB Discord Button Box with 3 programmable buttons and RGB LEDs. Mute, deafen, and disconnect with one click.");
    context.insert("meta_keywords", "discord button, discord mute, usb button box, streaming gear, ds2000");
    context.insert("og_title", "DS2000");
    context.insert("og_description", "DS2000 - USB Discord Button Box with 3 programmable buttons and RGB LEDs. Mute, deafen, and disconnect with one click.");
    context.insert("og_type", "product");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", "ds2000");

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

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" { "es_ES" } else { "en_US" };

    // Generate JSON-LD schema
    let schema = json_ld::product_schema(language.as_str());
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags
    context.insert("meta_description", "DS2000 - USB Discord Button Box with 3 programmable buttons and RGB LEDs. Mute, deafen, and disconnect with one click.");
    context.insert("meta_keywords", "discord button, discord mute, usb button box, streaming gear, ds2000");
    context.insert("og_title", "DS2000 - Privacy Policy");
    context.insert("og_description", "DS2000 - USB Discord Button Box with 3 programmable buttons and RGB LEDs. Mute, deafen, and disconnect with one click.");
    context.insert("og_type", "product");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", "ds2000/privacy-policy");

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

    // Determine og_locale based on language
    let og_locale = if language.as_str() == "es" { "es_ES" } else { "en_US" };

    // Generate JSON-LD schema
    let schema = json_ld::product_schema(language.as_str());
    let json_ld_schema = serde_json::to_string(&schema).unwrap_or_default();

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", "DS2000");
    context.insert("t", &t);
    context.insert("json_ld_schema", &json_ld_schema);

    // SEO meta tags
    context.insert("meta_description", "DS2000 - USB Discord Button Box with 3 programmable buttons and RGB LEDs. Mute, deafen, and disconnect with one click.");
    context.insert("meta_keywords", "discord button, discord mute, usb button box, streaming gear, ds2000");
    context.insert("og_title", "DS2000 - Terms of Service");
    context.insert("og_description", "DS2000 - USB Discord Button Box with 3 programmable buttons and RGB LEDs. Mute, deafen, and disconnect with one click.");
    context.insert("og_type", "product");
    context.insert("og_locale", og_locale);
    context.insert("canonical_path", "ds2000/terms-of-service");

    let rendered = tera
        .render("ds2000/terms_of_service.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}
