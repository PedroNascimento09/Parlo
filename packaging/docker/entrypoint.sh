#!/bin/sh
set -eu

port="${PARLO_PORT:-8787}"
pin="${PARLO_PIN:-}"

case "$pin" in
  *[!0-9]*)
    echo "PARLO_PIN must be digits only" >&2
    exit 1
    ;;
esac

if [ "${#pin}" -lt 4 ] || [ "${#pin}" -gt 8 ]; then
  echo "PARLO_PIN must be four to eight digits, the length parlo accepts" >&2
  exit 1
fi

config="${PARLO_CONFIG:-/data/config.json}"
mkdir -p "$(dirname "$config")"
umask 077
printf '{"port": %s, "share": true, "share_pin": "%s", "share_port": %s}\n' \
  "$port" "$pin" "$port" > "$config"

exec /opt/parlo/parlo serve --port "$port"
