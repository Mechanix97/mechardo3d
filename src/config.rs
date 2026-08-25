use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use tracing::{info, warn};

/// Public reCAPTCHA v3 site key used by the contact form.
const DEFAULT_RECAPTCHA_SITE_KEY: &str = "6LfuI5YrAAAAAOEUv-Xp1Ewo4dhr1TgCrCG_aqa8";
const DEFAULT_BASE_URL: &str = "https://mechardo3d.xyz";
const DEFAULT_SECRET_FILE: &str = "secrets/recaptcha.env";

/// Runtime configuration, resolved once at startup.
///
/// Every value has a safe default so that `cargo run` works with no environment
/// set up at all; production overrides them through the environment.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Address the HTTP server binds to (`BIND_ADDR` + `PORT`).
    pub bind_addr: SocketAddr,
    /// Public origin of the site, without a trailing slash (`BASE_URL`).
    pub base_url: String,
    /// Whether the language cookie is flagged `Secure` (`COOKIE_SECURE`).
    pub cookie_secure: bool,
    /// Whether `X-Forwarded-For` / `X-Real-IP` may be trusted (`TRUST_PROXY_HEADERS`).
    ///
    /// The app always runs behind Caddy in production, where the socket address
    /// is the proxy's and not the visitor's.
    pub trust_proxy_headers: bool,
    /// Optional `Content-Security-Policy` header value (`CONTENT_SECURITY_POLICY`).
    pub content_security_policy: Option<String>,
    /// Whether to send `Strict-Transport-Security` (`HSTS_ENABLED`).
    pub hsts_enabled: bool,
    pub recaptcha: RecaptchaConfig,
    /// Minimum delay between two contact submissions from the same client.
    pub contact_rate_limit: Duration,
    /// Largest accepted contact message, in characters.
    pub max_message_chars: usize,
    pub data_dir: PathBuf,
    pub static_dir: PathBuf,
    pub templates_dir: PathBuf,
    pub translations_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct RecaptchaConfig {
    pub site_key: String,
    pub secret: String,
    pub min_score: f32,
    /// Skips verification entirely. Only meant for local development.
    pub disabled: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let host = env_string("BIND_ADDR", "0.0.0.0");
        let port: u16 = env_parse("PORT", 3000);
        let bind_addr = format!("{}:{}", host, port)
            .parse::<SocketAddr>()
            .unwrap_or_else(|e| {
                warn!(
                    "Invalid BIND_ADDR/PORT ({}:{}): {}. Falling back to 0.0.0.0:3000",
                    host, port, e
                );
                SocketAddr::from(([0, 0, 0, 0], 3000))
            });

        let base_url = env_string("BASE_URL", DEFAULT_BASE_URL)
            .trim_end_matches('/')
            .to_string();

        let secret_file = env_string("RECAPTCHA_SECRET_FILE", DEFAULT_SECRET_FILE);
        let secret = env::var("RECAPTCHA_SECRET_KEY")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.trim().to_string())
            .or_else(|| read_secret_file(&secret_file, "RECAPTCHA_SECRET_KEY"))
            .unwrap_or_default();

        let disabled = env_bool("RECAPTCHA_DISABLED", false);
        if secret.is_empty() && !disabled {
            warn!(
                "No reCAPTCHA secret found (set RECAPTCHA_SECRET_KEY or {}). \
                 Contact form submissions will be rejected. \
                 Set RECAPTCHA_DISABLED=true to bypass verification locally.",
                secret_file
            );
        }

        let config = Self {
            bind_addr,
            base_url,
            cookie_secure: env_bool("COOKIE_SECURE", false),
            trust_proxy_headers: env_bool("TRUST_PROXY_HEADERS", true),
            content_security_policy: env::var("CONTENT_SECURITY_POLICY")
                .ok()
                .filter(|v| !v.trim().is_empty()),
            hsts_enabled: env_bool("HSTS_ENABLED", true),
            recaptcha: RecaptchaConfig {
                site_key: env_string("RECAPTCHA_SITE_KEY", DEFAULT_RECAPTCHA_SITE_KEY),
                secret,
                min_score: env_parse("RECAPTCHA_MIN_SCORE", 0.6_f32),
                disabled,
            },
            contact_rate_limit: Duration::from_secs(env_parse("CONTACT_RATE_LIMIT_SECS", 300)),
            max_message_chars: env_parse("MAX_MESSAGE_CHARS", 5000),
            data_dir: PathBuf::from(env_string("DATA_DIR", "data")),
            static_dir: PathBuf::from(env_string("STATIC_DIR", "static")),
            templates_dir: PathBuf::from(env_string("TEMPLATES_DIR", "templates")),
            translations_dir: PathBuf::from(env_string("TRANSLATIONS_DIR", "translations")),
        };

        info!(
            "Configuration loaded: bind={} base_url={} trust_proxy_headers={} recaptcha_disabled={}",
            config.bind_addr,
            config.base_url,
            config.trust_proxy_headers,
            config.recaptcha.disabled
        );

        config
    }

    /// Absolute URL for a site-relative path (`me` -> `https://host/en/me`).
    pub fn url(&self, path: &str) -> String {
        let path = path.trim_start_matches('/');
        if path.is_empty() {
            format!("{}/", self.base_url)
        } else {
            format!("{}/{}", self.base_url, path)
        }
    }

    /// Glob Tera uses to discover templates.
    pub fn template_glob(&self) -> String {
        format!("{}/**/*", self.templates_dir.display())
    }
}

