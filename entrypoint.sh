#!/usr/bin/env bash
set -euo pipefail

# Start backend server on internal address only
cargo run -p hrt-server --release &

# Start frontend server with API proxy on external address
cargo leptos serve --release &

wait -n
