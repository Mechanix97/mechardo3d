use axum::Extension;
use axum::response::Response;
use rand::rng;
use rand::seq::IndexedRandom;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;

use crate::extract::Lang;
use crate::json_ld;
use crate::language::Language;
use crate::pages::{self, page_meta};
use crate::state::AppState;

/// Breadcrumb label for the DS2000 pages, matching the nav link text in
/// `base.html` rather than `page_titles.ds2000` (which carries the longer,
/// SEO-oriented `<title>` copy).
const DS2000_CRUMB: &str = "DS2000";

/// Link preview (Open Graph / Twitter card) for the product page: the cream
/// front render at 1200 x 630, in PNG because not every platform takes WebP.
const OG_IMAGE: &str = "static/images/og-ds2000.png";

/// The colourways the DS2000 is rendered in; renders are named
/// `static/images/DS2000/renders/<colourway>-<view>.webp`.
pub const COLOURWAYS: [&str; 4] = ["negro", "original", "snes", "rosa"];

/// A colourway picked at random for each page view, so the home page and the
/// product page show a different DS2000 on every visit. It is chosen on the
/// server, so the first image the browser loads is already the right one.
pub fn random_colourway() -> &'static str {
    COLOURWAYS.choose(&mut rng()).copied().unwrap_or("original")
}

pub async fn ds2000(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let meta = page_meta(&state, lang, "ds2000").path("ds2000");

    let schema = json_ld::ds2000_schema(&state.config, lang, &meta.title, &meta.description);
    let breadcrumbs = json_ld::breadcrumbs(
        &state.config,
        lang,
        &[(home_crumb(&state, lang), ""), (DS2000_CRUMB, "ds2000")],
    );
    let meta = meta
        .og_type("product")
        .og_image(OG_IMAGE)
        .schema(json!([schema, breadcrumbs]));

    let mut context = pages::base_context(&state, lang, &meta);
    context.insert("colourway", random_colourway());
    context.insert("tabs", &tabs());
    context.insert("active_tab", "product");
    pages::render(&state, "ds2000/ds2000.html", &context, lang)
}

/// The tabs of the DS2000 section, in order: tab key (also the translation key
/// under `ds2000_guide.tabs`) and path below the language prefix.
pub const TABS: [(&str, &str); 5] = [
    ("product", "ds2000"),
    ("download", "ds2000/download"),
    ("install", "ds2000/install"),
    ("discord", "ds2000/discord-setup"),
    ("source", "ds2000/source"),
];

/// Screenshots for the guides live here, one WebP per figure, named after the
/// figure (`discord-03-oauth2.webp`). A figure whose file is missing renders
/// as a labelled placeholder, so a guide can ship before its screenshots.
const GUIDE_IMAGES: &str = "static/images/DS2000/guide";

#[derive(Serialize)]
struct Figure {
    src: String,
    width: u32,
    height: u32,
}

#[derive(Serialize)]
struct Tab {
    key: &'static str,
    path: &'static str,
}

pub async fn download(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let (app, firmware) = tokio::join!(
        state.ds2000_app_release.latest(),
        state.ds2000_firmware_release.latest()
    );
    let mut context = guide_context(&state, lang, "download");
    context.insert("app_repo", state.ds2000_app_release.repo());
    context.insert("firmware_repo", state.ds2000_firmware_release.repo());
    context.insert("app_release", &app);
    context.insert("firmware_release", &firmware);
    pages::render(&state, "ds2000/download.html", &context, lang)
}

pub async fn install(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let context = guide_context(&state, lang, "install");
    pages::render(&state, "ds2000/install.html", &context, lang)
}

pub async fn discord_setup(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let context = guide_context(&state, lang, "discord");
    pages::render(&state, "ds2000/discord_setup.html", &context, lang)
}

pub async fn source(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    let context = guide_context(&state, lang, "source");
    pages::render(&state, "ds2000/source.html", &context, lang)
}

