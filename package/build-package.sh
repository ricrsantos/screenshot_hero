#!/usr/bin/env bash
set -e

APP_ID="dev.codethings.schero"
VERSION=$(git describe --tags --always 2>/dev/null || echo "dev")

PKG_DIR="dist/${APP_ID}-${VERSION}"

rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR"

cp package/install.sh "$PKG_DIR/"
cp package/uninstall.sh "$PKG_DIR/"
cp package/${APP_ID}.desktop "$PKG_DIR/"

mkdir -p "$PKG_DIR/bin"
cp target/release/screenshot-hero "$PKG_DIR/bin/"

mkdir -p "$PKG_DIR/icons"
cp -r data/icons/hicolor "$PKG_DIR/icons/"

tar -czf "dist/${APP_ID}-${VERSION}-linux-x86_64.tar.gz" \
    -C dist "$(basename "$PKG_DIR")"

rm -rf "$PKG_DIR"

echo "Pacote gerado em dist/"