# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based web application built with the Axum framework. It serves as a personal/professional website for Mechardo Labs, featuring a blog, contact form, product pages (DS2000), and language support (English/Spanish).

## Build and Run Commands

### Development
- `cargo run` - Start the development server (runs on port 3000)
- `make run` - Alias for `cargo run`
- `make watch` - Run server with hot reload for Rust, HTML, and CSS changes (requires `watchexec`)
- `npm run build:css` - Build Tailwind CSS
- `npm run watch:css` - Watch and rebuild Tailwind CSS
- `cargo test` - Run unit tests

### Docker
- `make build-image` - Build Docker image locally
- `make run-image` - Build and run Docker container locally (port 3000)
- `make stop-image` - Stop and remove local Docker container
- `make run-prod` - Run production stack with docker-compose (includes Caddy reverse proxy)
- `make stop-prod` - Stop production containers

## High-Level Architecture

### Request Flow
1. All routes are prefixed with `/{lang}` where lang is "en" or "es"
2. Root "/" redirects to the detected language based on:
   - Cookie preference (if set)
   - Accept-Language HTTP header (parsed in `src/language_detection.rs`)
   - Default: Spanish
3. Language preference is stored in a cookie with 1-year expiration

### Core Modules

**`src/main.rs`** - Server initialization and routing
- Initializes Axum router with all endpoints
- Sets up Tera templating engine with custom `date_format` filter
- Configures rate limiting state for contact form
- Applies cookie middleware layer

**`src/language.rs`** - Language enum (En/Es)
- Provides `Language::from_str()` and `Language::as_str()` for conversions
- Default language is Spanish

**`src/language_detection.rs`** - HTTP Accept-Language header parsing
- Parses quality values (q-values) from Accept-Language header
- Returns best matching language for current request
- Contains unit tests for various Accept-Language formats

**`src/date_format.rs`** - Tera filter for date formatting
- Used in templates with `{{ date|date_format }}` syntax
- Formats dates with Spanish month names

**`src/routes/`** - Route handlers
- `home.rs` - Home page with 3 most recent blog posts
- `blog.rs` - Blog listing and individual blog post pages
- `contact.rs` - Contact form with rate limiting and reCAPTCHA
- `ds2000.rs` - Product pages with terms and privacy policy
- `me.rs` - About/profile page

**`src/data/blog_data.rs`** - Data loading
- Loads blog posts from `data/blog_posts.json`
- File is read synchronously during request handling

**`src/models/blog_post.rs`** - Data model for blog posts
- Includes fields: title, date, content, author, etc.
- Deserialized from JSON

### Template System
- Templates use Tera engine, located in `templates/` directory
- All templates support both English and Spanish (lang variable passed in context)
- Date formatting uses `date_format` filter for Spanish month names

### Styling
- Tailwind CSS v4 (alpha version)
- Source: `static/tailwind.css`
- Output: `static/style.css`
- Build via `npm run build:css` or `make build-css`

### Deployment
- Docker multi-stage build (see `Dockerfile`)
- Production uses Caddy reverse proxy (see `docker-compose.yml` and `Caddyfile`)
- Environment variables stored in `secrets/` directory

## Key Development Patterns

### Adding a New Route
1. Create handler in `src/routes/[name].rs`
2. Add module declaration in `src/routes/mod.rs`
3. Register in router within `src/main.rs` with pattern `/{lang}/path`
4. Handler should accept `Path(lang)` parameter and return templated HTML
5. Use language parameter to set context for Tera rendering

### Blog Posts
- Stored as JSON in `data/blog_posts.json`
- Structure: `{ "title": "...", "date": "YYYY-MM-DD", "content": "...", ... }`
- Home page sorts by date descending and displays top 3
- Individual posts are accessed via blog ID

### Language Implementation
- Every page route must accept `Path(lang)` and validate it
- Set language cookie on each request (1-year expiration)
- Pass language string to Tera context for template conditionals
- Use `Language::from_str()` to convert URL param to enum

### Rate Limiting
- Contact form uses in-memory HashMap-based rate limiting
- State is shared via `Extension` middleware
- Stored in `src/routes/contact.rs`

## Notes

- Cargo.toml shows edition = "2024" (latest Rust edition)
- Static files are served via handler, not direct serving
- All routes redirect to language-prefixed paths
- Blog posts are loaded from filesystem during each request (not cached)
- Contact form includes reCAPTCHA validation for spam protection
