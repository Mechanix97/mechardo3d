use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use tracing::{info, warn};

use crate::language::Language;
use crate::models::blog_post::BlogPost;

/// Cached view over the blog content stored on disk.
///
/// Posts and post bodies used to be read and parsed on every single request.
/// They are now parsed once and re-read only when the file's modification time
/// changes, which keeps `cargo run` hot-editing while removing the per-request
/// disk hit in production.
pub struct BlogStore {
    posts_path: PathBuf,
    content_dir: PathBuf,
    posts: RwLock<Option<CachedPosts>>,
    content: RwLock<HashMap<PathBuf, CachedContent>>,
}

struct CachedPosts {
    modified: Option<SystemTime>,
    posts: Arc<Vec<BlogPost>>,
}

struct CachedContent {
    modified: Option<SystemTime>,
    body: Arc<String>,
}

impl BlogStore {
    pub fn new(data_dir: &Path, templates_dir: &Path) -> Self {
        Self {
            posts_path: data_dir.join("blog_posts.json"),
            content_dir: templates_dir.join("blog"),
            posts: RwLock::new(None),
            content: RwLock::new(HashMap::new()),
        }
    }

    /// All posts, most recent first.
    pub fn posts(&self) -> io::Result<Arc<Vec<BlogPost>>> {
        let modified = modified_at(&self.posts_path);

        {
            let cache = self.posts.read().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = cache.as_ref()
                && cached.modified == modified
                && modified.is_some()
            {
                return Ok(Arc::clone(&cached.posts));
            }
        }

        let posts = Arc::new(read_posts(&self.posts_path)?);
        let mut cache = self.posts.write().unwrap_or_else(|e| e.into_inner());
        *cache = Some(CachedPosts {
            modified,
            posts: Arc::clone(&posts),
        });
        Ok(posts)
    }

    /// Rendered body of a post, in the requested language.
    pub fn content(&self, route: &str, lang: Language) -> io::Result<Arc<String>> {
        if !is_safe_route(route) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("Invalid post route: {}", route),
            ));
        }

        let path = self
            .content_dir
            .join(route)
            .join(format!("{}.html", lang.as_str()));
        let modified = modified_at(&path);

        {
            let cache = self.content.read().unwrap_or_else(|e| e.into_inner());
            if let Some(cached) = cache.get(&path)
                && cached.modified == modified
                && modified.is_some()
            {
                return Ok(Arc::clone(&cached.body));
            }
        }

        let contents = Arc::new(fs::read_to_string(&path).map_err(|e| {
            io::Error::new(e.kind(), format!("Error reading {}: {}", path.display(), e))
        })?);

        let mut cache = self.content.write().unwrap_or_else(|e| e.into_inner());
        cache.insert(
            path,
            CachedContent {
                modified,
                body: Arc::clone(&contents),
            },
        );
        Ok(contents)
    }

    /// Load posts eagerly so that a broken data file is reported at startup
    /// instead of on the first request.
    pub fn warm(&self) {
        match self.posts() {
            Ok(posts) => info!("Loaded {} blog posts", posts.len()),
            Err(e) => warn!("Blog posts unavailable: {}", e),
        }
    }
}

fn read_posts(path: &Path) -> io::Result<Vec<BlogPost>> {
    let data = fs::read_to_string(path).map_err(|e| {
        io::Error::new(e.kind(), format!("Error opening {}: {}", path.display(), e))
    })?;
    let mut posts: Vec<BlogPost> = serde_json::from_str(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error parsing {}: {}", path.display(), e),
        )
    })?;
    posts.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(posts)
}

fn modified_at(path: &Path) -> Option<SystemTime> {
    fs::metadata(path)
        .ok()
        .and_then(|meta| meta.modified().ok())
}

/// Post routes come from the data file and are used to build a path, so they
/// must stay a single, plain directory name.
fn is_safe_route(route: &str) -> bool {
    !route.is_empty()
        && route != "."
        && route != ".."
        && !route.contains('/')
        && !route.contains('\\')
        && !route.contains('\0')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_routes_that_escape_the_content_directory() {
        assert!(is_safe_route("iot-alarm"));
        assert!(!is_safe_route(".."));
        assert!(!is_safe_route("../../secrets"));
        assert!(!is_safe_route("a/b"));
        assert!(!is_safe_route(""));
    }

    #[test]
    fn loads_repository_posts_sorted_by_date() {
        let store = BlogStore::new(Path::new("data"), Path::new("templates"));
        let posts = store.posts().expect("repository posts should parse");
        assert!(!posts.is_empty());
        for pair in posts.windows(2) {
            assert!(pair[0].date >= pair[1].date, "posts must be newest first");
        }
    }

    #[test]
    fn serves_posts_from_cache_on_repeated_reads() {
        let store = BlogStore::new(Path::new("data"), Path::new("templates"));
        let first = store.posts().expect("posts");
        let second = store.posts().expect("posts");
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn reads_post_bodies_for_every_language() {
        let store = BlogStore::new(Path::new("data"), Path::new("templates"));
        let posts = store.posts().expect("posts");
        for post in posts.iter() {
            let Some(route) = post.route.as_deref() else {
                continue;
            };
            for language in Language::ALL {
                assert!(
                    store.content(route, language).is_ok(),
                    "missing body for post {} in {}",
                    post.id,
                    language.as_str()
                );
            }
        }
    }

    #[test]
    fn rejects_traversal_routes_at_runtime() {
        let store = BlogStore::new(Path::new("data"), Path::new("templates"));
        assert!(store.content("../../secrets", Language::Spanish).is_err());
    }
}
