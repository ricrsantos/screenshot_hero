# Screenshot Capture and Loading — Tasks

**Design:** `.specs/features/capture-and-loading/design.md`  
**Spec:** `.specs/features/capture-and-loading/spec.md`  
**Status:** Approved

---

## Execution Plan

### Phase 1: Project Foundation (Sequential)

Must be done first, in order. Establishes Cargo project and application skeleton.

```
T1 → T2 → T3
```

### Phase 2: Data Layer + UI Shell (Parallel)

After T3, these two tasks have no shared state and can run simultaneously.

```
T3 ──┬──→ T4 [P]
     └──→ T5 [P]
```

### Phase 3: Services + Canvas (Parallel after T4 + T5)

Services depend on ImageData (T4). Canvas depends on both T4 and T5.

```
T4 + T5 complete, then:
     ┌──→ T6 [P]  (CaptureService)
T4 ──┼──→ T7 [P]  (FileLoader)
     └──→ T8      (Canvas — also needs T5, sequential to avoid GTK state conflict)
```

> Note: T8 (Canvas, a GObject subclass) should be sequential with T6/T7 to avoid any potential GTK registration conflicts during development. Strip [P] from T8.

### Phase 4: Integration (Sequential)

Wire everything together. Must be done in order.

```
T6 + T7 + T8 complete, then:
T9 → T10 → T11
```

### Phase 5: Packaging (After T3, Independent)

Flatpak manifest can be written independently once the app ID is known.

```
T3 ──→ T12 (independent of Phases 2-4, can run anytime after T3)
```

---

## Task Breakdown

### T1: Initialize Cargo project

**What:** Create `Cargo.toml` with all required dependencies and project metadata, plus directory scaffold (`src/`, `build/`, `tests/fixtures/`)
**Where:** `Cargo.toml`, `src/lib.rs` (empty), directory structure
**Depends on:** None
**Reuses:** Nothing
**Requirement:** CAPT-01 (prerequisite for all)

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `Cargo.toml` has `[package]` with name `screenshot-hero`, edition 2021
- [ ] Dependencies include `gtk4`, `libadwaita`, `ashpd`, `gdk-pixbuf`, `serde` + `serde_json`, `uuid`, `log`, `env_logger`
- [ ] `cargo build` compiles without errors (empty project)
- [ ] `src/`, `build/`, `tests/fixtures/` directories exist

**Tests:** none
**Gate:** build → `cargo build`

**Commit:** `build: initialize cargo project with gtk4 and libadwaita dependencies`

---

### T2: Create Application struct

**What:** Implement `adw::Application` GObject subclass that creates the main window on `activate`
**Where:** `src/application.rs`, `src/main.rs`
**Depends on:** T1
**Reuses:** Standard gtk4-rs GObject subclass pattern
**Requirement:** CAPT-01

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `Application` is an `adw::Application` subclass (using `glib::subclass`)
- [ ] `main.rs` creates the application and calls `.run()`
- [ ] `activate` signal handler is wired (creates/presents `MainWindow`)
- [ ] `cargo build` passes

**Tests:** none
**Gate:** build → `cargo build`

**Commit:** `feat(app): add adw::Application subclass with activate handler`

---

### T3: Create MainWindow skeleton

**What:** Implement `adw::ApplicationWindow` GObject subclass with `adw::ToolbarView`, `adw::HeaderBar`, and placeholder content area; register `new-screenshot` and `open-file` GActions (no-op handlers for now)
**Where:** `src/ui/window/mod.rs`, `src/ui/window/imp.rs`
**Depends on:** T2
**Reuses:** Standard gtk4-rs ApplicationWindow subclass pattern
**Requirement:** CAPT-01

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `MainWindow` is an `adw::ApplicationWindow` GObject subclass
- [ ] Window structure: `adw::ToolbarView` → `adw::HeaderBar` (top) + placeholder widget (content)
- [ ] Header bar has "New Screenshot" button, "Open File" button
- [ ] `new-screenshot` and `open-file` GActions are registered with no-op handlers
- [ ] Window title is "Screenshot Hero"
- [ ] `cargo build` passes
- [ ] `cargo run` shows the window visually (manual verification)

