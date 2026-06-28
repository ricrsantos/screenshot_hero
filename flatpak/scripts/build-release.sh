#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MANIFEST="$PROJECT_ROOT/flatpak/dev.codethings.schero.yml"

BUILD_DIR="$PROJECT_ROOT/flatpak_artifacts/build-dir"
REPO_DIR="$PROJECT_ROOT/flatpak_artifacts/repo"

echo "==> Building Screenshot Hero (release)..."

flatpak-builder \
    --repo="$REPO_DIR" \
    --force-clean \
    "$BUILD_DIR" \
    "$MANIFEST"

echo
echo "✅ Release repository created."

echo
echo "Run:"
echo "    ./flatpak/scripts/package.sh"