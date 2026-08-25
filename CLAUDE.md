# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based web application built with the Axum framework. It serves as a personal/professional website for Mechardo Labs, featuring a blog, contact form, product pages (DS2000), and language support (English/Spanish).

## Build and Run Commands

### Development
- `cargo run` - Start the development server (port 3000 by default, see Configuration)
- `make run` - Alias for `cargo run`
- `make watch` - Run server with hot reload for Rust, HTML, and CSS changes (requires `watchexec`)
- `npm run build:css` - Build Tailwind CSS
- `npm run watch:css` - Watch and rebuild Tailwind CSS
- `make test` / `cargo test` - Run all unit tests
- `make lint` - `cargo fmt --check` plus `cargo clippy -- -D warnings` (what CI runs)
- `make check` - Lint and test together
- `RECAPTCHA_DISABLED=true cargo run` - Work on the contact form without a reCAPTCHA secret

### Docker
- `make build-image` - Build Docker image locally
- `make run-image` - Build and run Docker container locally (port 3000)
- `make stop-image` - Stop and remove local Docker container
- `make run-prod` - Run production stack with docker-compose (includes Caddy reverse proxy)
- `make stop-prod` - Stop production containers

## Configuration

All runtime configuration comes from environment variables, read once at startup in `src/config.rs`
into an `AppConfig`. Every value has a default, so no setup is required for `cargo run`. The full
table lives in `README.md`; the ones that change behaviour most often are `BASE_URL`,
`RECAPTCHA_SECRET_KEY` / `RECAPTCHA_DISABLED`, `TRUST_PROXY_HEADERS` and `RUST_LOG`.

Never hard-code the public origin, a port, a key or a path in a handler - add it to `AppConfig`.

## High-Level Architecture

### Request Flow
1. Page routes are prefixed with `/{lang}` where lang is "en" or "es"
2. Root "/" redirects to the detected language based on:
   - Cookie preference (if set)
   - Accept-Language HTTP header (parsed in `src/language_detection.rs`)
   - Default: Spanish
3. The `Lang` extractor (`src/extract.rs`) validates the prefix. A prefix that is not a supported
   language is redirected to the visitor's language instead of being rendered as Spanish
4. Unmatched paths either render a localized 404 (when they already carry a supported language) or
   are redirected to their localized equivalent - never both, so there are no redirect loops
5. Language preference is stored in a cookie with 1-year expiration, set by the server on every HTML
   response and also by the front-end when the visitor uses the picker

### Shared State
`src/state.rs` builds one `AppState` (an `Arc` handle) holding the configuration, the Tera engine,
the translations, the blog store, the message store, the contact rate limiter and a shared
`reqwest::Client`. It is attached with a single `Extension` layer, so handlers take
`Extension(state): Extension<AppState>` and reach everything through it.

### Core Modules

**`src/main.rs`** - Router and startup
- Builds the router, applies the middleware layers, binds the listener
- Handles `/`, `/health` and the fallback

**`src/config.rs`** - Environment-driven `AppConfig`

**`src/state.rs`** - `AppState`, shared by every handler

**`src/extract.rs`** - `Lang` extractor plus the URL helpers that decide where a request without a
usable language prefix should go

**`src/middleware.rs`** - Security headers (`nosniff`, `X-Frame-Options`, `Referrer-Policy`,
`Permissions-Policy`, optional HSTS and CSP) and one structured log line per request

**`src/pages.rs`** - `PageMeta`, the shared page context (SEO tags, canonical and `hreflang` URLs,
JSON-LD), `render` (a template failure becomes a 500 page, not a panic), `not_found`, `server_error`

**`src/responses.rs`** - `HtmlWithLang`, the HTML response that sets the language cookie and
`Content-Language`; `language_redirect` for redirects that vary by language

**`src/static_files.rs`** - Static assets: request paths are resolved segment by segment so they
cannot escape the static directory, plus `ETag`/`If-None-Match` and `Cache-Control`

**`src/sitemap.rs`** - `/sitemap.xml` and `/robots.txt`, generated from the routing table and the
blog data so they cannot drift

**`src/rate_limit.rs`** - Fixed-window limiter used by the contact form; expired entries are pruned

**`src/client_ip.rs`** - Client address, read from `X-Forwarded-For` (rightmost entry, the one the
proxy recorded) when proxy headers are trusted

**`src/language.rs`** - `Language` enum (En/Es) with `ALL`, display names, `locale()` and
`og_locale()`

**`src/language_detection.rs`** - Cookie first, then Accept-Language q-value parsing

**`src/date_format.rs`** - Tera filter used as `{{ date | date_format(lang=lang) }}`

**`src/translations.rs`** - Loads and merges every `translations/{lang}/*.json` file; lookups go
through `text`/`text_or` with a dotted path (`page_titles.home`)

**`src/json_ld.rs`** - Schema.org payloads, built from `AppConfig` so URLs follow `BASE_URL`

**`src/routes/`** - One module per page. Handlers are thin: resolve data, build a `PageMeta`, render

**`src/data/blog_data.rs`** - `BlogStore`: posts and post bodies are parsed once and re-read only
when the file's modification time changes

**`src/data/messages.rs`** - `MessageStore`: contact messages, written through a mutex and replaced
atomically

