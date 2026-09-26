use axum::Extension;
use axum::extract::{ConnectInfo, Form};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Json, Redirect, Response};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tracing::{error, info, warn};

use crate::client_ip::client_ip;
use crate::data::messages::ContactMessage;
use crate::extract::Lang;
use crate::json_ld;
use crate::language::Language;
use crate::pages::{self, page_meta};
use crate::state::AppState;

const MAX_NAME_CHARS: usize = 100;
const MAX_EMAIL_CHARS: usize = 254;
const RECAPTCHA_VERIFY_URL: &str = "https://www.google.com/recaptcha/api/siteverify";
/// The action name `static/js/contact.js` executes the widget with
/// (`grecaptcha.execute(key, { action: 'submit' })`). A token solved for any
/// other action shouldn't verify here.
const RECAPTCHA_ACTION: &str = "submit";

#[derive(Deserialize, Serialize)]
pub struct ContactForm {
    name: String,
    email: String,
    message: String,
    #[serde(rename = "g-recaptcha-response", default)]
    g_recaptcha_response: String,
}

#[derive(Deserialize)]
struct RecaptchaResponse {
    success: bool,
    /// Absent for reCAPTCHA v2 and for error responses. Required here: a
    /// missing score (`success: true` included) usually means the secret key
    /// isn't actually a v3 key, which would otherwise silently accept every
    /// submission regardless of `min_score`.
    score: Option<f32>,
    /// The action the widget was executed with (v3). Absent on failure.
    #[serde(default)]
    action: Option<String>,
    /// The hostname the token was solved on. Absent on failure.
    #[serde(default)]
    hostname: Option<String>,
    #[serde(rename = "error-codes", default)]
    error_codes: Vec<String>,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

/// Error returned to the contact form's `fetch` call.
pub struct ContactError {
    status: StatusCode,
    message: String,
    retry_after: Option<i64>,
}

impl ContactError {
    fn new(state: &AppState, lang: Language, status: StatusCode, key: &str) -> Self {
        Self {
            status,
            message: state
                .translations
                .text_or(lang, &format!("errors.{}", key), key)
                .to_string(),
            retry_after: None,
        }
    }

    fn retry_after(mut self, seconds: i64) -> Self {
        self.retry_after = Some(seconds);
        self
    }
}

impl IntoResponse for ContactError {
    fn into_response(self) -> Response {
        let mut response = (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response();

        if let Some(seconds) = self.retry_after
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }

        response
    }
}

/// `GET /{lang}/contact`
pub async fn contact(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let meta = page_meta(&state, lang, "contact").path("contact");
    let schema = json_ld::webpage_schema(
        &state.config,
        lang,
        &meta.title,
        &meta.description,
        "ContactPage",
        &format!("{}/contact", lang.as_str()),
    );
    let meta = meta.schema(schema);

    let mut context = pages::base_context(&state, lang, &meta);
    context.insert("recaptcha_site_key", &state.config.recaptcha.site_key);

    pages::render(&state, "contact.html", &context, lang)
}

/// `GET /{lang}/contact_success`
pub async fn contact_success(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let meta = page_meta(&state, lang, "message_sent").path("contact_success");
    let schema = json_ld::webpage_schema(
        &state.config,
        lang,
        &meta.title,
        &meta.description,
        "ContactPage",
        &format!("{}/contact_success", lang.as_str()),
    );
    let meta = meta.schema(schema);

    let context = pages::base_context(&state, lang, &meta);
    pages::render(&state, "contact_success.html", &context, lang)
}

/// `POST /{lang}/contact`
pub async fn contact_submit(
    Lang(lang): Lang,
    Extension(state): Extension<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Form(form): Form<ContactForm>,
) -> Result<Redirect, ContactError> {
    let ip = client_ip(&headers, addr, state.config.trust_proxy_headers);
    let now = Utc::now();

    if let Some(retry_after) = state.contact_rate_limit.retry_after(&ip, now) {
        info!("Rate limit hit for {}", ip);
        return Err(
            ContactError::new(&state, lang, StatusCode::TOO_MANY_REQUESTS, "rate_limit")
                .retry_after(retry_after),
        );
    }

    let message = validate(&form, state.config.max_message_chars).map_err(|key| {
        info!("Rejected contact submission from {}: {}", ip, key);
        ContactError::new(&state, lang, StatusCode::BAD_REQUEST, key)
    })?;

    verify_recaptcha(&state, lang, &form.g_recaptcha_response, &ip).await?;

    let stored = ContactMessage {
        name: message.name,
        email: message.email,
        message: message.message,
        timestamp: now,
    };

    if let Err(e) = state.messages.append(&stored).await {
        error!("Failed to store contact message: {}", e);
        return Err(ContactError::new(
            &state,
            lang,
            StatusCode::INTERNAL_SERVER_ERROR,
            "save_failed",
        ));
    }

    state.contact_rate_limit.record(&ip, now);
    info!("Stored contact message from {}", ip);

    Ok(Redirect::to(&format!("/{}/contact_success", lang.as_str())))
}

struct ValidatedMessage {
    name: String,
    email: String,
    message: String,
}

/// Validate the submitted form, returning the translation key of the failure.
fn validate(
    form: &ContactForm,
    max_message_chars: usize,
) -> Result<ValidatedMessage, &'static str> {
    let name = form.name.trim();
    let email = form.email.trim();
    let message = form.message.trim();

