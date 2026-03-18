FROM rust:1.88 AS chef

RUN apt-get update && apt-get install -y \
    build-essential \
    libclang-dev \
    libc6 \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

WORKDIR /mechardo3d

FROM node:20 AS tailwind

WORKDIR /mechardo3d
COPY package.json package-lock.json ./
RUN npm install
COPY static/tailwind.css ./static/
COPY templates ./templates
COPY tailwind.config.js ./
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
COPY secrets ./secrets
COPY static ./static
COPY translations ./translations
COPY --from=tailwind /mechardo3d/static/style.css ./static/style.css
RUN cargo build --release $BUILD_FLAGS

FROM ubuntu:24.04
WORKDIR /usr/local/bin

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    iputils-ping \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /mechardo3d/data ./data
COPY --from=builder /mechardo3d/secrets ./secrets
COPY --from=builder /mechardo3d/target/release/mechardo3d .
COPY --from=builder /mechardo3d/templates ./templates
COPY --from=builder /mechardo3d/static ./static
COPY --from=builder /mechardo3d/translations ./translations

EXPOSE 3000
ENTRYPOINT ["./mechardo3d"]
