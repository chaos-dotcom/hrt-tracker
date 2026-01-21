FROM rust:1.92-bookworm AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    clang \
    curl \
    git \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos --locked
RUN cargo install wasm-bindgen-cli --locked

COPY crates crates

COPY . .

RUN cargo leptos build --release
RUN cargo build -p hrt-web -p hrt-server --release

FROM debian:bookworm-slim AS runner

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/hrt-web /app/hrt-web
COPY --from=builder /app/target/release/hrt-server /app/hrt-server
COPY --from=builder /app/target/site /app/target/site
COPY entrypoint.sh /app/entrypoint.sh

RUN chmod +x /app/entrypoint.sh

ENV HRT_WEB_ADDR=0.0.0.0:4100
ENV HRT_SERVER_ADDR=0.0.0.0:4200

EXPOSE 4100 4200

ENTRYPOINT ["tini", "--"]
CMD ["/app/entrypoint.sh"]
