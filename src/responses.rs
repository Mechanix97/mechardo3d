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
        headers.insert(
            "Set-Cookie",
            format!(
                "language={}; Path=/; Max-Age=31536000; SameSite=Lax",
                self.lang.as_str()
            )
            .parse()
            .unwrap(),
        );
        // Clear old 'lang' cookie from previous versions
        headers.append(
            "Set-Cookie",
            "lang=; Path=/; Max-Age=0; SameSite=Lax".parse().unwrap(),
        );

        (StatusCode::OK, headers, self.html).into_response()
    }
}
