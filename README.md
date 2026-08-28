# mechardo3d

Web app for [mechardo3d.xyz](https://mechardo3d.xyz) - a bilingual (es/en) site
with a blog, a contact form and the DS2000 product pages. Built with Rust,
[Axum](https://github.com/tokio-rs/axum) and [Tera](https://keats.github.io/tera/),
served behind Caddy in production.

## Running it

```bash
cargo run              # http://localhost:3000
npm install            # once, for Tailwind
npm run build:css      # static/tailwind.css -> static/style.css
make watch             # server + CSS with hot reload (needs watchexec)
```

Useful targets: `make test`, `make lint` (fmt + clippy), `make check` (both),
`make run-image` / `make stop-image` for a local container, `make run-prod` for
the full docker-compose stack.

The contact form verifies reCAPTCHA before storing a message. For local work
either export a real secret or turn verification off:

```bash
RECAPTCHA_DISABLED=true cargo run
```

## Configuration

Everything is read from the environment once at startup; every value has a
default, so `cargo run` works with nothing set.

| Variable | Default | What it does |
| --- | --- | --- |
| `BIND_ADDR` | `0.0.0.0` | Address the server binds to |
| `PORT` | `3000` | Port the server binds to |
| `BASE_URL` | `https://mechardo3d.xyz` | Public origin used for canonical URLs, JSON-LD and the sitemap |
| `RUST_LOG` | `info` | Log level (`trace`/`debug`/`info`/`warn`/`error`, `target=level` accepted) |
| `COOKIE_SECURE` | `false` | Marks the language cookie `Secure` (on in production) |
| `TRUST_PROXY_HEADERS` | `true` | Read the client IP from `X-Forwarded-For` / `X-Real-IP` |
| `HSTS_ENABLED` | `true` | Send `Strict-Transport-Security` |
| `CONTENT_SECURITY_POLICY` | unset | Sent verbatim as the CSP header when set |
| `RECAPTCHA_SITE_KEY` | public site key | Site key handed to the contact page |
| `RECAPTCHA_SECRET_KEY` | from `secrets/recaptcha.env` | Secret used to verify submissions |
| `RECAPTCHA_SECRET_FILE` | `secrets/recaptcha.env` | Fallback file for the secret |
| `RECAPTCHA_MIN_SCORE` | `0.6` | Minimum score accepted from reCAPTCHA v3 |
| `RECAPTCHA_DISABLED` | `false` | Skips verification - local development only |
| `CONTACT_RATE_LIMIT_SECS` | `300` | Minimum delay between submissions from one client |
| `GITHUB_TOKEN` | from `secrets/github.env` | Read-only token for the resume repository. Unset hides the CV download |
| `GITHUB_SECRET_FILE` | `secrets/github.env` | Fallback file for the token |
| `RESUME_REPO` | `Mechanix97/Resume` | `owner/name` of the repository publishing the resume releases |
| `RESUME_ASSET_ES` | `Lucas_Rack_Software_Engineer_CV.pdf` | Release asset served to Spanish visitors |
| `RESUME_ASSET_EN` | `Lucas_Rack_Software_Engineer_Resume.pdf` | Release asset served to English visitors |
| `RESUME_CACHE_SECS` | `3600` | How long a downloaded PDF is reused before GitHub is asked again |
| `MAX_MESSAGE_CHARS` | `5000` | Longest accepted contact message |
| `DATA_DIR` | `data` | Blog posts and stored messages |
| `STATIC_DIR` | `static` | Served under `/static` |
| `TEMPLATES_DIR` | `templates` | Tera templates and blog bodies |
| `TRANSLATIONS_DIR` | `translations` | Translation modules |

### Secrets

`secrets/` is not tracked. The production stack reads these files, which have to
exist on the server before `docker compose up`:

| File | Holds | Used by |
| --- | --- | --- |
| `secrets/recaptcha.env` | `RECAPTCHA_SECRET_KEY` | Contact form verification |
| `secrets/github.env` | `GITHUB_TOKEN` | The CV download on `/me` |
| `secrets/plausible-db.env` | `POSTGRES_PASSWORD` | Plausible's Postgres |
| `secrets/plausible.env` | `DATABASE_URL` plus Plausible's own settings | Plausible |
| `secrets/mail.env` | `SENDER_EMAIL`, `SENDER_PASSWORD` | `tools/send_email.py` |

`DATABASE_URL` embeds the same password, so the two Plausible files have to
agree:

```
# secrets/plausible-db.env
POSTGRES_PASSWORD=<password>

# secrets/plausible.env
DATABASE_URL=postgres://postgres:<password>@plausible_db:5432/plausible
```

Changing `POSTGRES_PASSWORD` on its own does **not** change the password of a
database that already exists: Postgres only reads that variable when it
initialises an empty data directory, and `db-data` is a persistent volume. To
rotate the password without dropping the analytics history, change it in the
running database first, then update both files:

```sh
docker compose exec plausible_db \
  psql -U postgres -c "ALTER USER postgres WITH PASSWORD '<new password>';"
```

## Routes

| Route | Notes |
| --- | --- |
| `/` | Redirects to the visitor's language |
| `/{lang}` | Home (`es`, `en`) |
| `/{lang}/me`, `/{lang}/blog`, `/{lang}/blog/{id}` | Pages |
| `/{lang}/cv` | The resume PDF, proxied from the latest release of `RESUME_REPO` |
| `/{lang}/contact` | `GET` renders the form, `POST` accepts it |
| `/{lang}/contact_success` | Confirmation |
| `/{lang}/ds2000`, `/{lang}/ds2000/terms-of-service`, `/{lang}/ds2000/privacy-policy` | Product |
| `/health` | Liveness probe used by Docker |
| `/robots.txt`, `/sitemap.xml` | Generated from the routing table and the blog data |
| `/static/{*path}`, `/favicon.ico` | Static assets, with `ETag` and `Cache-Control` |

Unknown paths render a localized 404; paths without a language prefix are
redirected to the visitor's language.

## Layout

```
src/
  main.rs            router, startup
  config.rs          environment-driven configuration
  state.rs           shared state (templates, translations, blog, HTTP client)
  extract.rs         language extractor and URL helpers
  middleware.rs      security headers, request logging
  pages.rs           shared page context, rendering, 404/500
  responses.rs       HTML response + language cookie
  static_files.rs    static assets, path checks, caching
  sitemap.rs         sitemap.xml and robots.txt
  rate_limit.rs      contact form rate limiting
  client_ip.rs       proxy-aware client address
  routes/            one module per page
  data/              blog store (cached) and message store
  models/            blog post model and view
translations/{lang}/*.json   loaded and merged automatically
templates/           Tera templates, blog bodies under templates/blog/
```

Contributor notes and conventions live in [CLAUDE.md](CLAUDE.md).