**Tests:** none
**Gate:** build → `cargo build`

**Commit:** `feat(window): add main window skeleton with header bar and no-op actions`

---

### T4: Create ImageData model [P]

**What:** Define `ImageData` and `SourceImage` structs with constructor and accessors
**Where:** `src/models/image.rs`, `src/models/mod.rs`
**Depends on:** T1 (Cargo.toml must exist)
**Reuses:** Nothing
**Requirement:** CAPT-06, CAPT-07

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `ImageData` struct with `pixbuf: gdk_pixbuf::Pixbuf` and `source: SourceImage`
- [ ] `SourceImage` struct with `path: PathBuf`, `width: i32`, `height: i32`
- [ ] `ImageData::from_pixbuf(pixbuf, source)` constructor
- [ ] Accessors: `pixbuf()`, `source()`, `width()`, `height()`
- [ ] Unit tests: verify width/height accessors return correct values from a mocked pixbuf dimensions
- [ ] Gate check passes: `cargo test --lib`

**Tests:** unit
**Gate:** quick → `cargo test --lib`

**Commit:** `feat(models): add ImageData and SourceImage structs`

---

### T5: Create Canvas widget skeleton [P]

**What:** Implement `Canvas` as a GTK4 GObject subclass wrapping `gtk4::DrawingArea`; `set_image()` stores the image and queues a redraw; `clear()` removes it
**Where:** `src/canvas/mod.rs`, `src/canvas/imp.rs`
**Depends on:** T3 (needs window to register widget type, must compile together), T4 (uses `ImageData`)
**Reuses:** GObject subclass pattern from T3
**Requirement:** CAPT-06, CAPT-07

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `Canvas` is a GObject subclass of `gtk4::DrawingArea`
- [ ] `imp.rs` has `image: RefCell<Option<ImageData>>`
- [ ] `Canvas::new()` creates widget, registers draw function via `set_draw_func`
- [ ] `Canvas::set_image(&self, image: ImageData)` stores image, calls `queue_draw()`
- [ ] `Canvas::clear(&self)` sets image to `None`, calls `queue_draw()`
- [ ] Draw function: if image is set, paints pixbuf at (0, 0) via Cairo; otherwise paints nothing (empty/gray background)
- [ ] `Canvas` replaces the placeholder in `MainWindow` content area
- [ ] `cargo build` passes

**Tests:** none (GTK widget, requires display)
**Gate:** build → `cargo build`

**Commit:** `feat(canvas): add Canvas GObject widget with image display`

---

### T6: Implement CaptureService [P]

**What:** Pure Rust service that invokes the GNOME Screenshot Portal via `ashpd` and returns an `ImageData` or a `CaptureError`
**Where:** `src/capture/service.rs`, `src/capture/mod.rs`
**Depends on:** T4 (uses `ImageData`), T7 (reuses `FileLoader::load_from_uri()`)
**Reuses:** `FileLoader` for image loading after receiving URI
**Requirement:** CAPT-02, CAPT-03, CAPT-04, CAPT-05

**Tools:**
- MCP: `context7` (verify ashpd screenshot API before implementing)
- Skill: NONE

**Done when:**
- [ ] `CaptureError` enum: `PortalUnavailable(String)`, `PortalCancelled`, `ImageLoadFailed(String)`
- [ ] `CaptureService::capture() -> Result<Option<ImageData>, CaptureError>` (async fn)
  - Returns `Ok(None)` on user cancellation (empty URI)
  - Returns `Ok(Some(image))` on success
  - Returns `Err(CaptureError::PortalUnavailable)` if portal fails to respond
- [ ] Unit tests: test cancellation path (Ok(None) when URI is empty) and error path
  - Note: actual portal call cannot be tested; test error handling logic around it
- [ ] Gate check passes: `cargo test --lib`

**Tests:** unit
**Gate:** quick → `cargo test --lib`

**Commit:** `feat(capture): add CaptureService with ashpd portal integration`

---

