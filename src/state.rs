use std::ops::Deref;
use std::sync::Arc;
use std::time::Duration;

use reqwest::Client;
use tera::Tera;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::data::blog_data::BlogStore;
use crate::data::messages::MessageStore;
use crate::data::resume::ResumeStore;
use crate::date_format;
use crate::rate_limit::RateLimiter;
use crate::translations::Translations;

/// Timeout for outbound calls (currently only reCAPTCHA verification), so a
/// slow third party cannot pin a request handler open.
const HTTP_TIMEOUT: Duration = Duration::from_secs(10);

pub struct AppStateInner {
    pub config: AppConfig,
    pub tera: Tera,
    pub translations: Translations,
    pub blog: BlogStore,
    pub messages: MessageStore,
    pub contact_rate_limit: RateLimiter,
    pub resume: ResumeStore,
    pub http: Client,
}

/// Everything the handlers share, behind a single cheap-to-clone handle.
///
/// The Tera engine and the translation tree used to be cloned in full on every
/// request; they are now shared by reference.
#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl AppState {
    pub fn build(config: AppConfig) -> Result<Self, tera::Error> {
        let mut tera = Tera::new(&config.template_glob())?;
        tera.register_filter("date_format", date_format::date_format);

        let translations = Translations::load(&config.translations_dir);
        let blog = BlogStore::new(&config.data_dir, &config.templates_dir);
        blog.warm();

        let messages = MessageStore::new(&config.data_dir);
        info!(
            "Contact messages are stored in {}",
            messages.path().display()
        );
        let contact_rate_limit = RateLimiter::new(config.contact_rate_limit);

        let http = Client::builder()
            .timeout(HTTP_TIMEOUT)
            .build()
            .unwrap_or_else(|e| {
                warn!("Falling back to a default HTTP client: {}", e);
                Client::new()
            });

        let resume = ResumeStore::new(config.resume.clone(), http.clone());

        Ok(Self(Arc::new(AppStateInner {
            config,
            tera,
            translations,
            blog,
            messages,
            contact_rate_limit,
            resume,
            http,
        })))
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