/// Context for a DS2000 tab other than the product page: SEO, a `WebPage`
/// schema and breadcrumbs one level below DS2000, the tab bar, and the
/// screenshots available for the guides.
fn guide_context(state: &AppState, lang: Language, tab: &str) -> tera::Context {
    let path = TABS
        .iter()
        .find(|(key, _)| *key == tab)
        .map(|(_, path)| *path)
        .unwrap_or("ds2000");
    let meta = page_meta(state, lang, &format!("ds2000_{tab}")).path(path);

    let schema = json_ld::webpage_schema(
        &state.config,
        lang,
        &meta.title,
        &meta.description,
        "WebPage",
        &format!("{}/{}", lang.as_str(), path),
    );
    let breadcrumbs = json_ld::breadcrumbs(
        &state.config,
        lang,
        &[
            (home_crumb(state, lang), ""),
            (DS2000_CRUMB, "ds2000"),
            (meta.title.as_str(), path),
        ],
    );
    let meta = meta.schema(json!([schema, breadcrumbs]));

    let mut context = pages::base_context(state, lang, &meta);
    context.insert("tabs", &tabs());
    context.insert("active_tab", tab);
    context.insert("figures", &guide_figures(Path::new(GUIDE_IMAGES)));
    context
}

fn tabs() -> Vec<Tab> {
    TABS.iter().map(|&(key, path)| Tab { key, path }).collect()
}

/// Every screenshot in `dir`, keyed by file stem, with its pixel size so the
/// page reserves its space before it loads. Read per request: there are a
/// handful, and a new screenshot shows up without a restart.
fn guide_figures(dir: &Path) -> HashMap<String, Figure> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return HashMap::new();
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension()? != "webp" {
                return None;
            }
            let stem = path.file_stem()?.to_str()?.to_string();
            let bytes = std::fs::read(&path).ok()?;
            let (width, height) = webp_size(&bytes)?;
            let src = format!("/{}/{}.webp", GUIDE_IMAGES, stem);
            Some((stem, Figure { src, width, height }))
        })
        .collect()
}

/// Width and height of a WebP image, read from its header (lossy, lossless
/// and extended formats).
fn webp_size(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 30 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    let le24 = |b: &[u8]| u32::from(b[0]) | (u32::from(b[1]) << 8) | (u32::from(b[2]) << 16);
    match &bytes[12..16] {
        b"VP8X" => Some((le24(&bytes[24..27]) + 1, le24(&bytes[27..30]) + 1)),
        b"VP8L" => {
            let bits = u32::from_le_bytes(bytes[21..25].try_into().ok()?);
            Some(((bits & 0x3FFF) + 1, ((bits >> 14) & 0x3FFF) + 1))
        }
        b"VP8 " => {
            let width = u32::from(u16::from_le_bytes([bytes[26], bytes[27]]) & 0x3FFF);
            let height = u32::from(u16::from_le_bytes([bytes[28], bytes[29]]) & 0x3FFF);
            Some((width, height))
        }
        _ => None,
    }
}

pub async fn privacy_policy(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    legal_page(
        &state,
        lang,
        "privacy_policy",
        "ds2000/privacy-policy",
        "privacy",
    )
}

pub async fn terms_of_service(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    legal_page(
        &state,
        lang,
        "terms_of_service",
        "ds2000/terms-of-service",
        "terms",
    )
}

/// The code signing policy SignPath Foundation requires of the projects it
/// signs for; the product and download pages link to it by that name.
pub async fn code_signing(Lang(lang): Lang, Extension(state): Extension<AppState>) -> Response {
    legal_page(
        &state,
        lang,
        "code_signing",
        "ds2000/code-signing",
        "signing",
    )
}

/// The DS2000's terms of service, privacy policy and code signing policy are
/// legal text rather than a product page, so they get a plain `WebPage` (like
/// any other content page) rather than `ds2000_schema` - and a breadcrumb one
/// level deeper, under DS2000. They share `ds2000/legal.html`; `doc` picks the
/// copy under `ds2000_legal`.
fn legal_page(state: &AppState, lang: Language, page: &str, path: &str, doc: &str) -> Response {
    let meta = page_meta(state, lang, page).path(path);

    let schema = json_ld::webpage_schema(
        &state.config,
        lang,
        &meta.title,
        &meta.description,
        "WebPage",
        &format!("{}/{}", lang.as_str(), path),
    );
    let breadcrumbs = json_ld::breadcrumbs(
        &state.config,
        lang,
        &[
            (home_crumb(state, lang), ""),
            (DS2000_CRUMB, "ds2000"),
            (meta.title.as_str(), path),
        ],
    );
    let meta = meta.schema(json!([schema, breadcrumbs]));

    let mut context = pages::base_context(state, lang, &meta);
    context.insert("doc", doc);
    pages::render(state, "ds2000/legal.html", &context, lang)
}

