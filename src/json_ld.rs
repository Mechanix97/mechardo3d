use serde_json::{Value, json};

/// Generate JSON-LD schema for a Person (used on /me page)
pub fn person_schema(lang: &str) -> Value {
    let locale = if lang == "es" { "es-ES" } else { "en-US" };

    json!({
        "@context": "https://schema.org",
        "@type": "Person",
        "name": "Lucas Rack",
        "url": "https://mechardo3d.xyz",
        "sameAs": [
            "https://github.com/Mechanix97",
            "https://linkedin.com/in/lucasalexisrack"
        ],
        "jobTitle": if lang == "es" {
            "Ingeniero de Software"
        } else {
            "Software Engineer"
        },
        "knowsAbout": [
            "Rust",
            "Blockchain",
            "Ethereum",
            "Smart Contracts",
            "P2P Protocols",
            "Python",
            "Docker",
            "Electronics"
        ],
        "email": "lucas_rack@live.com.ar",
        "description": if lang == "es" {
            "Ingeniero de Software especializado en Rust, Blockchain y Electrónica"
        } else {
            "Software Engineer specialized in Rust, Blockchain, and Electronics"
        },
        "inLanguage": locale
    })
}

/// Generate JSON-LD schema for Organization (used on home page)
pub fn organization_schema(lang: &str) -> Value {
    let locale = if lang == "es" { "es-ES" } else { "en-US" };

    json!({
        "@context": "https://schema.org",
        "@type": "Organization",
        "name": "Mechardo Labs",
        "url": "https://mechardo3d.xyz",
        "description": if lang == "es" {
            "Laboratorio de Electrónica y Software Engineering"
        } else {
            "Electronics and Software Engineering Lab"
        },
        "sameAs": [
            "https://github.com/Mechanix97",
            "https://linkedin.com/in/lucasalexisrack"
        ],
        "inLanguage": locale
    })
}

/// Generate JSON-LD schema for a Product (used on DS2000 page)
pub fn product_schema(lang: &str) -> Value {
    let locale = if lang == "es" { "es-ES" } else { "en-US" };

    json!({
        "@context": "https://schema.org",
        "@type": "Product",
        "name": "DS2000",
        "description": if lang == "es" {
            "Botonera USB para Discord con 3 botones programables y 2 LEDs RGB"
        } else {
            "USB Discord Button Box with 3 programmable buttons and 2 RGB LEDs"
        },
        "url": format!("https://mechardo3d.xyz/{}/ds2000", lang),
        "image": "https://mechardo3d.xyz/static/images/og-image.png",
        "brand": {
            "@type": "Brand",
            "name": "Mechardo Labs"
        },
        "features": [
            if lang == "es" {
                "3 botones programables para mutear/desmutear, ensordecer/desensordecer, desconectar"
            } else {
                "3 programmable buttons for mute/unmute, deafen/undeafen, disconnect"
            },
            if lang == "es" {
                "2 LEDs RGB configurables"
            } else {
                "2 configurable RGB LEDs"
            },
            "USB Plug-and-Play",
            "Compatible con Windows, macOS, Linux"
        ],
        "inLanguage": locale
    })
}

/// Generate JSON-LD schema for a BlogPosting
pub fn blog_post_schema(
    title: &str,
    description: &str,
    date: &str,
    author: &str,
    lang: &str,
) -> Value {
    let locale = if lang == "es" { "es-ES" } else { "en-US" };

    json!({
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        "headline": title,
        "description": description,
        "datePublished": date,
        "dateModified": date,
        "author": {
            "@type": "Person",
            "name": author,
            "url": "https://mechardo3d.xyz"
        },
        "publisher": {
            "@type": "Organization",
            "name": "Mechardo Labs",
            "url": "https://mechardo3d.xyz"
        },
        "inLanguage": locale
    })
}

/// Generate JSON-LD schema for a WebPage
pub fn webpage_schema(title: &str, description: &str, page_type: &str, lang: &str) -> Value {
    let locale = if lang == "es" { "es-ES" } else { "en-US" };

    json!({
        "@context": "https://schema.org",
        "@type": page_type,
        "name": title,
        "description": description,
        "url": format!("https://mechardo3d.xyz/{}", lang),
        "inLanguage": locale
    })
}
