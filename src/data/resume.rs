use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use reqwest::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use serde::Deserialize;
use tracing::{info, warn};

use crate::config::ResumeConfig;

const GITHUB_API: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";
/// GitHub rejects API calls without one.
const AGENT: &str = "mechardo3d-site";
/// A resume that grows past this is a sign something else came back - an HTML
/// error page, the wrong asset - and is not worth holding in memory.
const MAX_PDF_BYTES: usize = 8 * 1024 * 1024;

/// The PDF of one release asset, and when it was downloaded.
#[derive(Clone)]
struct CachedPdf {
    bytes: Arc<Vec<u8>>,
    fetched_at: Instant,
}

/// Serves the resume PDFs published as releases of a private repository.
///
/// The repository stays private: the token never leaves the server, and
/// visitors only ever see the bytes of the asset. Downloads are cached for
/// `cache_ttl` so a burst of visitors is one call to GitHub, not one each.
pub struct ResumeStore {
    config: ResumeConfig,
    http: Client,
    /// Keyed by asset file name.
    cache: Mutex<HashMap<String, CachedPdf>>,
}

#[derive(Deserialize)]
struct Release {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    assets: Vec<ReleaseAsset>,
}

#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    id: u64,
}

impl ResumeStore {
    pub fn new(config: ResumeConfig, http: Client) -> Self {
        Self {
            config,
            http,
            cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn enabled(&self) -> bool {
        self.config.enabled()
    }

    /// Asset name for a language, as configured.
    pub fn asset_name(&self, lang: crate::language::Language) -> &str {
        match lang {
            crate::language::Language::Spanish => &self.config.asset_es,
            crate::language::Language::English => &self.config.asset_en,
        }
    }

    /// The PDF for a language, from the cache when it is still fresh.
    ///
    /// A failed refresh falls back to the cached copy, however old: an expired
    /// entry and a GitHub outage should not add up to a broken button.
    pub async fn pdf(&self, lang: crate::language::Language) -> Option<Arc<Vec<u8>>> {
        if !self.enabled() {
            return None;
        }

        let asset = self.asset_name(lang).to_string();
        let cached = self.cached(&asset);
        if let Some(entry) = &cached
            && entry.fetched_at.elapsed() < self.config.cache_ttl
        {
            return Some(Arc::clone(&entry.bytes));
        }

        match self.download(&asset).await {
            Ok(bytes) => {
                let bytes = Arc::new(bytes);
                self.store(&asset, Arc::clone(&bytes));
                Some(bytes)
            }
            Err(e) => {
                warn!("Could not refresh {} from GitHub: {}", asset, e);
                cached.map(|entry| entry.bytes)
            }
        }
    }

    fn cached(&self, asset: &str) -> Option<CachedPdf> {
        self.cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(asset).cloned())
    }

    fn store(&self, asset: &str, bytes: Arc<Vec<u8>>) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                asset.to_string(),
                CachedPdf {
                    bytes,
                    fetched_at: Instant::now(),
                },
            );
        }
    }

    /// Resolve the latest release, then download the asset by name.
    async fn download(&self, asset: &str) -> Result<Vec<u8>, String> {
        let release: Release = self
            .get(&format!(
                "{}/repos/{}/releases/latest",
                GITHUB_API, self.config.repo
            ))
            .header(ACCEPT, "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| format!("release lookup failed: {}", e))?
            .error_for_status()
            .map_err(|e| format!("release lookup rejected: {}", e))?
            .json()
            .await
            .map_err(|e| format!("release payload unreadable: {}", e))?;

        let id = release
            .assets
            .iter()
            .find(|candidate| candidate.name == asset)
            .map(|candidate| candidate.id)
            .ok_or_else(|| format!("release {} has no asset named {}", release.tag_name, asset))?;

        // Asking for the octet-stream redirects to storage. reqwest drops the
        // Authorization header when a redirect crosses hosts, so the token is
        // not handed to the storage backend.
        let bytes = self
            .get(&format!(
                "{}/repos/{}/releases/assets/{}",
                GITHUB_API, self.config.repo, id
            ))
            .header(ACCEPT, "application/octet-stream")
            .send()
            .await
            .map_err(|e| format!("asset download failed: {}", e))?
            .error_for_status()
            .map_err(|e| format!("asset download rejected: {}", e))?
            .bytes()
            .await
            .map_err(|e| format!("asset body unreadable: {}", e))?;

        if bytes.len() > MAX_PDF_BYTES {
            return Err(format!("asset is {} bytes, refusing to cache", bytes.len()));
        }
        if !bytes.starts_with(b"%PDF") {
            return Err("asset does not look like a PDF".to_string());
        }

        info!(
            "Fetched {} ({} bytes) from release {}",
            asset,
            bytes.len(),
            release.tag_name
        );
        Ok(bytes.to_vec())
    }

    fn get(&self, url: &str) -> reqwest::RequestBuilder {
        self.http
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {}", self.config.token))
            .header(USER_AGENT, AGENT)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::Language;
    use std::time::Duration;

    fn store(token: &str) -> ResumeStore {
        ResumeStore::new(
            ResumeConfig {
                repo: "owner/repo".to_string(),
                token: token.to_string(),
                asset_es: "cv-es.pdf".to_string(),
                asset_en: "cv-en.pdf".to_string(),
                cache_ttl: Duration::from_secs(60),
            },
            Client::new(),
        )
    }

    #[test]
    fn stays_disabled_without_a_token() {
        assert!(!store("").enabled());
        assert!(store("token").enabled());
    }

    #[test]
    fn serves_a_different_variant_per_language() {
        let store = store("token");
        assert_eq!(store.asset_name(Language::Spanish), "cv-es.pdf");
        assert_eq!(store.asset_name(Language::English), "cv-en.pdf");
    }

    #[tokio::test]
    async fn never_calls_github_when_disabled() {
        assert!(store("").pdf(Language::Spanish).await.is_none());
    }

    #[test]
    fn reuses_a_fresh_download() {
        let store = store("token");
        store.store("cv-es.pdf", Arc::new(b"%PDF-1.7".to_vec()));

        let cached = store.cached("cv-es.pdf").expect("entry should be cached");
        assert!(cached.fetched_at.elapsed() < store.config.cache_ttl);
        assert_eq!(cached.bytes.as_slice(), b"%PDF-1.7");
        assert!(store.cached("cv-en.pdf").is_none());
    }
}
