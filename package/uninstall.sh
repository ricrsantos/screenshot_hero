#!/usr/bin/env bash
set -e

APP_ID="dev.codethings.schero"

echo "Removendo Screenshot Hero..."

rm -f /usr/local/bin/screenshot-hero
rm -f "/usr/local/share/applications/${APP_ID}.desktop"

for size in 16x16 24x24 32x32 48x48 64x64 128x128 256x256 scalable; do
    rm -f "/usr/local/share/icons/hicolor/${size}/apps/${APP_ID}.png"
    rm -f "/usr/local/share/icons/hicolor/${size}/apps/${APP_ID}.svg"
done

gtk-update-icon-cache -q -t -f /usr/local/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/local/share/applications 2>/dev/null || true

echo "Remoção concluída."