### T7: Implement FileLoader [P]

**What:** Pure Rust service that loads a PNG or JPEG file from a path or URI and returns an `ImageData` or a `LoadError`
**Where:** `src/capture/loader.rs`
**Depends on:** T4 (uses `ImageData`)
**Reuses:** Nothing
**Requirement:** CAPT-08, CAPT-09, CAPT-10

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `LoadError` enum: `FileNotFound(PathBuf)`, `UnsupportedFormat(PathBuf)`, `DecodeFailed(String)`, `InvalidUri(String)`
- [ ] `FileLoader::load_from_path(path: &Path) -> Result<ImageData, LoadError>`
  - Validates extension (`.png`, `.jpg`, `.jpeg`) → `UnsupportedFormat` if not matching
  - Checks file exists → `FileNotFound` if not
  - Calls `gdk_pixbuf::Pixbuf::from_file()` → `DecodeFailed` on error
  - Returns `ImageData` on success
- [ ] `FileLoader::load_from_uri(uri: &str) -> Result<ImageData, LoadError>`
  - Converts `file://` URI to path → delegates to `load_from_path()`
  - Returns `InvalidUri` on malformed URI
- [ ] Unit tests with fixture files in `tests/fixtures/`:
  - `test_load_png_valid_file` — loads a real PNG fixture, asserts Ok
  - `test_load_jpeg_valid_file` — loads a real JPEG fixture, asserts Ok
  - `test_load_unsupported_format` — `.bmp` extension → Err(UnsupportedFormat)
  - `test_load_nonexistent_file` — path doesn't exist → Err(FileNotFound)
  - `test_load_from_uri_valid` — file:// URI → asserts Ok
  - `test_load_from_uri_invalid` — bad URI → Err(InvalidUri)
- [ ] Add sample PNG and JPEG fixtures to `tests/fixtures/`
- [ ] Gate check passes: `cargo test --lib` (6 tests pass)

**Tests:** unit
**Gate:** quick → `cargo test --lib`

**Commit:** `feat(capture): add FileLoader with PNG and JPEG support`

---

### T8: Wire "New Screenshot" action → CaptureService → Canvas

**What:** Connect the `new-screenshot` GAction in `MainWindow` to call `CaptureService::capture()` asynchronously, then call `canvas.set_image()` on success or show an error dialog on failure
**Where:** `src/ui/window/imp.rs` (replace no-op handler), `src/ui/window/mod.rs`
**Depends on:** T5 (Canvas in window), T6 (CaptureService), error_dialog helper → see T10
**Reuses:** `CaptureService`, `Canvas::set_image()`
**Requirement:** CAPT-01, CAPT-02, CAPT-03, CAPT-04, CAPT-05

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `new-screenshot` action handler uses `glib::spawn_future_local` (or equivalent) to call `CaptureService::capture()` asynchronously
- [ ] On `Ok(Some(image))` → calls `self.canvas.set_image(image)`
- [ ] On `Ok(None)` (cancelled) → does nothing (no dialog)
- [ ] On `Err(CaptureError::PortalUnavailable(msg))` → shows `adw::AlertDialog` with `msg`
- [ ] On `Err(CaptureError::ImageLoadFailed(msg))` → shows `adw::AlertDialog` with `msg`
- [ ] Log all errors at `Error` level before showing dialog
- [ ] `cargo build` passes
- [ ] Manual test: activate "New Screenshot", select region, image appears in canvas

**Tests:** none (GTK widget + async + portal)
**Gate:** build → `cargo build`

**Commit:** `feat(window): wire new-screenshot action to capture service and canvas`

---

### T9: Wire "Open File" action → FileLoader → Canvas

