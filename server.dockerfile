FROM rust:1.92-bookworm

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    clang \
    curl \
    git \
    libssl-dev \
    libfontconfig \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock cargo-leptos.toml ./
COPY crates ./crates
COPY static ./static
RUN cargo build --release
