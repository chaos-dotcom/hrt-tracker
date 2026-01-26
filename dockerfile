FROM rust:1.92-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    clang \
    curl \
    git \
    libfontconfig1-dev \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*


RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

RUN rustup target add wasm32-unknown-unknown
RUN cargo binstall cargo-leptos
RUN cargo binstall wasm-bindgen-cli

COPY Cargo.toml Cargo.lock cargo-leptos.toml ./
COPY crates ./crates
COPY static ./static

RUN cargo build -p hrt-server -p hrt-web --release
RUN cargo leptos build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libfontconfig1 \
    libssl3 \
    supervisor \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hrt-server /app/hrt-server
COPY --from=builder /app/target/release/hrt-web /app/hrt-web
COPY --from=builder /app/target/site /app/target/site
COPY supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# Environment variables will be set via docker-compose.yml
# These are defaults for development
ENV HRT_WEB_ADDR=0.0.0.0:4100
ENV HRT_SERVER_ADDR=127.0.0.1:4200
ENV HRT_ALLOWED_ORIGINS=https://hrt.example.com,http://127.0.0.1:4100

EXPOSE 4100
CMD ["/usr/bin/supervisord", "-c", "/etc/supervisor/conf.d/supervisord.conf"]
