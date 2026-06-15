# Behavior Settings Extend - Design

**Spec:** `.specs/features/behavior-settings-extend/spec.md`  
**Status:** Implemented

---

## Architecture

Feature usa o padrão já existente:

1. **Schema** (`data/com.screenshot_hero.ScreenshotHero.gschema.xml`)
2. **Wrapper tipado** (`src/settings/mod.rs`)
3. **UI de preferências** (`src/ui/preferences/mod.rs`)
4. **Consumo runtime** (`src/application.rs`, `src/ui/window/imp.rs`)

---

## New Settings Keys

- `post-capture-editing-disabled` (`b`, default `false`)
- `post-capture-editing-temporary-enabled` (`b`, default `false`)
- `post-capture-editing-temporary-minutes` (`u`, default `1`)
- `post-capture-editing-temporary-seconds` (`u`, default `0`)
- `post-capture-editing-temporary-started-at` (`x`, default `0`)
- `exit-after-paste` (`b`, default `true`)
- `open-new-window-on-capture` (`b`, default `false`)

---

## Runtime Behavior

### `--capture` flow (`src/application.rs`)

- Antes de processar resultado da captura, app resolve política efetiva via `AppSettings`.
- `is_post_capture_editing_effectively_disabled()` combina:
  - toggle permanente;
  - toggle temporário + tempo decorrido;
  - auto-reset do modo temporário ao expirar.
- Se edição estiver efetivamente desabilitada:
  - invoca portal de captura;
  - encerra app sem abrir editor.

### Exit After Paste monitor

- Em sessão `--capture` com editor aberto e `exit-after-paste = true`:
  - app copia captura para clipboard;
  - registra monitor por `clipboard.connect_changed`;
  - ao detectar mudança no clipboard, chama `app.quit()`.

### Reuse vs New Window (`src/ui/window/imp.rs` + `src/application.rs`)

- `open-new-window-on-capture = false`:
  - captura nova substitui imagem da janela atual.
- `open-new-window-on-capture = true`:
  - captura nova abre janela adicional.
- Em `application.rs`, fluxo de `--capture` reaproveita janela existente quando permitido.

---

## Validation Note (Clipboard Change Detection)

A detecção de descarregamento/troca do clipboard foi validada tecnicamente usando o sinal nativo `gdk::Clipboard::connect_changed`, que notifica mudanças de proprietário/conteúdo no clipboard do display atual.
