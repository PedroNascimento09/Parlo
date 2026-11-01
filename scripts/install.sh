#!/usr/bin/env bash
set -euo pipefail

platform="$(uname -s)"
machine="$(uname -m)"

case "$platform" in
    Darwin)
        asset="parlo-macos-aarch64.tar.gz"
        dest="${PARLO_DIR:-$HOME/Applications/Parlo}"
        bin_dir="$HOME/.local/bin"
        ;;
    Linux)
        asset="parlo-linux-x86_64.tar.gz"
        dest="${PARLO_DIR:-$HOME/.local/opt/parlo}"
        bin_dir="$HOME/.local/bin"
        ;;
    *)
        echo "This installer handles macOS and Linux. On Windows run scripts\\install.ps1 in PowerShell." >&2
        exit 1
        ;;
esac

if [ "$platform" = "Darwin" ] && [ "$machine" != "arm64" ]; then
    echo "The published macOS build is for Apple silicon. Install from source with scripts/parlo-app.sh." >&2
    exit 1
fi

command -v curl >/dev/null || { echo "curl is needed to download Parlo." >&2; exit 1; }
command -v tar >/dev/null || { echo "tar is needed to unpack Parlo." >&2; exit 1; }

url="https://github.com/PedroNascimento09/Parlo/releases/latest/download/$asset"
archive="${TMPDIR:-/tmp}/$asset"

echo "Downloading $asset..."
curl -fL --retry 2 --progress-bar "$url" -o "$archive"

rm -rf "$dest"
mkdir -p "$dest"
tar -xzf "$archive" -C "$dest"
rm -f "$archive"

app="$(find "$dest" -maxdepth 2 -name parlo-app -type f | head -n1)"
if [ -z "$app" ]; then
    echo "The archive did not contain parlo-app. Nothing is usable in $dest." >&2
    exit 1
fi

chmod +x "$app"
found_cli="$(find "$dest" -maxdepth 2 -name parlo -type f | head -n1)"
if [ -n "$found_cli" ]; then
    chmod +x "$found_cli"
fi

mkdir -p "$bin_dir"
ln -sf "$app" "$bin_dir/parlo-app"

echo "Parlo is installed in $dest"
echo "Open it with parlo-app, or from the launcher if $bin_dir is on your PATH."
echo "The first run downloads about 800 MB of models, once."
