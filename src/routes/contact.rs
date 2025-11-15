use crate::responses::HtmlWithLang;
use crate::translations::get_translations_for_lang;
use axum::{
    Extension,
    extract::{ConnectInfo, Form},
    http::StatusCode,
    response::{Json, Redirect},
};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path as StdPath;
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use tracing::{error, info};

#[derive(Deserialize, Serialize)]
pub struct ContactForm {
    name: String,
    email: String,
    message: String,
    #[serde(rename = "g-recaptcha-response")]
    g_recaptcha_response: String,
}

#[derive(Deserialize)]
pub struct RecaptchaResponse {
    success: bool,
    score: f32,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}

pub type RateLimitState = Arc<RwLock<HashMap<String, chrono::DateTime<Utc>>>>;

use crate::language::Language;
use axum::extract::Path as AxumPath;

// Handler for GET /contact
pub async fn contact(
    AxumPath(lang): AxumPath<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let title = t
        .get("page_titles")
        .and_then(|pt| pt.get("contact"))
        .and_then(|v| v.as_str())
        .unwrap_or("Contact");

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", title);
    context.insert("t", &t);
    context.insert(
        "recaptcha_site_key",
        "6LfuI5YrAAAAAOEUv-Xp1Ewo4dhr1TgCrCG_aqa8",
    );
    let rendered = tera
        .render("contact.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}

// Handler for GET /contact_success
pub async fn contact_success(
    AxumPath(lang): AxumPath<String>,
    Extension(tera): Extension<Tera>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
) -> HtmlWithLang {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    let t = get_translations_for_lang(&translations, language.as_str());

    let title = t
        .get("page_titles")
        .and_then(|pt| pt.get("message_sent"))
        .and_then(|v| v.as_str())
        .unwrap_or("Message Sent");

    let mut context = Context::new();
    context.insert("lang", language.as_str());
    context.insert("title", title);
    context.insert("t", &t);
    let rendered = tera
        .render("contact_success.html", &context)
        .expect("Error rendering template");
    HtmlWithLang::new(rendered, language)
}

/// Helper function to get error message translation
fn get_error_msg(translations: &HashMap<String, Value>, lang: &str, key: &str) -> String {
    let t = get_translations_for_lang(translations, lang);
    t.get("errors")
        .and_then(|e| e.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or(key)
        .to_string()
}

// Handler for POST /contact
pub async fn contact_submit(
    AxumPath(lang): AxumPath<String>,
    Extension(rate_limit): Extension<RateLimitState>,
    Extension(translations): Extension<Arc<HashMap<String, Value>>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(form): Form<ContactForm>,
) -> Result<Redirect, (StatusCode, Json<ErrorResponse>)> {
    let language = Language::from_str(&lang).unwrap_or_else(Language::default);
    info!("Processing contact form submission from IP: {}", addr.ip());

    // Rate limiting by IP (1 message every 5 minutes)
    let ip: String = addr.ip().to_string();
    let mut rate_limit = rate_limit.write().await;
    let now = Utc::now();
    if let Some(last_submission) = rate_limit.get(&ip) {
        let elapsed = now.signed_duration_since(*last_submission).num_seconds();
        if elapsed < 300 {
            info!("Rate limit exceeded for IP: {}", ip);
            return Err((
                StatusCode::TOO_MANY_REQUESTS,
                Json(ErrorResponse {
                    error: get_error_msg(&translations, language.as_str(), "rate_limit"),
                }),
            ));
        }
    }

    // Validate form fields
    if form.name.trim().is_empty() || form.email.trim().is_empty() || form.message.trim().is_empty()
    {
        info!("Invalid form data from IP: {}", ip);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: get_error_msg(&translations, language.as_str(), "fill_fields"),
            }),
        ));
    }

    // Read reCAPTCHA secret key from secrets/recaptcha.env
    let recaptcha_secret = match fs::read_to_string("secrets/recaptcha.env") {
        Ok(content) => content
            .lines()
            .find_map(|line| {
                let parts: Vec<&str> = line.splitn(2, '=').collect();
                if parts.len() == 2 && parts[0].trim() == "RECAPTCHA_SECRET_KEY" {
                    Some(parts[1].trim().to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                error!("RECAPTCHA_SECRET_KEY not found in secrets/recaptcha.env");
                String::from("") // Fallback for local testing
            }),
        Err(e) => {
            error!("Failed to read secrets/recaptcha.env: {}", e);
            String::from("") // Fallback for local testing
        }
    };

    // Verify reCAPTCHA
    info!("Verifying reCAPTCHA for IP: {}", ip);
    let client = Client::new();
    let recaptcha_response = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&[
            ("secret", &recaptcha_secret),
            ("response", &form.g_recaptcha_response),
            ("remoteip", &ip),
        ])
        .send()
        .await;

    match recaptcha_response {
        Ok(response) => {
            info!("reCAPTCHA response status: {}", response.status());
            if response.status().is_success() {
                match response.text().await {
                    Ok(text) => {
                        info!("reCAPTCHA response body: {}", text);
                        match serde_json::from_str::<RecaptchaResponse>(&text) {
                            Ok(recaptcha_data) => {
                                if !recaptcha_data.success || recaptcha_data.score < 0.6 {
                                    info!(
                                        "reCAPTCHA verification failed for IP: {}. Success: {}, Score: {}",
                                        ip, recaptcha_data.success, recaptcha_data.score
                                    );
                                    return Err((
                                        StatusCode::FORBIDDEN,
                                        Json(ErrorResponse {
                                            error: get_error_msg(
                                                &translations,
                                                language.as_str(),
                                                "recaptcha_failed",
                                            ),
                                        }),
                                    ));
                                }
                            }
                            Err(e) => {
                                error!(
                                    "Failed to parse reCAPTCHA response for IP: {}: {}. Response body: {}",
                                    ip, e, text
                                );
                                return Err((
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    Json(ErrorResponse {
                                        error: get_error_msg(
                                            &translations,
                                            language.as_str(),
                                            "recaptcha_failed",
                                        ),
                                    }),
                                ));
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to read reCAPTCHA response body for IP: {}: {}",
                            ip, e
                        );
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: get_error_msg(
                                    &translations,
                                    language.as_str(),
                                    "recaptcha_failed",
                                ),
                            }),
                        ));
                    }
                }
            } else {
                let status = response.status();
                let text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "No response body".to_string());
                error!(
                    "reCAPTCHA request failed for IP: {}. Status: {}. Response: {}",
                    ip, status, text
                );
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: get_error_msg(&translations, language.as_str(), "recaptcha_failed"),
                    }),
                ));
            }
        }
        Err(e) => {
            error!("reCAPTCHA request error for IP: {}: {}", ip, e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: get_error_msg(&translations, language.as_str(), "recaptcha_failed"),
                }),
            ));
        }
    }

    // Save message to messages.json
    info!("Saving message to messages.json for IP: {}", ip);
    let message = json!({
        "name": form.name,
        "email": form.email,
        "message": form.message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let messages_file = "data/messages.json";
    let mut messages: Vec<serde_json::Value> = if StdPath::new(messages_file).exists() {
        match fs::read_to_string(messages_file) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(messages) => messages,
                Err(e) => {
                    error!("Failed to parse messages.json: {}", e);
                    return Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: get_error_msg(&translations, language.as_str(), "save_failed"),
                        }),
                    ));
                }
            },
            Err(e) => {
                error!("Failed to read messages.json: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: get_error_msg(&translations, language.as_str(), "save_failed"),
                    }),
                ));
            }
        }
    } else {
        vec![]
    };

    messages.push(message);
    match serde_json::to_string_pretty(&messages) {
        Ok(data) => match fs::write(messages_file, &data) {
            Ok(_) => info!("Message saved to messages.json for IP: {}", ip),
            Err(e) => {
                error!("Failed to write to messages.json: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: get_error_msg(&translations, language.as_str(), "save_failed"),
                    }),
                ));
            }
        },
        Err(e) => {
            error!("Failed to serialize messages to JSON: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: get_error_msg(&translations, language.as_str(), "save_failed"),
                }),
            ));
        }
    }

    // Update rate limit for IP
    info!("Updating rate limit for IP: {}", ip);
    rate_limit.insert(ip, now);

    // Redirect to /contact_success
    Ok(Redirect::to(&format!(
        "/{}/contact_success",
        language.as_str()
    )))
}
