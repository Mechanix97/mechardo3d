use axum::Extension;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tracing::warn;

use crate::config::AppConfig;
use crate::language::Language;
use crate::models::blog_post::BlogPost;
use crate::state::AppState;

/// Pages that exist in every language, with their crawl hints.
const STATIC_PAGES: [(&str, &str, &str); 7] = [
    ("", "weekly", "1.0"),
    ("me", "monthly", "0.9"),
    ("ds2000", "monthly", "0.9"),
    ("blog", "weekly", "0.8"),
    ("contact", "yearly", "0.7"),
    ("ds2000/terms-of-service", "yearly", "0.5"),
    ("ds2000/privacy-policy", "yearly", "0.5"),
];

/// `GET /sitemap.xml`
///
/// Generated from the routing table and the blog data, so blog posts are
/// included and entries cannot drift away from the site as it changes.
pub async fn sitemap(Extension(state): Extension<AppState>) -> Response {
    let posts = match state.blog.posts() {
        Ok(posts) => posts,
        Err(e) => {
            warn!("Sitemap generated without blog posts: {}", e);
            Default::default()
        }
    };

    let body = render_sitemap(&state.config, &posts);
    xml_response(body)
}

/// `GET /robots.txt`
pub async fn robots(Extension(state): Extension<AppState>) -> Response {
    let body = render_robots(&state.config);
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=3600"),
            ),
        ],
        body,
    )
        .into_response()
}

fn xml_response(body: String) -> Response {
    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/xml; charset=utf-8"),
            ),
            (
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=3600"),
            ),
        ],
        body,
    )
        .into_response()
}

pub fn render_robots(config: &AppConfig) -> String {
    format!(
        "# Robots.txt for Mechardo Labs\n\
         User-agent: *\n\
         Allow: /\n\n\
         Sitemap: {}\n",
        config.url("sitemap.xml")
    )
}

pub fn render_sitemap(config: &AppConfig, posts: &[BlogPost]) -> String {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\"\n\
         \x20       xmlns:xhtml=\"http://www.w3.org/1999/xhtml\">\n",
    );

    for (path, changefreq, priority) in STATIC_PAGES {
        for language in Language::ALL {
            xml.push_str(&url_entry(
                config, language, path, None, changefreq, priority,
            ));
        }
    }

    for post in posts {
        let path = format!("blog/{}", post.id);
        let lastmod = post.date.format("%Y-%m-%d").to_string();
        for language in Language::ALL {
            xml.push_str(&url_entry(
                config,
                language,
                &path,
                Some(&lastmod),
                "monthly",
                "0.6",
            ));
        }
    }

    xml.push_str("</urlset>\n");
    xml
}

fn url_entry(
    config: &AppConfig,
    language: Language,
    path: &str,
    lastmod: Option<&str>,
    changefreq: &str,
    priority: &str,
) -> String {
    let mut entry = String::from("    <url>\n");
    entry.push_str(&format!(
        "        <loc>{}</loc>\n",
        escape_xml(&page_url(config, language, path))
    ));

    for alternate in Language::ALL {
        entry.push_str(&format!(
            "        <xhtml:link rel=\"alternate\" hreflang=\"{}\" href=\"{}\"/>\n",
            alternate.as_str(),
            escape_xml(&page_url(config, alternate, path))
        ));
    }

    if let Some(lastmod) = lastmod {
        entry.push_str(&format!("        <lastmod>{}</lastmod>\n", lastmod));
    }
    entry.push_str(&format!(
        "        <changefreq>{}</changefreq>\n",
        changefreq
    ));
    entry.push_str(&format!("        <priority>{}</priority>\n", priority));
    entry.push_str("    </url>\n");
    entry
}

fn page_url(config: &AppConfig, language: Language, path: &str) -> String {
    if path.is_empty() {
        config.url(language.as_str())
    } else {
        config.url(&format!("{}/{}", language.as_str(), path))
    }
}

fn escape_xml(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::blog_data::BlogStore;
    use std::path::Path;

    fn config() -> AppConfig {
        AppConfig::from_env()
    }

    fn posts() -> Vec<BlogPost> {
        let store = BlogStore::new(Path::new("data"), Path::new("templates"));
        store.posts().expect("posts").as_ref().clone()
    }

    #[test]
    fn lists_every_page_in_every_language() {
        let config = config();
        let xml = render_sitemap(&config, &posts());
        for language in Language::ALL {
            assert!(xml.contains(&format!(
                "<loc>{}/{}</loc>",
                config.base_url,
                language.as_str()
            )));
            assert!(xml.contains(&format!(
                "<loc>{}/{}/ds2000/privacy-policy</loc>",
                config.base_url,
                language.as_str()
            )));
        }
    }

    #[test]
    fn includes_blog_posts_with_their_date() {
        let config = config();
        let posts = posts();
        let xml = render_sitemap(&config, &posts);
        let post = posts.first().expect("at least one post");
        assert!(xml.contains(&format!(
            "<loc>{}/es/blog/{}</loc>",
            config.base_url, post.id
        )));
        assert!(xml.contains(&format!(
            "<lastmod>{}</lastmod>",
            post.date.format("%Y-%m-%d")
        )));
    }

    #[test]
    fn opens_and_closes_the_urlset() {
        let xml = render_sitemap(&config(), &[]);
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.trim_end().ends_with("</urlset>"));
        assert_eq!(
            xml.matches("<url>").count(),
            STATIC_PAGES.len() * Language::ALL.len()
        );
    }

    #[test]
    fn robots_points_at_the_generated_sitemap() {
        let config = config();
        let robots = render_robots(&config);
        assert!(robots.contains(&format!("Sitemap: {}/sitemap.xml", config.base_url)));
        assert!(!robots.contains("Disallow: /static/"));
    }

    #[test]
    fn escapes_xml_entities() {
        assert_eq!(escape_xml("a&b<c>"), "a&amp;b&lt;c&gt;");
    }
}