    if name.is_empty() || email.is_empty() || message.is_empty() {
        return Err("fill_fields");
    }
    if name.chars().count() > MAX_NAME_CHARS || email.chars().count() > MAX_EMAIL_CHARS {
        return Err("too_long");
    }
    if message.chars().count() > max_message_chars {
        return Err("message_too_long");
    }
    if !is_plausible_email(email) {
        return Err("invalid_email");
    }

    Ok(ValidatedMessage {
        name: name.to_string(),
        email: email.to_string(),
        message: message.to_string(),
    })
}

/// Cheap sanity check; the real verification is the reply the sender gets.
fn is_plausible_email(email: &str) -> bool {
    if email.chars().any(|c| c.is_whitespace() || c == ',') {
        return false;
    }
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.contains('@') {
        return false;
    }
    match domain.split_once('.') {
        Some((host, tld)) => !host.is_empty() && tld.len() >= 2 && !tld.starts_with('.'),
        None => false,
    }
}

async fn verify_recaptcha(
    state: &AppState,
    lang: Language,
    token: &str,
    ip: &str,
) -> Result<(), ContactError> {
    let config = &state.config.recaptcha;

    if config.disabled {
        warn!(
            "reCAPTCHA verification is disabled; accepting submission from {}",
            ip
        );
        return Ok(());
    }

    if config.secret.is_empty() {
        error!("Refusing contact submission: no reCAPTCHA secret configured");
        return Err(recaptcha_error(
            state,
            lang,
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    if token.trim().is_empty() {
        info!("Contact submission from {} carried no reCAPTCHA token", ip);
        return Err(recaptcha_error(state, lang, StatusCode::FORBIDDEN));
    }

    let response = state
        .http
        .post(RECAPTCHA_VERIFY_URL)
        .form(&[
            ("secret", config.secret.as_str()),
            ("response", token),
            ("remoteip", ip),
        ])
        .send()
        .await
        .map_err(|e| {
            error!("reCAPTCHA request failed: {}", e);
            recaptcha_error(state, lang, StatusCode::INTERNAL_SERVER_ERROR)
        })?;

    if !response.status().is_success() {
        error!("reCAPTCHA responded with status {}", response.status());
        return Err(recaptcha_error(
            state,
            lang,
            StatusCode::INTERNAL_SERVER_ERROR,
        ));
    }

    let verification: RecaptchaResponse = response.json().await.map_err(|e| {
        error!("Could not parse the reCAPTCHA response: {}", e);
        recaptcha_error(state, lang, StatusCode::INTERNAL_SERVER_ERROR)
    })?;

    match recaptcha_rejection(&verification, config.min_score, &config.expected_hostname) {
        None => Ok(()),
        // A missing score means the secret isn't a v3 key, not that a
        // visitor did anything wrong - that's a deploy-time misconfiguration,
        // not a rejected submission.
        Some("no_score") => {
            error!(
                "reCAPTCHA response for {} had no score - is RECAPTCHA_SECRET_KEY a v3 key?",
                ip
            );
            Err(recaptcha_error(
                state,
                lang,
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
        Some(reason) => {
            info!(
                "reCAPTCHA rejected {} ({}): success={}, score={:?}, action={:?}, hostname={:?}, errors={:?}",
                ip,
                reason,
                verification.success,
                verification.score,
                verification.action,
                verification.hostname,
                verification.error_codes
            );
            Err(recaptcha_error(state, lang, StatusCode::FORBIDDEN))
        }
    }
}

/// Reason to reject a parsed reCAPTCHA verification response, or `None` when
/// it should be accepted. Checking `action` and `hostname` (not just
/// `success`/`score`) stops a token solved for a different action, or on a
/// different site that reuses this site key, from passing anyway. See #55.
fn recaptcha_rejection(
    response: &RecaptchaResponse,
    min_score: f32,
    expected_hostname: &str,
) -> Option<&'static str> {
    if !response.success {
        return Some("not_success");
    }
    let Some(score) = response.score else {
        return Some("no_score");
    };
    if score < min_score {
        return Some("low_score");
    }
    if response.action.as_deref() != Some(RECAPTCHA_ACTION) {
        return Some("wrong_action");
    }
    if !response
        .hostname
        .as_deref()
        .is_some_and(|hostname| hostname.eq_ignore_ascii_case(expected_hostname))
    {
        return Some("wrong_hostname");
    }
    None
}

fn recaptcha_error(state: &AppState, lang: Language, status: StatusCode) -> ContactError {
    // The visitor sees the same message either way; the cause is in the logs.
    ContactError::new(state, lang, status, "recaptcha_failed")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form(name: &str, email: &str, message: &str) -> ContactForm {
        ContactForm {
            name: name.to_string(),
            email: email.to_string(),
            message: message.to_string(),
            g_recaptcha_response: "token".to_string(),
        }
    }

    #[test]
    fn accepts_a_complete_form() {
        let validated = validate(&form(" Lucas ", "lucas@example.com", " hola "), 5000)
            .expect("form should validate");
        assert_eq!(validated.name, "Lucas");
        assert_eq!(validated.message, "hola");
    }

    #[test]
    fn rejects_empty_fields() {
        assert_eq!(
            validate(&form("", "lucas@example.com", "hola"), 5000).err(),
            Some("fill_fields")
        );
        assert_eq!(
            validate(&form("Lucas", "lucas@example.com", "   "), 5000).err(),
            Some("fill_fields")
        );
    }

    #[test]
    fn rejects_oversized_input() {
        let long_name = "a".repeat(MAX_NAME_CHARS + 1);
        assert_eq!(
            validate(&form(&long_name, "lucas@example.com", "hola"), 5000).err(),
            Some("too_long")
        );
        let long_message = "a".repeat(11);
        assert_eq!(
            validate(&form("Lucas", "lucas@example.com", &long_message), 10).err(),
            Some("message_too_long")
        );
    }

    #[test]
    fn rejects_implausible_emails() {
        for email in [
            "lucas",
            "lucas@",
            "@example.com",
            "lucas@example",
            "a b@c.com",
        ] {
            assert_eq!(
                validate(&form("Lucas", email, "hola"), 5000).err(),
                Some("invalid_email"),
                "accepted {}",
                email
            );
        }
    }

    #[test]
    fn accepts_ordinary_emails() {
        for email in [
            "lucas@example.com",
            "lucas.rack+tag@sub.example.co.uk",
            "l@e.io",
        ] {
            assert!(is_plausible_email(email), "rejected {}", email);
        }
    }

    #[test]
    fn parses_recaptcha_responses_without_a_score() {
        let parsed: RecaptchaResponse =
            serde_json::from_str(r#"{"success": false, "error-codes": ["timeout-or-duplicate"]}"#)
                .expect("response should parse");
        assert!(!parsed.success);
        assert!(parsed.score.is_none());
        assert_eq!(parsed.error_codes, vec!["timeout-or-duplicate"]);
    }

    fn recaptcha_response(
        success: bool,
        score: Option<f32>,
        action: &str,
        hostname: &str,
    ) -> RecaptchaResponse {
        RecaptchaResponse {
            success,
            score,
            action: Some(action.to_string()),
            hostname: Some(hostname.to_string()),
            error_codes: Vec::new(),
        }
    }

    #[test]
    fn accepts_a_matching_response() {
        let response = recaptcha_response(true, Some(0.9), "submit", "mechardo3d.xyz");
        assert_eq!(recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"), None);
    }

    #[test]
    fn accepts_a_hostname_regardless_of_case() {
        let response = recaptcha_response(true, Some(0.9), "submit", "Mechardo3D.xyz");
        assert_eq!(recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"), None);
    }

    #[test]
    fn rejects_a_missing_score_as_a_configuration_error() {
        let response = recaptcha_response(true, None, "submit", "mechardo3d.xyz");
        assert_eq!(
            recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"),
            Some("no_score")
        );
    }

    #[test]
    fn rejects_a_score_below_the_threshold() {
        let response = recaptcha_response(true, Some(0.3), "submit", "mechardo3d.xyz");
        assert_eq!(
            recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"),
            Some("low_score")
        );
    }

    #[test]
    fn rejects_a_token_solved_for_another_action() {
        let response = recaptcha_response(true, Some(0.9), "login", "mechardo3d.xyz");
        assert_eq!(
            recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"),
            Some("wrong_action")
        );
    }

    #[test]
    fn rejects_a_token_solved_on_another_hostname() {
        let response = recaptcha_response(true, Some(0.9), "submit", "attacker.example");
        assert_eq!(
            recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"),
            Some("wrong_hostname")
        );
    }

    #[test]
    fn rejects_an_unsuccessful_response_before_anything_else() {
        let response = recaptcha_response(false, None, "submit", "mechardo3d.xyz");
        assert_eq!(
            recaptcha_rejection(&response, 0.5, "mechardo3d.xyz"),
            Some("not_success")
        );
    }
}