fn home_crumb(state: &AppState, lang: Language) -> &str {
    state.translations.text_or(lang, "page_titles.home", "Home")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_colourway_is_one_of_the_colourways() {
        for _ in 0..50 {
            assert!(COLOURWAYS.contains(&random_colourway()));
        }
    }

    #[test]
    fn reads_webp_sizes() {
        // The site's own renders are real WebP files of a known size.
        let bytes = std::fs::read("static/images/DS2000/renders/original-frente.webp").unwrap();
        assert_eq!(webp_size(&bytes), Some((1536, 1024)));
        assert_eq!(webp_size(b"not an image at all, not even close"), None);
    }

    #[test]
    fn every_guide_screenshot_is_readable() {
        // A screenshot the header parser cannot read would silently render as
        // a placeholder instead of the image.
        if let Ok(entries) = std::fs::read_dir(GUIDE_IMAGES) {
            let webps = entries
                .flatten()
                .filter(|e| e.path().extension().is_some_and(|ext| ext == "webp"))
                .count();
            assert_eq!(guide_figures(Path::new(GUIDE_IMAGES)).len(), webps);
        }
    }

    #[test]
    fn every_tab_has_a_title_and_description() {
        let state = AppState::build(crate::config::AppConfig::for_tests()).unwrap();
        for (tab, _) in TABS.iter().skip(1) {
            for lang in Language::ALL {
                let meta = page_meta(&state, lang, &format!("ds2000_{tab}"));
                assert_ne!(meta.title, format!("ds2000_{tab}"), "{tab} has no title");
                assert!(!meta.description.is_empty(), "{tab} has no description");
            }
        }
    }

    #[test]
    fn download_page_lists_the_published_files() {
        use crate::data::github_releases::{AssetInfo, ReleaseInfo};
        let asset = |name: &str| AssetInfo {
            name: name.to_string(),
            url: format!("https://example.com/{name}"),
            size: "1.0 MB".to_string(),
            sha256: Some("feedface".to_string()),
        };
        let release = |tag: &str, assets: Vec<AssetInfo>| ReleaseInfo {
            tag: tag.to_string(),
            date: "2026-10-20".to_string(),
            url: format!("https://example.com/{tag}"),
            assets,
        };

        let state = AppState::build(crate::config::AppConfig::for_tests()).unwrap();
        let mut context = guide_context(&state, Language::English, "download");
        context.insert("app_repo", "owner/app");
        context.insert("firmware_repo", "owner/firmware");
        context.insert(
            "app_release",
            &Some(release("v0.2.0", vec![asset("DS2000_0.2.0_x64_en-US.msi")])),
        );
        context.insert(
            "firmware_release",
            &Some(release(
                "v0.2.1",
                vec![
                    asset("DS2000-Firmware_0.2.1_pico.uf2"),
                    asset("DS2000-Firmware_0.2.1_rp2350_zero.uf2"),
                    asset("SHA256SUMS"),
                ],
            )),
        );
        // Tera escapes `/` in attributes; browsers read it back the same.
        let html = state
            .tera
            .render("ds2000/download.html", &context)
            .unwrap()
            .replace("&#x2F;", "/");

        assert!(html.contains("https://example.com/DS2000_0.2.0_x64_en-US.msi"));
        assert!(html.contains("feedface"));
        assert!(html.contains("https://example.com/DS2000-Firmware_0.2.1_rp2350_zero.uf2"));
        assert!(html.contains("https://example.com/DS2000-Firmware_0.2.1_pico.uf2"));
        // The prototype's board comes first, whatever order GitHub lists them in.
        assert!(html.find("rp2350_zero.uf2") < html.find("_pico.uf2"));
        assert!(!html.contains("SHA256SUMS\""));
    }

    #[test]
    fn link_preview_image_exists() {
        assert!(
            std::path::Path::new(OG_IMAGE).is_file(),
            "missing {OG_IMAGE}"
        );
    }

    #[test]
    fn every_colourway_has_all_four_renders() {
        // The templates build the image paths from the colourway, so a missing
        // file would only show up as a broken image on some visits.
        for colourway in COLOURWAYS {
            for view in ["frente", "trasera", "superior", "explotada"] {
                let path = format!("static/images/DS2000/renders/{colourway}-{view}.webp");
                assert!(std::path::Path::new(&path).is_file(), "missing {path}");
            }
        }
    }
}
