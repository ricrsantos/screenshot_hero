#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

echo "Cleaning Flatpak artifacts..."

rm -rf \
    "$PROJECT_ROOT/flatpak_artifacts/build-dir" \
    "$PROJECT_ROOT/flatpak_artifacts/repo" \
    "$PROJECT_ROOT/flatpak_artifacts/release"

echo "✅ Done."