fn env_string(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => value.trim().to_string(),
        _ => default.to_string(),
    }
}

fn env_parse<T: FromStr>(key: &str, default: T) -> T {
    match env::var(key) {
        Ok(value) => match value.trim().parse::<T>() {
            Ok(parsed) => parsed,
            Err(_) => {
                warn!("Invalid value for {}: {:?}. Using default.", key, value);
                default
            }
        },
        Err(_) => default,
    }
}

fn env_bool(key: &str, default: bool) -> bool {
    match env::var(key) {
        Ok(value) => parse_bool(&value).unwrap_or_else(|| {
            warn!("Invalid boolean for {}: {:?}. Using default.", key, value);
            default
        }),
        Err(_) => default,
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn read_secret_file(path: &str, key: &str) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(contents) => value_from_env_file(&contents, key),
        Err(e) => {
            warn!("Could not read {}: {}", path, e);
            None
        }
    }
}

/// Extract `KEY=value` from the contents of a dotenv-style file.
fn value_from_env_file(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }
        let (name, value) = line.split_once('=')?;
        if name.trim() != key {
            return None;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_booleans() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool(" YES "), Some(true));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn reads_values_from_env_file() {
        let contents = "# comment\nOTHER=1\nRECAPTCHA_SECRET_KEY=\"abc123\"\n";
        assert_eq!(
            value_from_env_file(contents, "RECAPTCHA_SECRET_KEY"),
            Some("abc123".to_string())
        );
        assert_eq!(value_from_env_file(contents, "MISSING"), None);
    }

    #[test]
    fn ignores_empty_values_in_env_file() {
        assert_eq!(value_from_env_file("KEY=\n", "KEY"), None);
    }

    #[test]
    fn builds_absolute_urls() {
        let config = AppConfig {
            base_url: "https://example.com".to_string(),
            ..test_config()
        };
        assert_eq!(config.url(""), "https://example.com/");
        assert_eq!(config.url("/en/me"), "https://example.com/en/me");
        assert_eq!(config.url("en/blog"), "https://example.com/en/blog");
    }

    fn test_config() -> AppConfig {
        AppConfig {
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
            base_url: "https://example.com".to_string(),
            cookie_secure: false,
            trust_proxy_headers: true,
            content_security_policy: None,
            hsts_enabled: true,
            recaptcha: RecaptchaConfig {
                site_key: "site".to_string(),
                secret: "secret".to_string(),
                min_score: 0.6,
                disabled: false,
            },
            contact_rate_limit: Duration::from_secs(300),
            max_message_chars: 5000,
            data_dir: PathBuf::from("data"),
            static_dir: PathBuf::from("static"),
            templates_dir: PathBuf::from("templates"),
            translations_dir: PathBuf::from("translations"),
        }
    }
}