**`src/models/blog_post.rs`** - `BlogPost` (data) and `BlogPostView` (resolved to one language)

### Template System
- Templates use Tera engine, located in `templates/` directory
- Every page extends `base.html`, which renders the SEO/`hreflang` tags from the shared context
- Shared markup lives in `templates/macros/`; the blog post card is
  `cards::post_card(post=..., lang=..., t=..., level="h2"|"h3")`, imported with
  `{% import "macros/cards.html" as cards %}` at the top of the page template
- Date formatting uses the `date_format` filter for localized month names
- One `<h1>` per page, rendered by the page template - blog post bodies under
  `templates/blog/` are fragments and start at `<h2>`
- Never nest a link inside another link; a card links from its title and its call to action,
  and its thumbnail link is `aria-hidden`

### Styling
- Tailwind CSS v4, configured from CSS: `@source` directives in `static/tailwind.css`
  (there is no `tailwind.config.js` - v4 ignores it unless referenced with `@config`)
- Source: `static/tailwind.css`
- Output: `static/style.css`
- Build via `npm run build:css` or `make build-css`
- Use v4 utility names (`shrink-0`, `bg-black/75`), not the v3 spellings
  (`flex-shrink-0`, `bg-opacity-75`) - those silently generate nothing
- Text and buttons use `blue-600`/`blue-700`; white on `blue-500` is 3.68:1 and fails WCAG AA

### Images
- Ship WebP; keep the source file out of `static/`
- Set `width`/`height` whenever CSS does not fix the height, so the page does not reflow
- `loading="lazy"` below the fold, `fetchpriority="high"` on the one image that is the LCP
- `alt=""` for a thumbnail whose link text already names the destination

### Deployment
- Docker multi-stage build (see `Dockerfile`), with a `HEALTHCHECK` against `/health`
- Production uses Caddy reverse proxy (see `docker-compose.yml` and `Caddyfile`), which terminates
  TLS, compresses responses and redirects `www` to the apex domain
- The app container is not published on the host; it is reachable through Caddy only
- Secrets are mounted from `secrets/` and passed as environment variables - they are not baked into
  the image

## Key Development Patterns

### Adding a New Route
1. Create handler in `src/routes/[name].rs`
2. Add module declaration in `src/routes/mod.rs`
3. Register in `router()` within `src/main.rs` with pattern `/{lang}/path`
4. Take `Lang(lang): Lang` and `Extension(state): Extension<AppState>`; return `Response`
5. Build the metadata with `pages::page_meta(&state, lang, "<page key>")`, adding `.path(...)`,
   `.og_type(...)` and `.schema(...)` as needed
6. Build the context with `pages::base_context(&state, lang, &meta)` and insert page-specific data
7. Render with `pages::render(&state, "template.html", &context, lang)` - never `.expect()` on a
   render, and never put an error message in the response body
8. Add the page to `STATIC_PAGES` in `src/sitemap.rs`

### Template Context Variables
Every template receives:
- `lang`, `locale` - Current language
- `t` - Translations for the current language (access via `t.key_name`)
- `title`, `meta_description`, `meta_keywords`, `og_*`, `robots`
- `base_url`, `canonical_url`, `canonical_path`, `alternates`, `x_default_url`
- `json_ld_schema` when the page provides one
- Route-specific data (e.g. blog posts, contact form fields)

### Translation System
- Add translations to `translations/{lang}/[module].json`; new files are picked up automatically
- Page copy lives under its own top-level key; SEO copy lives in `meta.json`
  (`meta.<page>.description`, `meta.<page>.keywords`), page titles in `common.json` under
  `page_titles`
- Use `{{ t.key_name }}` in templates and `state.translations.text_or(lang, "a.b", default)` in Rust
- Keys must exist in both languages - a test enforces that

### Blog Posts
- Stored as JSON in `data/blog_posts.json`
- Structure: `{ "id": "...", "title": {"es": ..., "en": ...}, "summary": {...}, "route": "...",
  "thumbnail": "/static/...", "date": "DD-MM-YYYY" }`
- Posts are sorted newest first when loaded; the home page shows the first three
- A post with a `route` renders `templates/blog/{route}/{lang}.html`, falling back to the default
  language when a translation is missing
- Content is cached and reloaded when the file changes, so editing a post during `cargo run` still
  shows up immediately

### Response Handling
- Return `pages::render(...)` for pages; it wraps the HTML in `HtmlWithLang`, which sets the
  language cookie and `Content-Language`
- Use `pages::not_found(&state, lang)` and `pages::server_error(&state, lang)` for error pages
- Never surface an internal error message to the client - log it and return a generic page

### Rate Limiting
- The contact form allows one submission per client per `CONTACT_RATE_LIMIT_SECS` (default 5 min)
- The client is identified by `client_ip`, not by the socket address, because the app runs behind
  Caddy
- A rejected submission answers `429` with a `Retry-After` header

## Notes

- Cargo.toml shows edition = "2024" (latest Rust edition)
- Static files are served by `src/static_files.rs`, not by a directory server
- Blog posts, post bodies and translations are cached in memory; only the first request after a file
  changes touches the disk
- Contact form includes reCAPTCHA validation for spam protection, and refuses to accept submissions
  when no secret is configured unless `RECAPTCHA_DISABLED=true`
