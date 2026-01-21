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

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked
RUN cargo install wasm-bindgen-cli --locked

COPY Cargo.toml Cargo.lock cargo-leptos.toml ./
COPY crates ./crates
COPY static ./static
COPY ./entrypoint.sh ./


RUN cargo build -p hrt-server -p hrt-shared --release
RUN cargo leptos build --release

# Environment variables will be set via docker-compose.yml
# These are defaults for development
ENV HRT_WEB_ADDR=0.0.0.0:4100
ENV HRT_SERVER_ADDR=127.0.0.1:4200
ENV HRT_ALLOWED_ORIGINS=https://hrt.example.com,http://127.0.0.1:4100

EXPOSE 4100
RUN chmod +x ./entrypoint.sh
ENTRYPOINT ["./entrypoint.sh"]