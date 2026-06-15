# Export and Clipboard Tasks

**Design**: `.specs/features/export-and-clipboard/design.md`
**Status**: Implemented in codebase (task checklist pending backfill)

---

## Execution Plan

### Phase 1: Pure Rust Export Module (Parallel)

Tasks T1–T4 are fully independent new files. T1 and T5 both touch `src/canvas/mod.rs` — T5 is placed in Phase 2 so they don't conflict.

```
T1 [P] ──┐
T2 [P] ──┤
T3 [P] ──┼──→ T5 → T6 → T7 → T8 → T9
T4 [P] ──┘
```

### Phase 2: Module Wiring + Canvas Extension (Sequential)

```
T5 → T6
```

### Phase 3: UI Integration (Sequential — all touch `window/imp.rs`)

```
T6 → T7 → T8 → T9
```

---

## Task Breakdown

### T1: Create `src/export/renderer.rs` — off-screen render to Pixbuf [P]

**What**: New file implementing off-screen Cairo rendering that composites source image + annotations into a `Pixbuf` at native resolution. Also changes `mod renderer;` → `pub(crate) mod renderer;` in `src/canvas/mod.rs` to expose `draw_all` crate-wide.

**Where**:
- `src/export/renderer.rs` (new)
- `src/canvas/mod.rs` (1-char change: `mod` → `pub(crate) mod`)

**Depends on**: None

**Reuses**: `crate::canvas::renderer::draw_all` (ADR-003: canvas == export)

**Requirement**: EXPRT-03, EXPRT-05

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `pub fn render_to_pixbuf(source: &Pixbuf, annotations: &[Annotation]) -> Option<Pixbuf>` is implemented
- [x] Creates `cairo::ImageSurface` at source image dimensions with `Format::ARgb32`
- [x] Draws source pixbuf at (0, 0) then calls `renderer::draw_all` at zoom=1.0, pan=(0,0), selected_id=None
- [x] `fn surface_to_pixbuf(surface: &cairo::ImageSurface) -> Option<Pixbuf>` converts ARGB32 (premultiplied) → RGBA (straight alpha) via byte-swap + un-premultiply
- [x] `pub(crate) mod renderer;` in `src/canvas/mod.rs` (was `mod renderer;`)
- [x] Unit test: `test_render_to_pixbuf_dimensions` — creates 100×80 Pixbuf, renders with empty annotations, asserts output is 100×80
- [x] Gate check passes: `cargo test --lib`
- [x] Test count: at least 1 new test passes

**Tests**: unit
**Gate**: quick

---

### T2: Create `src/export/auto_export.rs` — export path computation [P]

**What**: New file with pure path logic for auto-export destination. Given a source image path and a suffix, computes the path where the auto-exported PNG should be saved.

**Where**: `src/export/auto_export.rs` (new)

**Depends on**: None

**Reuses**: `std::path` only

**Requirement**: EXPRT-13, EXPRT-14

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `pub fn build_auto_export_path(source: &Path, suffix: &str) -> PathBuf` is implemented
- [x] Stem extraction: `source.file_stem()` used to build `{stem}{suffix}.png`
- [x] Result placed in the same directory as source (`source.with_file_name(new_name)`)
- [x] Unit test `test_build_auto_export_path_png`: `/home/u/Screenshots/shot.png` + `"_shero"` → `/home/u/Screenshots/shot_shero.png`
- [x] Unit test `test_build_auto_export_path_jpeg_source`: source is `.jpg` → output is `.png` (always PNG output)
- [x] Unit test `test_build_auto_export_path_no_extension`: source has no extension → `shot_shero.png`
- [x] Gate check passes: `cargo test --lib`
- [x] Test count: at least 3 new tests pass

**Tests**: unit
**Gate**: quick

---

### T3: Create `src/export/exporter.rs` — file write (PNG + JPEG) [P]

**What**: New file that writes a `Pixbuf` to disk as PNG or JPEG using `gdk_pixbuf::Pixbuf::savev`. Defines `ExportError`.

**Where**: `src/export/exporter.rs` (new)

**Depends on**: None

**Reuses**: `gdk_pixbuf::Pixbuf::savev` built-in format support

