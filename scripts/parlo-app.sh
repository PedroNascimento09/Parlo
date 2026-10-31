#!/bin/sh
set -e
cd "$(dirname "$0")/.."
if [ ! -x target/release/parlo-app ]; then
    echo "Building Parlo, this takes a few minutes the first time..."
    cargo build --release -p parlo-cli -p parlo-app
fi
exec target/release/parlo-app "$@"
