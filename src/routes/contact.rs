use axum::{Extension, extract::Form, response::Html};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

#[derive(Deserialize, Serialize)]
pub struct ContactForm {
    name: String,
    email: String,
    message: String,
}

pub async fn contact(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("title", "Contacto");
    context.insert("content", "Ponte en contacto con nosotros.");

    let rendered = tera
        .render("contact.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}

// Handler para POST /contact
pub async fn contact_submit(
    Extension(tera): Extension<Tera>,
    Form(form): Form<ContactForm>,
) -> Html<String> {
    if form.name.trim().is_empty() || form.email.trim().is_empty() || form.message.trim().is_empty()
    {
        let mut context = Context::new();
        context.insert("title", "Error");
        context.insert(
            "content",
            "Por favor, completá todos los campos del formulario.",
        );
        let rendered = tera
            .render("contact.html", &context)
            .expect("Error rendering template");
        return Html(rendered);
    }

    // Guardar en messages.json
    let message = serde_json::json!({
        "name": form.name,
        "email": form.email,
        "message": form.message,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    });

    let messages_file = "data/messages.json";
    let mut messages: Vec<serde_json::Value> = if Path::new(messages_file).exists() {
        let data = fs::read_to_string(messages_file).expect("Error reading messages.json");
        serde_json::from_str(&data).unwrap_or_else(|_| vec![])
    } else {
        vec![]
    };

    messages.push(message);
    fs::write(
        messages_file,
        serde_json::to_string_pretty(&messages).expect("Error serializing messages"),
    )
    .expect("Error writing to messages.json");

    let mut context = Context::new();
    context.insert("title", "Contacto");
    let rendered = tera
        .render("contact_success.html", &context)
        .expect("Error rendering template");
    Html(rendered)
}
