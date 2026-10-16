use std::sync::Mutex;
use std::time::Instant;

use reqwest::header::{ACCEPT, AUTHORIZATION, USER_AGENT};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::config::ReleaseFeedConfig;

const GITHUB_API: &str = "https://api.github.com";
const GITHUB_API_VERSION: &str = "2022-11-28";
/// GitHub rejects API calls without one.
const AGENT: &str = "mechardo3d-site";

/// The latest published release of a public repository, as the download page
/// shows it.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReleaseInfo {
    pub tag: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    pub url: String,
    pub assets: Vec<AssetInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AssetInfo {
    pub name: String,
    pub url: String,
    /// Human-readable size, e.g. `4.2 MB`.
    pub size: String,
    /// Hex SHA-256 when GitHub reports a digest for the asset.
    pub sha256: Option<String>,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    published_at: Option<String>,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    size: u64,
    #[serde(default)]
    digest: Option<String>,
}

/// What was last learnt about a repository. `release` is `None` when it has
/// no published release yet, which is a normal state, not an error.
#[derive(Clone)]
struct Cached {
    release: Option<ReleaseInfo>,
    fetched_at: Instant,
}

/// Latest release of one public repository, cached for `cache_ttl`.
///
/// Only metadata is fetched: visitors download straight from GitHub. The token
/// is optional and only raises the API rate limit.
pub struct ReleaseFeed {
    repo: String,
    token: String,
    cache_ttl: std::time::Duration,
    http: Client,
    cache: Mutex<Option<Cached>>,
}

impl ReleaseFeed {
    pub fn new(repo: &str, config: &ReleaseFeedConfig, http: Client) -> Self {
        Self {
            repo: repo.to_string(),
            token: config.token.clone(),
            cache_ttl: config.cache_ttl,
            http,
            cache: Mutex::new(None),
        }
    }

    /// `owner/name` of the repository.
    pub fn repo(&self) -> &str {
        &self.repo
    }

    /// The latest release, from the cache while it is fresh.
    ///
    /// A failed refresh keeps whatever was known before, however old: a GitHub
    /// outage should not empty the download page.
    pub async fn latest(&self) -> Option<ReleaseInfo> {
        let cached = self.cache.lock().ok().and_then(|cache| cache.clone());
        if let Some(entry) = &cached
            && entry.fetched_at.elapsed() < self.cache_ttl
        {
            return entry.release.clone();
        }

        match self.fetch().await {
            Ok(release) => {
                if let Ok(mut cache) = self.cache.lock() {
                    *cache = Some(Cached {
                        release: release.clone(),
                        fetched_at: Instant::now(),
                    });
                }
                release
            }
            Err(e) => {
                warn!("Could not read the latest release of {}: {}", self.repo, e);
                cached.and_then(|entry| entry.release)
            }
        }
    }

    async fn fetch(&self) -> Result<Option<ReleaseInfo>, String> {
        let mut request = self
            .http
            .get(format!(
                "{}/repos/{}/releases/latest",
                GITHUB_API, self.repo
            ))
            .header(ACCEPT, "application/vnd.github+json")
            .header(USER_AGENT, AGENT)
            .header("X-GitHub-Api-Version", GITHUB_API_VERSION);
        if !self.token.is_empty() {
            request = request.header(AUTHORIZATION, format!("Bearer {}", self.token));
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("request failed: {}", e))?;
        // `releases/latest` answers 404 while only drafts and pre-releases exist.
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let release: Release = response
            .error_for_status()
            .map_err(|e| format!("rejected: {}", e))?
            .json()
            .await
            .map_err(|e| format!("payload unreadable: {}", e))?;
        Ok(Some(release.into()))
    }
}

impl From<Release> for ReleaseInfo {
    fn from(release: Release) -> Self {
        Self {
            tag: release.tag_name,
            date: release
                .published_at
                .as_deref()
                .and_then(|date| date.get(..10))
                .unwrap_or_default()
                .to_string(),
            url: release.html_url,
            assets: release
                .assets
                .into_iter()
                .map(|asset| AssetInfo {
                    sha256: asset
                        .digest
                        .as_deref()
                        .and_then(|digest| digest.strip_prefix("sha256:"))
                        .map(str::to_string),
                    size: human_size(asset.size),
                    name: asset.name,
                    url: asset.browser_download_url,
                })
                .collect(),
        }
    }
}

fn human_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    let bytes = bytes as f64;
    if bytes >= KB * KB {
        format!("{:.1} MB", bytes / (KB * KB))
    } else {
        format!("{:.0} KB", (bytes / KB).max(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_github_release_payload() {
        let payload = r#"{
            "tag_name": "v0.2.0",
            "html_url": "https://github.com/o/r/releases/tag/v0.2.0",
            "published_at": "2026-10-20T12:00:00Z",
            "assets": [{
                "name": "DS2000_0.2.0_x64_en-US.msi",
                "browser_download_url": "https://github.com/o/r/releases/download/v0.2.0/a.msi",
                "size": 5452595,
                "digest": "sha256:abc123"
            }, {
                "name": "SHA256SUMS",
                "browser_download_url": "https://github.com/o/r/releases/download/v0.2.0/SHA256SUMS",
                "size": 300
            }]
        }"#;
        let release: ReleaseInfo = serde_json::from_str::<Release>(payload).unwrap().into();

        assert_eq!(release.tag, "v0.2.0");
        assert_eq!(release.date, "2026-10-20");
        assert_eq!(release.assets[0].size, "5.2 MB");
        assert_eq!(release.assets[0].sha256.as_deref(), Some("abc123"));
        assert_eq!(release.assets[1].size, "1 KB");
        assert_eq!(release.assets[1].sha256, None);
    }

    #[tokio::test]
    async fn serves_a_fresh_cache_without_calling_github() {
        let feed = ReleaseFeed::new(
            "owner/repo",
            &ReleaseFeedConfig {
                app_repo: "owner/repo".to_string(),
                firmware_repo: String::new(),
                token: String::new(),
                cache_ttl: std::time::Duration::from_secs(60),
            },
            // Any request would fail: the address is unroutable.
            Client::builder()
                .timeout(std::time::Duration::from_millis(1))
                .build()
                .unwrap(),
        );
        *feed.cache.lock().unwrap() = Some(Cached {
            release: None,
            fetched_at: Instant::now(),
        });
        assert_eq!(feed.latest().await, None);
    }
}
