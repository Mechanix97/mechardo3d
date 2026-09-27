FROM rust:1.88 AS chef

RUN apt-get update && apt-get install -y \
    build-essential \
    libclang-dev \
    libc6 \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

WORKDIR /mechardo3d

FROM node:20 AS tailwind

WORKDIR /mechardo3d
COPY package.json package-lock.json ./
RUN npm ci
COPY static/tailwind.css ./static/
COPY templates ./templates
# The JS builds class names too (language picker), so Tailwind has to scan it.
COPY static/js ./static/js
RUN npx tailwindcss -i static/tailwind.css -o static/style.css

FROM chef AS planner
COPY src ./src
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /mechardo3d/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ARG BUILD_FLAGS=""
COPY src ./src
COPY Cargo.toml Cargo.lock ./
COPY templates ./templates
COPY data ./data
COPY static ./static
COPY translations ./translations
COPY --from=tailwind /mechardo3d/static/style.css ./static/style.css
RUN cargo build --release $BUILD_FLAGS

FROM debian:bookworm-slim
WORKDIR /usr/local/bin

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -r -u 10001 app

COPY --from=builder --chown=app:app /mechardo3d/data ./data
COPY --from=builder --chown=app:app /mechardo3d/target/release/mechardo3d .
COPY --from=builder --chown=app:app /mechardo3d/templates ./templates
COPY --from=builder --chown=app:app /mechardo3d/static ./static
COPY --from=builder --chown=app:app /mechardo3d/translations ./translations

USER app

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -fsS http://127.0.0.1:3000/health || exit 1

ENTRYPOINT ["./mechardo3d"]
