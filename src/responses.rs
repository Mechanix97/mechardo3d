use crate::language::Language;
use axum::{
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};

/// Response wrapper that automatically sets a language cookie
pub struct HtmlWithLang {
    html: String,
    lang: Language,
}

impl HtmlWithLang {
    pub fn new(html: String, lang: Language) -> Self {
        Self { html, lang }
    }
}

impl IntoResponse for HtmlWithLang {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "text/html; charset=utf-8".parse().unwrap());

        (StatusCode::OK, headers, self.html).into_response()
    }
}
