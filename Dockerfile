FROM rust:1.87 AS chef

RUN apt-get update && apt-get install -y \
    build-essential \
    libclang-dev \
    libc6 \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef

WORKDIR /mechardo3d

FROM chef AS planner
COPY src ./src
COPY Cargo.* .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /mechardo3d/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ARG BUILD_FLAGS=""
COPY src ./src
COPY Cargo.* ./
RUN cargo build --release $BUILD_FLAGS

FROM ubuntu:24.04
WORKDIR /usr/local/bin

COPY --from=builder /mechardo3d/target/release/mechardo3d .
EXPOSE 3000
ENTRYPOINT [ "./mechardo3d" ]
