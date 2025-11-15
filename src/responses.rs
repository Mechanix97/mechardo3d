use crate::language::Language;
use axum::{
    http::StatusCode,
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
        let cookie_header = format!(
            "language={}; Path=/; Max-Age=31536000; SameSite=Lax",
            self.lang.as_str()
        );

        (
            StatusCode::OK,
            [
                ("Content-Type", "text/html; charset=utf-8"),
                ("Set-Cookie", &cookie_header),
            ],
            self.html,
        )
            .into_response()
    }
}
