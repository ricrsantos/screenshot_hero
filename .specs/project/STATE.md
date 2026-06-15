# Screenshot Hero - State

*Persistent memory across sessions: decisions, blockers, lessons, deferred ideas.*

---

## Current Focus

**Workstream:** behavior-settings-extend  
**Phase:** Active  
**Next action:** Run manual UAT for capture behavior settings (`--capture` with and without temporary mode, clipboard exit, and window reuse/new-window policy).

### Previous Focus

**Feature:** PRD-006 - Settings and Preferences  
**Phase:** Implemented

**Feature:** PRD-004 - Project Management  
**Phase:** Implemented (T1–T8 complete)  
**Next action (deferred):** Manual UAT — save/open project, verify auto-save updates `.shero` after annotation changes

**Feature:** PRD-003 - Annotations  
**Phase:** Planning complete → Ready to Execute (tasks in `.specs/features/annotations/tasks.md`)

### Earlier Focus

**Feature:** PRD-002 - Canvas and Navigation  
**Phase:** Implemented (see `.specs/features/canvas-and-navigation/`)

**Feature:** PRD-001 - Screenshot Capture and Loading  
**Phase:** Implemented (see `.specs/features/capture-and-loading/`)

---

## Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-15 | Exit-after-paste implemented using `gdk::Clipboard::connect_changed` watcher | Native GTK signal reliably indicates clipboard ownership/content change; allows auto-quit without polling |
| 2026-06-15 | Temporary post-capture disable state tracks `started-at` epoch and auto-resets on expiry | Ensures feature self-deactivates after configured duration and restores default behavior |
| 2026-06-15 | Default behavior for in-app new capture is now window reuse (`open-new-window-on-capture=false`) | Reduces window proliferation and matches requested default UX |
| 2026-06-06 | Off-screen render uses `cairo::Format::ARgb32` + manual BGRA→RGBA + un-premultiply conversion | Required for annotation alpha blending; avoids additional crate dependency; `Pixbuf::from_bytes` accepts raw RGBA |
| 2026-06-06 | Auto-export always produces PNG (not JPEG) | ADR-001 specifies `original_name_shero.png`; PNG is lossless, appropriate for screenshots |
| 2026-06-06 | GSettings schema ID: `com.screenshot_hero.ScreenshotHero`, path `/com/screenshot_hero/ScreenshotHero/` | Matches app-id in Flatpak manifest and `Application::new()` property |
| 2026-06-06 | `env_logger` not yet initialized anywhere — currently all `log::*` calls are silent | Will be fixed in PRD-006 T7: `env_logger::Builder::new().filter_level(Trace).init()` in `Application::startup()` |
| 2026-06-06 | GSettings `color-scheme` mapped to `adw::StyleManager::set_color_scheme()` using `ColorScheme::Default / ForceLight / ForceDark` | Libadwaita-native; avoids custom theme switching |
| 2026-06-06 | `log::set_max_level()` used for runtime log level changes | Works with env_logger because `log` checks max level before dispatching; no logger reinit needed |
| 2026-06-06 | `AppSettings::try_new()` returns `Option<Self>` for graceful degradation when GSettings schema is absent | Prevents crash on dev machines without compiled schema; fallback to hardcoded defaults |
| 2026-06-06 | Dev run requires: `glib-compile-schemas data/ && GSETTINGS_SCHEMA_DIR=data/ cargo run` | GSettings schema must be compiled before any `gio::Settings::new()` call |
| 2026-06-06 | Export config stored as `Cell<bool>` / `RefCell<String>` fields in `MainWindow` | PRD-006 T8 will replace these with GSettings reads; fields removed from struct |
| 2026-06-06 | Debounce via `glib::timeout_add_local_once` + `SourceId::remove()` stored in `RefCell<Option<glib::SourceId>>` | GLib main-thread safe; consistent with project's use of GLib primitives; 300ms per ADR-001 |
| 2026-06-06 | No new crate dependencies for export/clipboard | All required APIs available via existing `gtk`, `glib`, `gdk-pixbuf`; keeps Flatpak manifest clean |
| 2026-06-06 | Add serde derives directly to `src/annotations/model.rs` (no DTO layer) | All model types are pure Rust with no GTK dependency; avoids redundant struct definitions; all crates (`serde`, `serde_json`, `uuid` serde feature) are already in Cargo.toml |
| 2026-06-06 | Timestamps stored as `String` (RFC 3339) in `.shero` | Avoids enabling `chrono/serde` feature; strings are portable and human-readable in the JSON file; `chrono::Utc::now().to_rfc3339()` is available without feature changes |
| 2026-06-06 | Auto-save trigger via `on_annotation_changed` callback at Window level | Callback already fires on every execute/undo/redo; zero new mechanism needed; keeps `src/annotations/` free of GTK/persistence concerns |
| 2026-06-06 | Atomic write for `.shero` save (write to `.shero.tmp` then `fs::rename`) | Prevents file corruption if process is killed mid-write; `fs::rename` is atomic on Linux for same-filesystem paths |
| 2026-06-06 | `ProjectManager` is pure Rust stored in `RefCell<ProjectManager>` in Window imp | No GTK dependency; fully unit-testable without display server; consistent with `src/annotations/` separation pattern |
| 2026-06-06 | `const APP_VERSION: &str = env!("CARGO_PKG_VERSION")` for metadata | Compile-time version injection; always matches built binary version |
| 2026-06-06 | Annotation coordinates stored in image-space only | Invariant under zoom/pan; required for ADR-003 "canvas == export" guarantee |
| 2026-06-06 | Separate `src/annotations/` module — no GTK dependency | Enables unit testing of model/engine/history without a display server |
| 2026-06-06 | Command pattern (AnnotationCommand enum) for undo/redo | Clean inverse-operation mapping; each command stores both old and new state |
| 2026-06-06 | Single GestureDrag (button=1) handles draw/move/resize via DrawingState dispatch | Avoids gesture conflicts; behavior determined at drag_begin by active_tool + hit-test |
| 2026-06-06 | Text editing via modal dialog (adw::MessageDialog + gtk::Entry) | Avoids complex in-canvas text cursor; simpler + correct GTK4 pattern |
| 2026-06-06 | Blur/Pixelate via pixbuf sub-region scale-down → scale-up | No external blur crate; acceptable privacy-grade quality |
| 2026-06-06 | Tool palette as left sidebar (vertical gtk::Box) | Standard for drawing tools; leaves full canvas area for annotation work |
| 2026-06-06 | Number marker counter in AnnotationEngine (not derived from list) | Survives tool switches; resets only on full undo of all markers |
| 2026-06-05 | Use `gtk4::DrawingArea` + Cairo for canvas | Validated in POC-003-04; direct rendering control, required for annotation layer later |
| 2026-06-05 | Use `gdk_pixbuf::Pixbuf` for image loading in Milestone 1 | Simpler API, validated in POC-002; migrate to `gdk4::Texture` if performance requires it |
| 2026-06-05 | Async portal calls via `glib::spawn_future_local` | Keeps GTK on main thread, avoids Send bounds issues with GTK objects |
| 2026-06-05 | Canvas zoom/pan deferred to PRD-002 | PRD-001 scope is capture + display; zoom/pan is navigation (PRD-002) |
| 2026-06-05 | No automated UI tests; unit tests for pure Rust logic only | GTK4 UI testing requires display server; impractical for CI without Xvfb/Wayland setup |
| 2026-06-05 | Zoom/pan state in `Cell<f64>` / `Cell<(f64,f64)>` (not RefCell) | `f64` and `(f64,f64)` are `Copy`; `Cell` is simpler for scalar values |
| 2026-06-05 | Zoom callback via stored closure `RefCell<Option<Box<dyn Fn(f64)>>>` | Avoids GObject property registration; single consumer (zoom label) sufficient |
| 2026-06-05 | Pointer tracking via `EventControllerMotion` in Canvas | Only GTK4-idiomatic way to get cursor coords for zoom-to-cursor in scroll handler |
| 2026-06-05 | Middle-mouse pan via `GestureDrag` with `set_button(2)` | GTK4-native; built-in delta math; cleaner than raw `EventControllerLegacy` |
| 2026-06-05 | Keyboard accels in `Application::startup()` | GTK4 best practice — startup fires once before any window is created |
| 2026-06-05 | No pan boundary clamping | Per spec: user can recover any position with fit-to-window |
| 2026-06-05 | Zoom step: 1.25× for buttons, 1.1× for scroll wheel | 1.25 is one visual "stop" per click (standard); 1.1 gives fine-grained scroll control |