**Requirement**: EXPRT-01, EXPRT-02

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `pub enum ExportError { SaveFailed(String) }` defined with `Display` impl
- [x] `pub fn export_png(pixbuf: &Pixbuf, path: &Path) -> Result<(), ExportError>` uses `pixbuf.savev(path, "png", &[], &[])`
- [x] `pub fn export_jpeg(pixbuf: &Pixbuf, path: &Path) -> Result<(), ExportError>` uses `pixbuf.savev(path, "jpeg", &["quality"], &["90"])`
- [x] Both functions convert `glib::Error` from `savev` into `ExportError::SaveFailed(err.to_string())`
- [x] Unit test `test_export_png_writes_file`: create small Pixbuf, call `export_png`, assert file exists and `len > 0`, clean up
- [x] Unit test `test_export_jpeg_writes_file`: same for JPEG
- [x] Gate check passes: `cargo test --lib`
- [x] Test count: at least 2 new tests pass

**Tests**: unit
**Gate**: quick

**Note**: ⚠️ Verify JPEG quality option key before implementing. Likely `"quality"` but confirm with gtk4-rs docs or crates.io. Fallback: use `&[]` for default quality if the option key is wrong — add to STATE.md todos.

---

### T4: Create `src/export/clipboard.rs` — clipboard write [P]

**What**: New file that copies a rendered `Pixbuf` to the system clipboard using GDK4's clipboard API.

**Where**: `src/export/clipboard.rs` (new)

**Depends on**: None

**Reuses**: `gtk::gdk::Texture::for_pixbuf`, `gdk::Display::clipboard`, `gdk::Clipboard::set_texture`

**Requirement**: EXPRT-04, EXPRT-05

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `pub fn copy_to_clipboard(display: &gdk::Display, pixbuf: &Pixbuf)` implemented
- [x] Converts `Pixbuf` → `gdk::Texture` via `gdk::Texture::for_pixbuf(pixbuf)`
- [x] Calls `display.clipboard().set_texture(&texture)`
- [x] No panics — errors are logged via `log::error!` if any step returns an error
- [x] Gate check passes: `cargo build`
- [x] No compiler errors or warnings

**Tests**: none (requires GDK display — manual validation only)
**Gate**: build

**Note**: ⚠️ Verify `gdk::Clipboard::set_texture` method signature before implementing. In gtk4-rs 0.9 this may be `set_content(Some(&ContentProvider::for_value(&texture.to_value())))` if `set_texture` is unavailable. Add to STATE.md todos.

---

### T5: Create `src/export/mod.rs` and register module in `src/lib.rs`

**What**: Wires the four export sub-modules into the crate. Creates `src/export/mod.rs` that re-exports the public API. Adds `pub mod export;` to `src/lib.rs`.

**Where**:
- `src/export/mod.rs` (new)
- `src/lib.rs` (add `pub mod export;`)

**Depends on**: T1, T2, T3, T4

**Reuses**: pattern from `src/persistence/mod.rs`

**Requirement**: all EXPRT-*

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `src/export/mod.rs` declares `pub mod renderer; pub mod auto_export; pub mod exporter; mod clipboard;` plus `pub use` of public types
- [x] `ExportError` is re-exported as `crate::export::ExportError`
- [x] `src/lib.rs` has `pub mod export;`
- [x] Gate check passes: `cargo build`
- [x] No compiler errors or warnings

**Tests**: none
**Gate**: build

---

### T6: Add `Canvas::source_pixbuf()` to `src/canvas/mod.rs`

**What**: Adds a new method to `Canvas` that returns a clone of the loaded source `Pixbuf`, enabling the window to feed the pixbuf to the export renderer without coupling export to Canvas internals.

**Where**: `src/canvas/mod.rs` (add method to `impl Canvas`)

**Depends on**: T5 (ensures the crate builds cleanly before modifying canvas)

**Reuses**: existing `imp.image: RefCell<Option<ImageData>>` and `ImageData::pixbuf()`

**Requirement**: EXPRT-01, EXPRT-02, EXPRT-04

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `pub fn source_pixbuf(&self) -> Option<gdk_pixbuf::Pixbuf>` added to `impl Canvas`
- [x] Returns `Some(img.pixbuf().clone())` when image is loaded, `None` otherwise
- [x] Gate check passes: `cargo build`
- [x] No compiler errors or warnings

**Tests**: none (UI layer — `Canvas` is a GTK widget)
**Gate**: build

---

### T7: Add manual export and clipboard GActions to `MainWindow`

**What**: Wires three new GActions into `MainWindow::constructed()` — `win.export-png`, `win.export-jpeg`, `win.copy-to-clipboard` — and adds corresponding header bar buttons. Actions are disabled when no image is loaded.

