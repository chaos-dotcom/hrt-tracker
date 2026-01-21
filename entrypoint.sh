#!/usr/bin/env bash
set -euo pipefail

cargo leptos serve --release
wait -n
