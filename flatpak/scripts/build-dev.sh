#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

MANIFEST="$PROJECT_ROOT/flatpak/dev.codethings.schero.yml"
BUILD_DIR="$PROJECT_ROOT/flatpak_artifacts/build-dir"

echo "==> Building Screenshot Hero (development)..."

flatpak-builder \
    --user \
    --install \
    --force-clean \
    "$BUILD_DIR" \
    "$MANIFEST"

echo
echo "✅ Development build completed."