**Where**: `src/ui/window/imp.rs` (modify `constructed()`)

**Depends on**: T6

**Reuses**:
- `show_save_project_dialog` pattern for file dialog (adapt for image formats)
- `update_save_project_enabled` pattern for action state
- `build_project_snapshot` style for collecting Canvas state

**Requirement**: EXPRT-01, EXPRT-02, EXPRT-03, EXPRT-04, EXPRT-05

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `win.export-png` GAction: opens `gtk::FileDialog` (save, `.png` filter) → calls `export::renderer::render_to_pixbuf` → calls `export::exporter::export_png` → shows `show_error_dialog` on `Err`
- [x] `win.export-jpeg` GAction: same flow with `.jpg`/`.jpeg` filter and `export_jpeg`
- [x] `win.copy-to-clipboard` GAction: `canvas.source_pixbuf()` + `canvas.all_annotations()` → `render_to_pixbuf` → `clipboard::copy_to_clipboard(display, &pixbuf)` (synchronous, no dialog)
- [x] All three actions start disabled; enabled when `canvas.source_image_path().is_some()` — use existing `update_save_project_enabled` extension point
- [x] Header bar: "Export PNG", "Export JPEG", "Copy" buttons added using same builder pattern as existing buttons
- [x] `glib::spawn_future_local` used for export dialogs (async) per project pattern
- [x] Gate check passes: `cargo build`
- [x] No compiler errors or warnings

**Tests**: none (UI)
**Gate**: build

---

### T8: Add auto-clipboard debounce to `MainWindow`

**What**: Extends `MainWindow` struct with debounce state and `auto_clipboard_enabled` flag. Integrates a 300ms debounced clipboard update into the existing `on_annotation_changed` callback.

**Where**: `src/ui/window/imp.rs`

**Depends on**: T7 (sequential — same file, same callback)

**Reuses**: existing `on_annotation_changed` closure; `glib::timeout_add_local_once` (glib 0.20)

**Requirement**: EXPRT-06, EXPRT-07, EXPRT-08, EXPRT-09, EXPRT-10

> Note (2026-06-15): Part of this task was superseded by PRD-006 settings integration. Runtime behavior is implemented, but specific field-shape criteria below (`auto_clipboard_enabled: Cell<bool>`) no longer match the final architecture.

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [ ] `MainWindow` struct gains `clipboard_debounce: RefCell<Option<glib::SourceId>>` and `auto_clipboard_enabled: Cell<bool>`
- [ ] `Default` impl initializes `clipboard_debounce: RefCell::new(None)`, `auto_clipboard_enabled: Cell::new(true)`
- [ ] Inside `on_annotation_changed` callback: if `auto_clipboard_enabled` is true, cancel pending `SourceId` (if any) via `id.remove()`, then schedule new `glib::timeout_add_local_once(Duration::from_millis(300), ...)`
- [x] The debounce callback: calls `canvas.source_pixbuf()` + `canvas.all_annotations()` → `render_to_pixbuf` → `clipboard::copy_to_clipboard` — all on main thread
- [x] New `SourceId` stored in `window.imp().clipboard_debounce`
- [x] Gate check passes: `cargo build`
- [x] No compiler errors or warnings

**Tests**: none (requires GDK display + GLib main loop)
**Gate**: build

---

### T9: Add auto-export logic to `MainWindow`

**What**: Extends `MainWindow` struct with `auto_export_enabled` and `auto_export_suffix` fields. Integrates auto-export trigger into `on_annotation_changed` callback. This is the final task and integration gate.

**Where**: `src/ui/window/imp.rs`

**Depends on**: T8 (sequential — same file, same callback)

**Reuses**: existing `on_annotation_changed` closure; `export::auto_export::build_auto_export_path`; `export::exporter::export_png`

**Requirement**: EXPRT-11, EXPRT-12, EXPRT-13, EXPRT-14

> Note (2026-06-15): Part of this task was superseded by PRD-006 settings integration. Runtime behavior is implemented, but criteria that require persistent `Cell/RefCell` fields for auto-export no longer match the final architecture.

**Tools**:
- MCP: NONE
- Skill: NONE

**Done when**:

- [ ] `MainWindow` struct gains `auto_export_enabled: Cell<bool>` and `auto_export_suffix: RefCell<String>`
- [ ] `Default` impl: `auto_export_enabled: Cell::new(false)`, `auto_export_suffix: RefCell::new("_shero".to_string())`
- [x] Inside `on_annotation_changed`: if `auto_export_enabled` → get `source_image_path` from canvas → `build_auto_export_path(path, &suffix)` → `render_to_pixbuf` → `export_png` → `log::warn!` on `Err`
- [x] Auto-export is skipped silently when `canvas.source_image_path()` is `None`
- [x] No user-facing dialog or feedback for auto-export (background operation)
- [x] Gate check passes: `cargo build && cargo test --lib`
- [x] All existing tests still pass (test count does not regress)

**Tests**: none (UI — requires display for manual validation)
**Gate**: full

**Commit**: `feat(export): add export, clipboard, and auto-clipboard support`

---

## Parallel Execution Map

```
Phase 1 (Parallel):
  T1 [P] ──┐
  T2 [P] ──┤
  T3 [P] ──┼──→ (all complete)
  T4 [P] ──┘

Phase 2 (Sequential):
  T5 ──→ T6

Phase 3 (Sequential):
  T6 ──→ T7 ──→ T8 ──→ T9
```

**Why T7–T9 are sequential**: All three modify `src/ui/window/imp.rs` — specifically the `MainWindow` struct, its `Default`, and the `constructed()` method. Sub-agents editing the same file in parallel would produce conflicting diffs.

---

## Task Granularity Check

| Task | Scope | Status |
|---|---|---|
| T1: `export/renderer.rs` + canvas/mod.rs 1-char change | 1 new file + 1 minor change | ✅ Granular |
| T2: `export/auto_export.rs` | 1 new file, 1 function | ✅ Granular |
| T3: `export/exporter.rs` | 1 new file, 2 functions + error type | ✅ Granular |
| T4: `export/clipboard.rs` | 1 new file, 1 function | ✅ Granular |
| T5: `export/mod.rs` + `lib.rs` | 1 new file + 1-line addition | ✅ Granular (pure wiring) |
| T6: `Canvas::source_pixbuf()` | 1 method addition | ✅ Granular |
| T7: 3 GActions + 3 buttons in window | 1 cohesive UI block in `constructed()` | ✅ Granular (all same concern) |
| T8: Debounce fields + callback extension | 1 logical feature in 1 file | ✅ Granular |
| T9: Auto-export fields + callback extension | 1 logical feature in 1 file | ✅ Granular |

---

## Diagram-Definition Cross-Check

| Task | Depends On (body) | Diagram Shows | Status |
|---|---|---|---|
| T1 | None | Start of Phase 1 | ✅ Match |
| T2 | None | Start of Phase 1 | ✅ Match |
| T3 | None | Start of Phase 1 | ✅ Match |
| T4 | None | Start of Phase 1 | ✅ Match |
| T5 | T1, T2, T3, T4 | Phase 1 complete → T5 | ✅ Match |
| T6 | T5 | T5 → T6 | ✅ Match |
| T7 | T6 | T6 → T7 | ✅ Match |
| T8 | T7 | T7 → T8 | ✅ Match |
| T9 | T8 | T8 → T9 | ✅ Match |

---

## Test Co-location Validation

The TESTING.md matrix does not yet include `src/export/` (new module). The matrix should be extended:

| Code Layer | Location | Test Type |
|---|---|---|
| Export renderer | `src/export/renderer.rs` | unit |
| Export path logic | `src/export/auto_export.rs` | unit |
| Export file writer | `src/export/exporter.rs` | unit |
| Clipboard writer | `src/export/clipboard.rs` | none (GDK display required) |

Validation against updated matrix:

| Task | Code Layer Created/Modified | Matrix Requires | Task Says | Status |
|---|---|---|---|---|
| T1: `export/renderer.rs` | export renderer | unit | unit | ✅ OK |
| T2: `export/auto_export.rs` | export path logic | unit | unit | ✅ OK |
| T3: `export/exporter.rs` | export file writer | unit | unit | ✅ OK |
| T4: `export/clipboard.rs` | clipboard writer | none | none | ✅ OK |
| T5: `export/mod.rs` | wiring only | none | none | ✅ OK |
| T6: `canvas/mod.rs` | UI component | none | none | ✅ OK |
| T7: `window/imp.rs` | UI component | none | none | ✅ OK |
| T8: `window/imp.rs` | UI component | none | none | ✅ OK |
| T9: `window/imp.rs` | UI component | none | none (full gate) | ✅ OK |

> **Action**: Update `.specs/codebase/TESTING.md` to add the `src/export/` rows before T1 execution.
