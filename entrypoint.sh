#!/usr/bin/env bash
set -euo pipefail

cargo run -p hrt-server --release & cargo leptos serve --release
wait -n
