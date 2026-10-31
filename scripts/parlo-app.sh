#!/bin/sh
set -e
cd "$(dirname "$0")/.."
if [ ! -x target/release/parlo ]; then
    echo "Building Parlo, this takes about a minute the first time..."
    cargo build --release -p parlo-cli
fi
exec target/release/parlo app "$@"
