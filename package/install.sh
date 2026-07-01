#!/usr/bin/env bash
set -e

APP_ID="dev.codethings.schero"

echo "Instalando Screenshot Hero..."

# Binário
install -Dm755 \
    bin/screenshot-hero \
    /usr/local/bin/screenshot-hero

# Desktop
install -Dm644 \
    "${APP_ID}.desktop" \
    "/usr/local/share/applications/${APP_ID}.desktop"

# Ícones (todos os tamanhos)
cp -r icons/hicolor/* /usr/local/share/icons/hicolor/

# Atualiza os caches (se disponíveis)
gtk-update-icon-cache -q -t -f /usr/local/share/icons/hicolor 2>/dev/null || true
update-desktop-database /usr/local/share/applications 2>/dev/null || true

echo "Instalação concluída."