**What:** Connect the `open-file` GAction to open a `gtk4::FileDialog` filtered to PNG/JPEG, then call `FileLoader::load_from_path()` and update the canvas
**Where:** `src/ui/window/imp.rs`, `src/ui/window/mod.rs`
**Depends on:** T5 (Canvas), T7 (FileLoader), T8 (pattern established)
**Reuses:** `FileLoader`, `Canvas::set_image()`, error dialog pattern from T8
**Requirement:** CAPT-08, CAPT-09, CAPT-10

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `open-file` action handler opens `gtk4::FileDialog` with PNG/JPEG filter
- [ ] On file selected → calls `FileLoader::load_from_path()` → `canvas.set_image()` on `Ok`
- [ ] On cancel → does nothing
- [ ] On `Err` → shows `adw::AlertDialog` with the filename + reason
- [ ] Log errors at `Error` level
- [ ] `cargo build` passes
- [ ] Manual test: open PNG → image appears; open JPEG → image appears; open corrupt file → error dialog

**Tests:** none
**Gate:** build → `cargo build`

**Commit:** `feat(window): wire open-file action to file loader and canvas`

---

### T10: Implement error dialog helper

**What:** Extract error dialog display into a reusable `show_error_dialog(parent: &impl IsA<gtk4::Window>, title: &str, message: &str)` function to avoid duplication between T8 and T9
**Where:** `src/ui/dialogs.rs`
**Depends on:** T8, T9 (both exist and use inline error dialog; refactor into helper)
**Reuses:** `adw::AlertDialog` pattern from T8/T9
**Requirement:** CAPT-10

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `show_error_dialog` function in `src/ui/dialogs.rs`
- [ ] T8 and T9 updated to use the helper (no inline `adw::AlertDialog` construction)
- [ ] `cargo build` passes
- [ ] Manual test: trigger capture error and file load error; both show dialog with correct message

**Tests:** none
**Gate:** build → `cargo build`

**Commit:** `refactor(ui): extract error dialog into reusable helper`

---

### T11: Full gate check — Phase 4 integration

**What:** Run the full gate check across all unit tests and build to confirm the integrated system compiles and all tests pass
**Where:** All source files (read-only validation)
**Depends on:** T10
**Reuses:** All tasks above
**Requirement:** All CAPT-* requirements

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] `cargo build && cargo test --lib` passes with 0 failures
- [ ] Unit test count: at minimum 8 (6 from T7 + 2 from T6 — no silent deletions)
- [ ] No compiler warnings that indicate logic errors (allow `dead_code` for stubs only)

**Tests:** unit
**Gate:** full → `cargo build && cargo test --lib`

**Commit:** none (validation task only)

---

### T12: Create Flatpak manifest

**What:** Write the Flatpak manifest with correct portal permissions, GNOME runtime version, and Rust SDK; validate with `flatpak-builder --build-only`
**Where:** `build/com.screenshot_hero.ScreenshotHero.yml`
**Depends on:** T3 (app ID established)
**Reuses:** Findings from POC-003-01, POC-003-02, POC-003-03
**Requirement:** All CAPT-* (NFR: Flatpak compatible)

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**
- [ ] Manifest has correct `app-id`, `runtime` (GNOME SDK), `sdk`, `command`
- [ ] Portal permissions: `--talk-name=org.freedesktop.portal.Screenshot`, `--talk-name=org.freedesktop.portal.Desktop`
- [ ] Filesystem permission: `xdg-pictures` (read) for screenshot access
- [ ] Rust build system module uses `cargo` build system
- [ ] `flatpak-builder --build-only build-dir build/com.screenshot_hero.ScreenshotHero.yml` succeeds
- [ ] Manual validation: `flatpak-builder --install --user` + `flatpak run` shows window and capture works

**Tests:** none
**Gate:** build → `flatpak-builder --build-only build-dir build/com.screenshot_hero.ScreenshotHero.yml --force-clean`

**Commit:** `build: add flatpak manifest with screenshot portal permissions`

---

## Parallel Execution Map

```
Phase 1 (Sequential):
  T1 ──→ T2 ──→ T3

Phase 2 (Parallel — after T3):
  T3 complete, then:
    ├── T4 [P]  (ImageData model)
    └── T5 [P]  (Canvas widget)
  Also: T12 can start after T3 independently

Phase 3 (Parallel — after T4):
  T4 + T5 complete, then:
    ├── T6 [P]  (CaptureService)
    └── T7 [P]  (FileLoader)
  T8 (Canvas) after T4 + T5 — sequential (GObject registration)

Phase 4 (Sequential):
  T6 + T7 + T8 complete, then:
    T9 ──→ T10 ──→ T11

Packaging (independent):
  T3 ──→ T12 (runs any time after T3)
```

