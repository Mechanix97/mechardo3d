use serde_json::{Value, json};

use crate::config::AppConfig;
use crate::language::Language;
use crate::models::blog_post::BlogPostView;

const AUTHOR: &str = "Lucas Rack";
const GITHUB_URL: &str = "https://github.com/Mechanix97";
const LINKEDIN_URL: &str = "https://linkedin.com/in/lucasalexisrack";

/// JSON-LD for a Person (used on `/me`).
///
/// The job title, employer and university mirror what the page itself renders,
/// so a search result cannot describe a role the portfolio no longer claims.
pub fn person_schema(config: &AppConfig, lang: Language) -> Value {
    let job_title = match lang {
        Language::Spanish => "Ingeniero de Software",
        Language::English => "Software Engineer",
    };
    let description = match lang {
        Language::Spanish => {
            "Ingeniero de software especializado en aplicaciones de AI/LLM, \
             sistemas distribuidos en Rust y electrónica"
        }
        Language::English => {
            "Software engineer specialized in AI/LLM applications, distributed \
             systems in Rust, and electronics"
        }
    };
    let alumni_of = match lang {
        Language::Spanish => "Universidad de Buenos Aires",
        Language::English => "University of Buenos Aires",
    };

    json!({
        "@context": "https://schema.org",
        "@type": "Person",
        "name": AUTHOR,
        "url": config.url(&format!("{}/me", lang.as_str())),
        "sameAs": [GITHUB_URL, LINKEDIN_URL],
        "jobTitle": job_title,
        "worksFor": {
            "@type": "Organization",
            "name": "Lenovo",
            "url": "https://www.lenovo.com/"
        },
        "alumniOf": {
            "@type": "CollegeOrUniversity",
            "name": alumni_of,
            "url": "https://www.fi.uba.ar/"
        },
        "address": {
            "@type": "PostalAddress",
            "addressLocality": "Buenos Aires",
            "addressCountry": "AR"
        },
        "knowsAbout": [
            "Artificial Intelligence",
            "Large Language Models",
            "Retrieval-Augmented Generation",
            "Python",
            "Rust",
            "Distributed Systems",
            "Ethereum",
            "Smart Contracts",
            "Embedded Systems",
            "Electronics"
        ],
        "knowsLanguage": ["es", "en"],
        "email": "lucas_rack@live.com.ar",
        "description": description,
        "inLanguage": lang.locale()
    })
}

/// JSON-LD for the Organization (used on the home page).
pub fn organization_schema(config: &AppConfig, lang: Language) -> Value {
    let description = match lang {
        Language::Spanish => "Laboratorio de Electrónica y Software Engineering",
        Language::English => "Electronics and Software Engineering Lab",
    };

    json!({
        "@context": "https://schema.org",
        "@type": "Organization",
        "name": "Mechardo Labs",
        "url": config.url(""),
        "logo": config.url("static/images/Mechardo-labs.png"),
        "description": description,
        "sameAs": [GITHUB_URL, LINKEDIN_URL],
        "inLanguage": lang.locale()
    })
}

/// JSON-LD for the DS2000 product page.
pub fn product_schema(config: &AppConfig, lang: Language) -> Value {
    let description = match lang {
        Language::Spanish => "Botonera USB para Discord con 3 botones programables y 2 LEDs RGB",
        Language::English => "USB Discord Button Box with 3 programmable buttons and 2 RGB LEDs",
    };
    let buttons = match lang {
        Language::Spanish => {
            "3 botones programables para mutear/desmutear, ensordecer/desensordecer, desconectar"
        }
        Language::English => "3 programmable buttons for mute/unmute, deafen/undeafen, disconnect",
    };
    let leds = match lang {
        Language::Spanish => "2 LEDs RGB configurables",
        Language::English => "2 configurable RGB LEDs",
    };
    let platforms = match lang {
        Language::Spanish => "Compatible con Windows, macOS y Linux",
        Language::English => "Compatible with Windows, macOS and Linux",
    };

    json!({
        "@context": "https://schema.org",
        "@type": "Product",
        "name": "DS2000",
        "description": description,
        "url": config.url(&format!("{}/ds2000", lang.as_str())),
        "image": config.url("static/images/og-image.png"),
        "brand": {
            "@type": "Brand",
            "name": "Mechardo Labs"
        },
        "features": [buttons, leds, "USB Plug-and-Play", platforms],
        "inLanguage": lang.locale()
    })
}

/// JSON-LD for a single blog post.
pub fn blog_post_schema(
    config: &AppConfig,
    lang: Language,
    post: &BlogPostView,
    description: &str,
    url_path: &str,
) -> Value {
    let mut schema = json!({
        "@context": "https://schema.org",
        "@type": "BlogPosting",
        "headline": post.title,
        "description": description,
        "datePublished": post.date.format("%Y-%m-%d").to_string(),
        "dateModified": post.date.format("%Y-%m-%d").to_string(),
        "mainEntityOfPage": config.url(url_path),
        "author": {
            "@type": "Person",
            "name": AUTHOR,
            "url": config.url("")
        },
        "publisher": {
            "@type": "Organization",
            "name": "Mechardo Labs",
            "url": config.url("")
        },
        "inLanguage": lang.locale()
    });

    if let Some(image) = post.thumbnail.as_deref()
        && let Some(object) = schema.as_object_mut()
    {
        object.insert(
            "image".to_string(),
            Value::String(config.url(image.trim_start_matches('/'))),
        );
    }

    schema
}

/// JSON-LD for a generic page.
pub fn webpage_schema(
    config: &AppConfig,
    lang: Language,
    title: &str,
    description: &str,
    page_type: &str,
    url_path: &str,
) -> Value {
    json!({
        "@context": "https://schema.org",
        "@type": page_type,
        "name": title,
        "description": description,
        "url": config.url(url_path),
        "inLanguage": lang.locale()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AppConfig {
        AppConfig::from_env()
    }

    #[test]
    fn uses_the_configured_base_url() {
        let config = config();
        let schema = organization_schema(&config, Language::Spanish);
        assert_eq!(schema["url"], json!(format!("{}/", config.base_url)));
    }

    #[test]
    fn describes_pages_with_their_own_url() {
        let config = config();
        let schema = webpage_schema(
            &config,
            Language::English,
            "Blog",
            "Posts",
            "Blog",
            "en/blog",
        );
        assert_eq!(schema["url"], json!(config.url("en/blog")));
        assert_eq!(schema["inLanguage"], json!("en-US"));
    }

    #[test]
    fn adds_an_image_to_blog_posts_when_available() {
        let config = config();
        let mut post = post();
        let schema = blog_post_schema(
            &config,
            Language::Spanish,
            &post,
            "Description",
            "es/blog/4",
        );
        assert_eq!(
            schema["image"],
            json!(config.url("static/images/thumb.png"))
        );
        assert_eq!(schema["headline"], json!("Hola"));
        assert_eq!(schema["datePublished"], json!("2025-08-18"));

        post.thumbnail = None;
        let without_image = blog_post_schema(
            &config,
            Language::Spanish,
            &post,
            "Description",
            "es/blog/4",
        );
        assert!(without_image.get("image").is_none());
    }

    fn post() -> BlogPostView {
        let post: crate::models::blog_post::BlogPost = serde_json::from_str(
            r#"{
                "id": "4",
                "title": { "es": "Hola", "en": "Hello" },
                "summary": { "es": "Resumen", "en": "Summary" },
                "thumbnail": "/static/images/thumb.png",
                "date": "18-08-2025"
            }"#,
        )
        .expect("post should deserialize");
        BlogPostView::new(&post, Language::Spanish)
    }
}
