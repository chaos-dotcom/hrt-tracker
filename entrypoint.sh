#!/usr/bin/env bash
set -euo pipefail

mkdir -p /app/data /app/data/dosage-photos

if [ ! -f /app/data/hrt-data.json ]; then
  echo "{}" > /app/data/hrt-data.json
fi

if [ ! -f /app/data/hrt-settings.yaml ]; then
  echo "{}" > /app/data/hrt-settings.yaml
fi

/app/hrt-server &
/app/hrt-web &

wait -n
