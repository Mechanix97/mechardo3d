use axum::{
    Extension,
    extract::{ConnectInfo, Form},
    http::StatusCode,
    response::{Html, Redirect},
};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::sync::RwLock;
use tracing::{error, info};

const RECAPTCHA_SITE_KEY: &str = "6LfuI5YrAAAAAOEUv-Xp1Ewo4dhr1TgCrCG_aqa8";

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

pub type RateLimitState = Arc<RwLock<HashMap<String, chrono::DateTime<Utc>>>>;

pub async fn contact(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Contacto");
    context.insert("content", "Ponte en contacto con nosotros.");
    context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
    let rendered = tera
        .render("contact.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn contact_success(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Mensaje enviado");
    let rendered = tera
        .render("contact_success.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

pub async fn contact_submit(
    Extension(tera): Extension<Tera>,
    Extension(rate_limit): Extension<RateLimitState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Form(form): Form<ContactForm>,
) -> Result<Redirect, (StatusCode, Html<String>)> {
    info!("Processing contact form submission from IP: {}", addr.ip());

    // Límite por IP (1 mensaje cada 5 minutos)
    let ip: String = addr.ip().to_string();
    let mut rate_limit = rate_limit.write().await;
    let now = Utc::now();
    if let Some(last_submission) = rate_limit.get(&ip) {
        let elapsed = now.signed_duration_since(*last_submission).num_seconds();
        if elapsed < 300 {
            info!("Rate limit exceeded for IP: {}", ip);
            let mut context = Context::new();
            context.insert("title", "Error");
            context.insert(
                "content",
                "Por favor, esperá 5 minutos antes de enviar otro mensaje.",
            );
            context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
            let rendered = tera
                .render("contact.html", &context)
                .expect("Error rendering template");
            return Err((StatusCode::TOO_MANY_REQUESTS, Html(rendered)));
        }
    }

    // Validación de campos
    if form.name.trim().is_empty() || form.email.trim().is_empty() || form.message.trim().is_empty()
    {
        info!("Invalid form data from IP: {}", ip);
        let mut context = Context::new();
        context.insert("title", "Error");
        context.insert(
            "content",
            "Por favor, completá todos los campos del formulario.",
        );
        context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
        let rendered = tera
            .render("contact.html", &context)
            .expect("Error rendering template");
        return Err((StatusCode::BAD_REQUEST, Html(rendered)));
    }

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
                String::from("")
            }),
        Err(e) => {
            error!("Failed to read secrets/recaptcha.env: {}", e);
            String::from("")
        }
    };

    // Verificar reCAPTCHA
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
        Ok(response) if response.status().is_success() => {
            match response.json::<RecaptchaResponse>().await {
                Ok(recaptcha_data) => {
                    if !recaptcha_data.success || recaptcha_data.score < 0.6 {
                        info!(
                            "reCAPTCHA verification failed for IP: {}. Success: {}, Score: {}",
                            ip, recaptcha_data.success, recaptcha_data.score
                        );
                        let mut context = Context::new();
                        context.insert("title", "Error");
                        context.insert(
                            "content",
                            "No se pudo verificar que no sos un bot. Intentá de nuevo.",
                        );
                        context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
                        let rendered = tera
                            .render("contact.html", &context)
                            .expect("Error rendering template");
                        return Err((StatusCode::FORBIDDEN, Html(rendered)));
                    }
                }
                Err(e) => {
                    error!("Failed to parse reCAPTCHA response for IP: {}: {}", ip, e);
                    let mut context = Context::new();
                    context.insert("title", "Error");
                    context.insert("content", "Error al verificar reCAPTCHA. Intentá de nuevo.");
                    context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
                    let rendered: String = tera
                        .render("contact.html", &context)
                        .expect("Error rendering template");
                    return Err((StatusCode::INTERNAL_SERVER_ERROR, Html(rendered)));
                }
            }
        }
        Ok(response) => {
            error!(
                "reCAPTCHA request failed for IP: {}. Status: {}",
                ip,
                response.status()
            );
            let mut context = Context::new();
            context.insert("title", "Error");
            context.insert("content", "Error al verificar reCAPTCHA. Intentá de nuevo.");
            context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
            let rendered = tera
                .render("contact.html", &context)
                .expect("Error rendering template");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Html(rendered)));
        }
        Err(e) => {
            error!("reCAPTCHA request error for IP: {}: {}", ip, e);
            let mut context = Context::new();
            context.insert("title", "Error");
            context.insert("content", "Error al verificar reCAPTCHA. Intentá de nuevo.");
            context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
            let rendered = tera
                .render("contact.html", &context)
                .expect("Error rendering template");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Html(rendered)));
        }
    }

    // Guardar en messages.json
    info!("Saving message to messages.json for IP: {}", ip);
    let message = serde_json::json!({
        "name": form.name,
        "email": form.email,
        "message": form.message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let messages_file = "data/messages.json";
    let mut messages: Vec<serde_json::Value> = if Path::new(messages_file).exists() {
        match fs::read_to_string(messages_file) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(messages) => messages,
                Err(e) => {
                    error!("Failed to parse messages.json: {}", e);
                    vec![]
                }
            },
            Err(e) => {
                error!("Failed to read messages.json: {}", e);
                vec![]
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
                let mut context = Context::new();
                context.insert("title", "Error");
                context.insert("content", "Error al guardar el mensaje. Intentá de nuevo.");
                context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
                let rendered = tera
                    .render("contact.html", &context)
                    .expect("Error rendering template");
                return Err((StatusCode::INTERNAL_SERVER_ERROR, Html(rendered)));
            }
        },
        Err(e) => {
            error!("Failed to serialize messages to JSON: {}", e);
            let mut context = Context::new();
            context.insert("title", "Error");
            context.insert("content", "Error al guardar el mensaje. Intentá de nuevo.");
            context.insert("recaptcha_site_key", RECAPTCHA_SITE_KEY);
            let rendered = tera
                .render("contact.html", &context)
                .expect("Error rendering template");
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Html(rendered)));
        }
    }

    // Actualizar límite por IP
    info!("Updating rate limit for IP: {}", ip);
    rate_limit.insert(ip.clone(), now);

    // Redirigir a /contact_success
    info!("Redirecting to /contact_success for IP: {}", ip);
    Ok(Redirect::to("/contact_success"))
}