---

## Blockers

*None currently.*

---

## Todos

- [ ] Run manual UAT checklist for all v1 flows in native Cargo runtime
- [ ] Run manual UAT checklist for all v1 flows in Flatpak runtime
- [ ] Decide whether to update feature task files (`tasks.md`) from `Draft` to executed/completed state
- [ ] Add CI smoke strategy for `cargo build` + `cargo test --lib` (no display server)

---

## Deferred Ideas

| Idea | Why Deferred | Revisit At |
|------|-------------|------------|
| `gdk4::Texture` for image loading | Simpler to start with Pixbuf; Texture has better GPU path | Milestone 5 (export) |
| Drag-and-drop image loading | Not in PRD-001 scope | PRD-001 backlog |
| Recent files list | Requires GSettings + file history | PRD-006 (settings) — excluded from PRD-004 scope |
| Image format validation (magic bytes) | File extension check sufficient for v1 | PRD-001 backlog |
| Multi-selection of annotations | ADR-003 defers this explicitly | Post-PRD-003 |
| Copy/paste annotations | Not in PRD-003 scope | Post-PRD-003 |
| Annotation Z-order (layer) management | Not in PRD-003 scope | Post-PRD-003 |
| Snap-to-grid for annotations | Not in PRD-003 scope | Post-PRD-003 |
| Curved arrows | Straight arrow sufficient for v1 | Post-PRD-003 |
| Annotation renumbering (auto-reorder on delete) | Gaps are acceptable per spec; renumbering adds complexity | Post-PRD-003 |
| In-canvas text cursor (no dialog) | Requires custom Cairo text input state machine | Post-PRD-003 polish |

---

## Lessons Learned

- Prompt-driven implementation can outpace planning docs quickly; the `ROADMAP.md` and `STATE.md` should be refreshed at the end of each implementation session.
- Keeping `.specs/codebase/` complete (all 7 files) makes future planning phases faster and reduces re-discovery.

---

## Preferences

- Prefer objective status updates backed by commands (`cargo test --lib`, `cargo build`) before changing roadmap state.
