#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

APP_ID="com.screenshot_hero.ScreenshotHero"

REPO_DIR="$PROJECT_ROOT/flatpak_artifacts/repo"
RELEASE_DIR="$PROJECT_ROOT/flatpak_artifacts/release"

mkdir -p "$RELEASE_DIR"

echo "==> Creating Flatpak bundle..."

flatpak build-bundle \
    "$REPO_DIR" \
    "$RELEASE_DIR/scHero.flatpak" \
    "$APP_ID"

echo
echo "==> Generating SHA256..."

(
    cd "$RELEASE_DIR"
    sha256sum scHero.flatpak > scHero.flatpak.sha256
)

echo
echo "Artifacts generated:"
echo
ls -lh "$RELEASE_DIR"

echo
echo "✅ Package created successfully."