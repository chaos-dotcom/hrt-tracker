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

COPY . .

RUN cargo build -p hrt-web -p hrt-server -p hrt-shared --release
RUN cargo leptos build --release

ENV HRT_WEB_ADDR=0.0.0.0:4100

EXPOSE 4100
RUN chmod +x ./entrypoint.sh
ENTRYPOINT ["./entrypoint.sh"]

CMD ["cargo leptos serve --release"]