use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

use tracing::{info, warn};

use crate::language::Language;
use crate::models::blog_post::BlogPost;

/// Blog posts and their bodies, loaded once at startup.
///
/// Posts and post bodies used to be read and parsed on every single request,
/// with a `stat` on every call to notice a changed file - useful while
/// hot-editing under `cargo run`, but blocking I/O on every request in
/// production for content that only ever changes with a deploy, which
/// recreates the process anyway. `make watch` already restarts on any
/// source/content/data change, so there was nothing left to hot-reload for.
pub struct BlogStore {
    posts: Vec<BlogPost>,
    /// Post bodies, keyed by (route, language).
    content: HashMap<(String, Language), String>,
}

impl BlogStore {
    /// Load every post and its body once.
    ///
    /// A missing `blog_posts.json` is treated as zero posts, not an error -
    /// `data_dir` is shared with other stores (contact messages), and a
    /// fresh checkout or a test exercising one of them shouldn't be forced to
    /// also provide blog data. A `blog_posts.json` that exists but doesn't
    /// parse, or a post declaring an unsafe route, does fail - the same
    /// treatment a broken Tera template already gets: reported at startup,
    /// not on the first request. A body missing for one language is not a
    /// data error either - `/blog/{id}` already falls back to the default
    /// language for it (see `routes::blog::post_body`) - so it's only logged
    /// once here instead of on every request that hits it.
    pub fn load(data_dir: &Path, content_dir: &Path) -> io::Result<Self> {
        let posts = read_posts(&data_dir.join("blog_posts.json"))?;
        let content_dir = content_dir.join("blog");

        let mut content = HashMap::new();
        for post in &posts {
            let Some(route) = post.route.as_deref() else {
                continue;
            };
            if !is_safe_route(route) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid post route: {}", route),
                ));
            }

            for language in Language::ALL {
                let path = content_dir
                    .join(route)
                    .join(format!("{}.html", language.as_str()));
                match fs::read_to_string(&path) {
                    Ok(body) => {
                        content.insert((route.to_string(), language), body);
                    }
                    Err(e) => warn!(
                        "Missing {} body for post {} ({}): {}",
                        language.as_str(),
                        post.id,
                        path.display(),
                        e
                    ),
                }
            }
        }

        info!("Loaded {} blog posts", posts.len());
        Ok(Self { posts, content })
    }

    /// All posts, most recent first.
    pub fn posts(&self) -> &[BlogPost] {
        &self.posts
    }

    /// Body of a post, in the requested language.
    pub fn content(&self, route: &str, lang: Language) -> Option<&str> {
        self.content
            .get(&(route.to_string(), lang))
            .map(String::as_str)
    }
}

