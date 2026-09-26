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

/// Real product renders, used instead of the site's generic social card so
/// the DS2000 page's structured data shows an actual photo of the device.
const DS2000_IMAGES: [&str; 3] = [
    "static/images/DS2000/renders/frente.webp",
    "static/images/DS2000/renders/trasero.webp",
    "static/images/DS2000/renders/superior.webp",
];

/// JSON-LD for the DS2000 project page.
///
/// It's a `WebPage`, not a `Product`: the DS2000 isn't for sale (it may end up
/// open source instead), so it has no price or availability to declare, and
/// marking it up as `Product` without an `offers`/`review`/`aggregateRating`
/// is exactly what Search Console flags as invalid. See #67.
pub fn ds2000_schema(config: &AppConfig, lang: Language, title: &str, description: &str) -> Value {
    let images: Vec<String> = DS2000_IMAGES.iter().map(|path| config.url(path)).collect();

    json!({
        "@context": "https://schema.org",
        "@type": "WebPage",
        "name": title,
        "description": description,
        "url": config.url(&format!("{}/ds2000", lang.as_str())),
        "image": images,
        "inLanguage": lang.locale()
    })
}

/// JSON-LD `BreadcrumbList`. `crumbs` are `(name, path)` pairs, root first;
/// `path` is the URL path after the language prefix (empty for the home
/// page).
pub fn breadcrumbs(config: &AppConfig, lang: Language, crumbs: &[(&str, &str)]) -> Value {
    let items: Vec<Value> = crumbs
        .iter()
        .enumerate()
        .map(|(index, (name, path))| {
            json!({
                "@type": "ListItem",
                "position": index + 1,
                "name": name,
                "item": crumb_url(config, lang, path)
            })
        })
        .collect();

    json!({
        "@context": "https://schema.org",
        "@type": "BreadcrumbList",
        "itemListElement": items
    })
}

fn crumb_url(config: &AppConfig, lang: Language, path: &str) -> String {
    if path.is_empty() {
        config.url(lang.as_str())
    } else {
        config.url(&format!("{}/{}", lang.as_str(), path))
    }
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

    #[test]
    fn ds2000_is_a_webpage_not_a_product() {
        // It isn't for sale (it may end up open source instead), so it has no
        // offers/review/aggregateRating - marking it up as `Product` without
        // those is exactly what Search Console flags as invalid. See #67.
        let config = config();
        let schema = ds2000_schema(&config, Language::Spanish, "DS2000", "Description");
        assert_eq!(schema["@type"], json!("WebPage"));
        assert_eq!(schema.get("offers"), None);
        assert_eq!(
            schema["image"],
            json!([
                config.url("static/images/DS2000/renders/frente.webp"),
                config.url("static/images/DS2000/renders/trasero.webp"),
                config.url("static/images/DS2000/renders/superior.webp"),
            ])
        );
    }

    #[test]
    fn breadcrumbs_number_items_from_one_and_resolve_their_urls() {
        let config = config();
        let schema = breadcrumbs(
            &config,
            Language::Spanish,
            &[
                ("Inicio", ""),
                ("DS2000", "ds2000"),
                ("Privacidad", "ds2000/privacy-policy"),
            ],
        );

        assert_eq!(schema["@type"], json!("BreadcrumbList"));
        let items = schema["itemListElement"].as_array().expect("array");
        assert_eq!(items.len(), 3);
        assert_eq!(items[0]["position"], json!(1));
        assert_eq!(items[0]["name"], json!("Inicio"));
        assert_eq!(items[0]["item"], json!(config.url("es")));
        assert_eq!(items[2]["position"], json!(3));
        assert_eq!(
            items[2]["item"],
            json!(config.url("es/ds2000/privacy-policy"))
        );
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
