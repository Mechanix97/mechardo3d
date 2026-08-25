use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};

use crate::language::Language;

/// One year, matching the cookie the front-end sets.
const LANGUAGE_COOKIE_MAX_AGE: i64 = 60 * 60 * 24 * 365;

/// HTML response that also persists the language preference.
///
/// Sending the cookie from the server means the preference survives requests
/// made before (or without) JavaScript, which is what the language detection in
/// [`crate::language_detection`] reads back on the next visit.
pub struct HtmlWithLang {
    html: String,
    lang: Language,
    status: StatusCode,
    secure_cookie: bool,
}

impl HtmlWithLang {
    pub fn new(html: String, lang: Language) -> Self {
        Self {
            html,
            lang,
            status: StatusCode::OK,
            secure_cookie: false,
        }
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    pub fn with_secure_cookie(mut self, secure: bool) -> Self {
        self.secure_cookie = secure;
        self
    }
}

impl IntoResponse for HtmlWithLang {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );

        if let Ok(value) = HeaderValue::from_str(self.lang.as_str()) {
            headers.insert(header::CONTENT_LANGUAGE, value);
        }

        let cookie = language_cookie(self.lang, self.secure_cookie);
        if let Ok(value) = HeaderValue::from_str(&cookie) {
            headers.insert(header::SET_COOKIE, value);
        }

        (self.status, headers, self.html).into_response()
    }
}

fn language_cookie(lang: Language, secure: bool) -> String {
    let mut cookie = format!(
        "language={}; Path=/; Max-Age={}; SameSite=Lax",
        lang.as_str(),
        LANGUAGE_COOKIE_MAX_AGE
    );
    if secure {
        cookie.push_str("; Secure");
    }
    cookie
}

/// Temporary redirect whose target depends on the visitor's language.
///
/// `Vary` keeps caches from serving one visitor's language redirect to another.
pub fn language_redirect(target: &str) -> Response {
    let mut response = Redirect::temporary(target).into_response();
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("accept-language, cookie"),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_the_language_cookie() {
        assert_eq!(
            language_cookie(Language::English, false),
            "language=en; Path=/; Max-Age=31536000; SameSite=Lax"
        );
    }

    #[test]
    fn marks_the_cookie_secure_when_asked() {
        assert!(language_cookie(Language::Spanish, true).ends_with("; Secure"));
    }

    #[test]
    fn html_responses_carry_the_language_cookie() {
        let response =
            HtmlWithLang::new("<p>hi</p>".to_string(), Language::English).into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        assert!(cookie.starts_with("language=en"));
        assert_eq!(
            response
                .headers()
                .get(header::CONTENT_LANGUAGE)
                .and_then(|value| value.to_str().ok()),
            Some("en")
        );
    }

    #[test]
    fn language_redirects_vary_on_language_inputs() {
        let response = language_redirect("/es/blog");
        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response
                .headers()
                .get(header::VARY)
                .and_then(|value| value.to_str().ok()),
            Some("accept-language, cookie")
        );
        assert_eq!(
            response
                .headers()
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("/es/blog")
        );
    }
}