/// A missing file is treated as "no posts yet" (a fresh checkout, a test
/// `data_dir` that only cares about some other store), not a startup
/// failure - only a file that exists but doesn't parse is, matching how
/// `MessageStore` already treats a missing `messages.json`.
fn read_posts(path: &Path) -> io::Result<Vec<BlogPost>> {
    let data = match fs::read_to_string(path) {
        Ok(data) => data,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => {
            return Err(io::Error::new(
                e.kind(),
                format!("Error opening {}: {}", path.display(), e),
            ));
        }
    };
    let mut posts: Vec<BlogPost> = serde_json::from_str(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Error parsing {}: {}", path.display(), e),
        )
    })?;
    posts.sort_by_key(|post| Reverse(post.date));
    Ok(posts)
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
        let store =
            BlogStore::load(Path::new("data"), Path::new("content")).expect("posts should load");
        let posts = store.posts();
        assert!(!posts.is_empty());
        for pair in posts.windows(2) {
            assert!(pair[0].date >= pair[1].date, "posts must be newest first");
        }
    }

    #[test]
    fn reads_post_bodies_for_every_language() {
        let store =
            BlogStore::load(Path::new("data"), Path::new("content")).expect("posts should load");
        for post in store.posts() {
            let Some(route) = post.route.as_deref() else {
                continue;
            };
            for language in Language::ALL {
                assert!(
                    store.content(route, language).is_some(),
                    "missing body for post {} in {}",
                    post.id,
                    language.as_str()
                );
            }
        }
    }

    #[test]
    fn an_unlisted_route_has_no_content() {
        let store =
            BlogStore::load(Path::new("data"), Path::new("content")).expect("posts should load");
        assert!(store.content("../../secrets", Language::Spanish).is_none());
        assert!(store.content("no-such-post", Language::Spanish).is_none());
    }

    /// `data_dir` is shared with other stores (contact messages), so a
    /// missing `blog_posts.json` there must not stop the whole app from
    /// building - only a file that exists but is broken should. See #92: a
    /// contact-form test that only cares about `messages.json` pointed
    /// `data_dir` at an otherwise-empty temp directory and broke when this
    /// treated the missing file as fatal.
    #[test]
    fn a_missing_posts_file_is_zero_posts_not_an_error() {
        let dir = fixture_dir("missing-posts-file");
        let store = BlogStore::load(&dir, &dir.join("content")).expect("should load with no file");
        assert!(store.posts().is_empty());
        cleanup(&dir);
    }

    /// A malicious or malformed route in `blog_posts.json` is refused at load
    /// time - there's no request-time path left to guard, since `content()`
    /// only ever looks up an in-memory map built from routes checked here.
    #[test]
    fn refuses_to_load_an_unsafe_route() {
        let dir = fixture_dir("unsafe-route");
        write_posts_json(
            &dir,
            r#"[{"id":"1","title":{"es":"a","en":"a"},
            "summary":null,"thumbnail":null,"route":"../../secrets","date":"01-01-2025"}]"#,
        );

        let result = BlogStore::load(&dir, &dir.join("content"));
        assert!(result.is_err(), "an unsafe route must fail to load");

        cleanup(&dir);
    }

    /// Post bodies used to live under `templates/blog/`, so Tera globbed and
    /// parsed them at startup along with every real template. A body
    /// containing a stray `{{` or `{%` - entirely plausible in a code sample -
    /// made that glob fail and refused to start the whole site. `content_dir`
    /// is a directory Tera never looks at, so a broken body can neither stop
    /// the app from building nor stop `BlogStore` from serving it as the
    /// plain text it is. See #90.
    #[test]
    fn a_broken_post_body_does_not_stop_tera_from_building() {
        let dir = fixture_dir("broken-body");
        let templates_dir = dir.join("templates");
        let content_dir = dir.join("content");
        fs::create_dir_all(&templates_dir).expect("create templates dir");
        fs::create_dir_all(content_dir.join("blog/broken")).expect("create content dir");
        fs::write(templates_dir.join("index.html"), "<html></html>").expect("write template");

        write_posts_json(
            &dir,
            r#"[{"id":"broken","title":{"es":"a","en":"a"},
                "summary":null,"thumbnail":null,"route":"broken","date":"01-01-2025"}]"#,
        );
        let broken_body = "Example: `{{ let x = 5; }}` and a lone `{% for`";
        fs::write(content_dir.join("blog/broken/es.html"), broken_body).expect("write body");
        fs::write(content_dir.join("blog/broken/en.html"), broken_body).expect("write body");

        let glob = format!("{}/**/*", templates_dir.display());
        assert!(
            tera::Tera::new(&glob).is_ok(),
            "a broken post body must not stop Tera from building the real templates"
        );

        let store = BlogStore::load(&dir, &content_dir).expect("posts should load");
        assert_eq!(
            store.content("broken", Language::Spanish),
            Some(broken_body)
        );

        cleanup(&dir);
    }

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("mechardo-blog-{}-{}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create fixture dir");
        dir
    }

    fn write_posts_json(dir: &Path, contents: &str) {
        fs::write(dir.join("blog_posts.json"), contents).expect("write blog_posts.json");
    }

    fn cleanup(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }
}