---

## Pre-Approval Validation

### Check 1: Task Granularity

| Task | Scope | Status |
|---|---|---|
| T1: Cargo.toml + scaffold | 1 config file | ✅ Granular |
| T2: Application struct | 2 files (app.rs + main.rs) — cohesive pair | ✅ OK |
| T3: MainWindow skeleton | 2 files (mod.rs + imp.rs) — GObject pattern requires both | ✅ OK |
| T4: ImageData model | 1 module, 2 structs | ✅ Granular |
| T5: Canvas widget | 2 files (mod.rs + imp.rs) — GObject pattern | ✅ OK |
| T6: CaptureService | 1 service file + error type | ✅ Granular |
| T7: FileLoader | 1 service file + error type | ✅ Granular |
| T8: Wire capture action | 1 handler in imp.rs | ✅ Granular |
| T9: Wire open-file action | 1 handler in imp.rs | ✅ Granular |
| T10: Error dialog helper | 1 function extraction | ✅ Granular |
| T11: Full gate check | Validation only | ✅ OK |
| T12: Flatpak manifest | 1 YAML file | ✅ Granular |

### Check 2: Diagram-Definition Cross-Check

| Task | Depends On (task body) | Diagram Shows | Status |
|---|---|---|---|
| T1 | None | Start of Phase 1 | ✅ Match |
| T2 | T1 | T1 → T2 | ✅ Match |
| T3 | T2 | T2 → T3 | ✅ Match |
| T4 | T1 (via T3) | T3 → T4 [P] | ✅ Match |
| T5 | T3, T4 | T3 → T5 [P] | ✅ Match |
| T6 | T4, T7 (reuses) | T4 → T6 [P] | ✅ Match |
| T7 | T4 | T4 → T7 [P] | ✅ Match |
| T8 | T5, T6 | T6 + T7 + T8 → Phase 4 | ✅ Match |
| T9 | T5, T7, T8 | T9 sequential after T8 | ✅ Match |
| T10 | T8, T9 | T9 → T10 | ✅ Match |
| T11 | T10 | T10 → T11 | ✅ Match |
| T12 | T3 | T3 → T12 (independent) | ✅ Match |

### Check 3: Test Co-location Validation

| Task | Code Layer | Matrix Requires | Task Says | Status |
|---|---|---|---|---|
| T1 | Config | none | none | ✅ OK |
| T2 | Entry/App | none | none | ✅ OK |
| T3 | UI component | none | none | ✅ OK |
| T4 | Model | unit | unit | ✅ OK |
| T5 | UI component | none | none | ✅ OK |
| T6 | Service | unit | unit | ✅ OK |
| T7 | Service | unit | unit | ✅ OK |
| T8 | UI wiring | none | none | ✅ OK |
| T9 | UI wiring | none | none | ✅ OK |
| T10 | UI helper | none | none | ✅ OK |
| T11 | Validation | unit (gate) | unit | ✅ OK |
| T12 | Build config | none | none | ✅ OK |

All checks pass ✅

---

## Requirement Traceability Update

| Requirement ID | Task(s) | Status |
|---|---|---|
| CAPT-01 | T1, T2, T3, T8 | In Tasks |
| CAPT-02 | T6, T8 | In Tasks |
| CAPT-03 | T6, T8 | In Tasks |
| CAPT-04 | T6, T8 | In Tasks |
| CAPT-05 | T6, T8 | In Tasks |
| CAPT-06 | T4, T5 | In Tasks |
| CAPT-07 | T4, T5 | In Tasks |
| CAPT-08 | T7, T9 | In Tasks |
| CAPT-09 | T7, T9 | In Tasks |
| CAPT-10 | T7, T8, T9, T10 | In Tasks |

**Coverage:** 10 total, 10 mapped to tasks, 0 unmapped ✅
