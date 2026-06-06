# Screenshot Hero - State

*Persistent memory across sessions: decisions, blockers, lessons, deferred ideas.*

---

## Current Focus

**Feature:** PRD-003 - Annotations  
**Phase:** Planning complete → Ready to Execute  
**Next action:** Start T1 (Create annotation data model in `src/annotations/model.rs`)

### Previous Focus

**Feature:** PRD-002 - Canvas and Navigation  
**Phase:** Planning complete → Implemented (see `.specs/features/canvas-and-navigation/`)

### Earlier Focus

**Feature:** PRD-001 - Screenshot Capture and Loading  
**Phase:** Implemented (see `.specs/features/capture-and-loading/`)

---

## Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
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

- [ ] Verify exact crate versions for gtk4-rs ecosystem before T1 (use Context7 or crates.io)
- [ ] Confirm Flatpak runtime version (GNOME SDK) used in POC-003 matches manifest in T12
- [ ] Confirm `ashpd` async runtime compatibility (zbus + glib) during T6
- [ ] PRD-002 T2: Verify `GestureDrag::connect_drag_begin` signature in gtk4-rs bindings version in use
- [ ] PRD-002 T2: Verify `set_cursor_from_name` availability on `Widget` (alternative: `gdk::Cursor::from_name` + `widget.set_cursor`)
- [ ] PRD-002 T2: Verify `cr.source()` returns filterable pattern in cairo-rs bindings (bilinear filter path)
- [ ] PRD-002 T4: Confirm `ApplicationImpl::startup()` is the correct override in the libadwaita subclassing pattern in use
- [ ] PRD-003 T7: Verify `cairo-rs` API for pixbuf sub-region extraction + scale operations (for Blur/Pixelate renderers)
- [ ] PRD-003 T12: Verify `adw::MessageDialog` with `gtk::Entry` API availability in current libadwaita-rs version
- [ ] PRD-003 T14: Verify `gtk::ColorButton` vs `gtk::ColorDialog` availability in gtk4-rs version in use
- [ ] PRD-003: Add `uuid` crate to `Cargo.toml` before starting T1 (features = ["v4"])

---

## Deferred Ideas

| Idea | Why Deferred | Revisit At |
|------|-------------|------------|
| `gdk4::Texture` for image loading | Simpler to start with Pixbuf; Texture has better GPU path | Milestone 5 (export) |
| Drag-and-drop image loading | Not in PRD-001 scope | PRD-001 backlog |
| Recent files list | Requires GSettings + file history | PRD-004 (project management) |
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

*None yet — first session.*

---

## Preferences

*None recorded yet.*
