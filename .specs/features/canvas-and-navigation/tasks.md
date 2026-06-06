# Canvas and Navigation — Tasks

**Design:** `.specs/features/canvas-and-navigation/design.md`  
**Status:** Complete

---

## Execution Plan

All tasks are sequential — each modifies a file that the next task either reads or also modifies. No parallel execution.

```
T1 → T2 → T3 → T4
```

```
Phase 1 (Canvas Core):
  T1: canvas/imp.rs — state fields
  T2: canvas/mod.rs — draw func + zoom API + event controllers

Phase 2 (Window Integration):
  T3: ui/window/ — zoom label + zoom GActions
  T4: application.rs — keyboard accelerators
```

---

## Task Breakdown

### T1: Extend Canvas State in `canvas/imp.rs`

**What:** Add zoom, pan, pointer-position, pan-base, and zoom-callback fields to the `Canvas` imp struct. Update `Default` derive to initialize `zoom` to 1.0.

**Where:** `src/canvas/imp.rs`

**Depends on:** None (first task)

**Reuses:** Existing `RefCell<Option<ImageData>>` field pattern in the same struct

**Requirement:** CNAV-01, CNAV-02, CNAV-05, CNAV-06, CNAV-07

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**

- [ ] `struct Canvas` has these new fields with correct types:
  - `zoom: Cell<f64>` — initialized to 1.0 (cannot use `Default` derive for non-zero; implement `Default` manually or use `#[default]` if available)
  - `pan_offset: Cell<(f64, f64)>` — default (0.0, 0.0)
  - `pointer_pos: Cell<(f64, f64)>` — default (0.0, 0.0)
  - `pan_base: Cell<(f64, f64)>` — default (0.0, 0.0)
  - `zoom_changed_cb: RefCell<Option<Box<dyn Fn(f64)>>>` — default None
- [ ] `std::cell::Cell` is imported
- [ ] `Default` impl is correct — `zoom` initializes to `1.0`, not `0.0`
- [ ] Gate check passes: `cargo build`

**Tests:** none (canvas layer — GTK4 widget, no unit test per TESTING.md)  
**Gate:** build

**Verify:**
```bash
cargo build
# Expected: zero errors, zero warnings related to T1 changes
```

**Commit:** `feat(canvas): add zoom and pan state fields to Canvas imp`

---

### T2: Update Draw Function, Add Zoom API, Wire Event Controllers in `canvas/mod.rs`

**What:** (1) Update the existing draw closure to apply Cairo zoom+pan transform and bilinear filter. (2) Add public zoom API methods (`zoom_in`, `zoom_out`, `zoom_100`, `fit_to_window`, `zoom_level`, `on_zoom_changed`) and private helpers (`apply_zoom`, `image_size`, `notify_zoom_changed`). (3) Add three event controllers inside `Canvas::new()`: `EventControllerMotion` (pointer tracking), `EventControllerScroll` (scroll-wheel zoom), `GestureDrag` with `set_button(2)` (middle-mouse pan).

**Where:** `src/canvas/mod.rs`

**Depends on:** T1 (accesses `imp.zoom`, `imp.pan_offset`, `imp.pointer_pos`, `imp.pan_base`, `imp.zoom_changed_cb`)

**Reuses:**
- Existing `Canvas::new()` structure (draw func + canvas init pattern)
- Existing `Canvas::set_image()` / `Canvas::clear()` methods (unchanged)

**Requirement:** CNAV-01, CNAV-02, CNAV-03, CNAV-04, CNAV-05, CNAV-07, CNAV-08

**Tools:**
- MCP: context7 (verify GTK4 EventControllerScroll, GestureDrag, EventControllerMotion APIs)
- Skill: NONE

**Done when:**

- [ ] Draw function applies `cr.translate(pan_x, pan_y)` then `cr.scale(zoom, zoom)` before `set_source_pixbuf` — verified visually that image moves with pan and scales with zoom
- [ ] Draw function sets bilinear filter on Cairo pattern (or skips gracefully if binding unavailable)
- [ ] `zoom_in()` multiplies zoom by `ZOOM_STEP` (1.25), clamped to 800%
- [ ] `zoom_out()` divides zoom by `ZOOM_STEP` (1.25), clamped to 10%
- [ ] `zoom_100()` sets zoom to 1.0 and centers the image in the canvas (if image is loaded)
- [ ] `fit_to_window()` computes `min(canvas_w/img_w, canvas_h/img_h)`, clamps, centers image; is a no-op if canvas size is 0 or no image
- [ ] `zoom_level()` returns the current zoom `f64`
- [ ] `on_zoom_changed(cb)` stores the callback; overwrites any previous one
- [ ] `apply_zoom(raw_zoom, anchor)` applies zoom-to-anchor math correctly (formula verified in design.md)
- [ ] `EventControllerMotion` updates `imp.pointer_pos` on `connect_motion`
- [ ] `EventControllerScroll` calls `apply_zoom` with scroll factor (1.1) and `pointer_pos` anchor on `connect_scroll`; returns `glib::Propagation::Stop`
- [ ] `GestureDrag` (button=2) captures `pan_base` on `connect_drag_begin`, updates `pan_offset` on `connect_drag_update`, restores cursor on `connect_drag_end`
- [ ] Constants `ZOOM_MIN`, `ZOOM_MAX`, `ZOOM_STEP`, `SCROLL_STEP` are defined as `const` on `Canvas`
- [ ] `zoom_changed_cb` is called after every zoom change (via `notify_zoom_changed`)
- [ ] Gate check passes: `cargo build`

**Tests:** none (canvas layer — GTK4 widget)  
**Gate:** build

**Verify:**
```bash
cargo build
# Expected: zero errors, zero warnings related to T2 changes
cargo run
# Manual: Load image → scroll up/down → zoom changes. Middle-drag → pan works. Ctrl+scroll for sanity check.
```

**Commit:** `feat(canvas): implement zoom/pan rendering, API, and event controllers`

---

### T3: Add Zoom Label and Zoom GActions to MainWindow

**What:** (1) Add a `zoom_label` (`gtk::Label`, initial text "100%") field to `MainWindow` imp struct, pack it into the existing `adw::HeaderBar`. (2) Connect `canvas.on_zoom_changed(...)` so the label updates whenever zoom changes. (3) Register four new `gio::SimpleAction`s in the existing `actions` `SimpleActionGroup`: `zoom-in`, `zoom-out`, `zoom-fit`, `zoom-100`. (4) Add four zoom buttons to the header bar linked to those actions.

**Where:** `src/ui/window/imp.rs` (field + constructed changes)

**Depends on:** T2 (uses `canvas.zoom_in()`, `canvas.zoom_out()`, `canvas.zoom_100()`, `canvas.fit_to_window()`, `canvas.on_zoom_changed()`)

**Reuses:**
- Existing `MainWindow` imp `struct` field pattern (`OnceCell<Canvas>`)
- Existing `gio::SimpleAction` + `actions.add_action()` pattern (copy exactly)
- Existing `adw::HeaderBar` + `header.pack_start/end` pattern

**Requirement:** CNAV-01, CNAV-02, CNAV-03, CNAV-04, CNAV-06

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**

- [ ] `struct MainWindow` has `zoom_label: OnceCell<gtk::Label>` field
- [ ] `zoom_label` is initialized in `constructed()` with text `"100%"` and stored in `self.zoom_label`
- [ ] `canvas.on_zoom_changed(...)` callback formats zoom as `format!("{}%", (zoom * 100.0).round() as i32)` and sets it on the label
- [ ] Four `gio::SimpleAction`s registered: `zoom-in`, `zoom-out`, `zoom-fit`, `zoom-100`
  - `zoom-in` calls `canvas.zoom_in()`
  - `zoom-out` calls `canvas.zoom_out()`
  - `zoom-fit` calls `canvas.fit_to_window()`
  - `zoom-100` calls `canvas.zoom_100()`
- [ ] All four actions added to the existing `actions` group before `window.insert_action_group()`
- [ ] Header bar has zoom buttons packed at the end (right side): `+` (`win.zoom-in`), `-` (`win.zoom-out`), fit button (`win.zoom-fit`), 1:1 button (`win.zoom-100`), followed by the zoom label
- [ ] Gate check passes: `cargo build`

**Tests:** none (UI layer)  
**Gate:** build

**Verify:**
```bash
cargo build
# Expected: zero errors, zero warnings
cargo run
# Manual: zoom label shows "100%" at startup
# Manual: clicking + zooms in, label updates; - zooms out, label updates
# Manual: fit button → image fits window, label shows rounded percentage
# Manual: 1:1 button → label shows "100%"
```

**Commit:** `feat(window): add zoom label, zoom actions, and header zoom controls`

---

### T4: Register Keyboard Accelerators in `application.rs`

**What:** Override `startup()` in `impl ApplicationImpl for Application` (inside `application.rs` imp block) to register keyboard accelerators for the four zoom window actions.

**Where:** `src/application.rs` (imp block, `impl ApplicationImpl`)

**Depends on:** T3 (actions `win.zoom-in`, `win.zoom-out`, `win.zoom-fit`, `win.zoom-100` must exist on the window)

**Reuses:** Existing `impl ApplicationImpl for Application` block structure

**Requirement:** CNAV-01, CNAV-02, CNAV-03, CNAV-04

**Tools:**
- MCP: NONE
- Skill: NONE

**Done when:**

- [ ] `startup()` override is added to `impl ApplicationImpl for Application` and calls `self.parent_startup()` first
- [ ] `app.set_accels_for_action("win.zoom-in", &["<Control>plus", "<Control>equal"])` registered
- [ ] `app.set_accels_for_action("win.zoom-out", &["<Control>minus"])` registered
- [ ] `app.set_accels_for_action("win.zoom-fit", &["<Control><Shift>f"])` registered
- [ ] `app.set_accels_for_action("win.zoom-100", &["<Control>0"])` registered
- [ ] Gate check passes: `cargo build && cargo test --lib`

**Tests:** none (application entry layer)  
**Gate:** full (`cargo build && cargo test --lib`)

**Verify:**
```bash
cargo build && cargo test --lib
# Expected: build succeeds, all existing tests pass (test count unchanged or +0 new tests)
cargo run
# Manual: press Ctrl++ → zoom increases, label updates
# Manual: press Ctrl+- → zoom decreases, label updates
# Manual: press Ctrl+Shift+F → fit to window
# Manual: press Ctrl+0 → 100% zoom, label shows "100%"
# Manual: scroll wheel over image → zoom changes with cursor as anchor
# Manual: middle-mouse drag → image pans; cursor changes to grabbing while dragging
```

**Commit:** `feat(app): register keyboard accelerators for zoom actions`

---

## Parallel Execution Map

```
Phase 1 (Sequential):
  T1 ──→ T2

Phase 2 (Sequential):
  T2 ──→ T3 ──→ T4
```

All tasks are sequential. No `[P]` flags. Rationale:
- T2 depends on T1's new state fields (must compile)
- T3 depends on T2's public API methods (must compile)
- T4 depends on T3's GActions being registered on the window action group

---

## Task Granularity Check

| Task | Scope | Status |
|---|---|---|
| T1: Canvas state fields | 1 file (`imp.rs`), struct extension | ✅ Granular |
| T2: Draw func + zoom API + event controllers | 1 file (`mod.rs`), all canvas behavior | ⚠️ Cohesive (3 related concerns in 1 file — cannot split without creating file conflicts) |
| T3: Zoom label + zoom GActions | 1 file (`imp.rs`), window integration | ✅ Granular |
| T4: Keyboard accelerators | 1 file (`application.rs`), 4 lines of accel registration | ✅ Granular |

T2 is the largest task but cannot be atomized further without creating parallel file-write conflicts (all three concerns live in `mod.rs Canvas::new()`). It has a single file scope and a clear, testable outcome.

---

## Diagram-Definition Cross-Check

| Task | Depends On (task body) | Diagram Shows | Status |
|---|---|---|---|
| T1 | None | No incoming arrow | ✅ Match |
| T2 | T1 | T1 → T2 | ✅ Match |
| T3 | T2 | T2 → T3 | ✅ Match |
| T4 | T3 | T3 → T4 | ✅ Match |

---

## Test Co-location Validation

| Task | Code Layer Created/Modified | Matrix Requires | Task Says | Status |
|---|---|---|---|---|
| T1 | `src/canvas/imp.rs` | none | none | ✅ OK |
| T2 | `src/canvas/mod.rs` | none | none | ✅ OK |
| T3 | `src/ui/window/imp.rs` | none | none | ✅ OK |
| T4 | `src/application.rs` | none | none | ✅ OK |

All code layers fall under the `none` test category per TESTING.md (GTK4 widgets + application entry). No test co-location violations.

---

## Requirement Coverage

| Requirement ID | Story | Covered by Task(s) |
|---|---|---|
| CNAV-01 | P1: Zoom Controls (zoom-in) | T2 (`zoom_in()`), T3 (GAction + button), T4 (Ctrl++) |
| CNAV-02 | P1: Zoom Controls (zoom-out) | T2 (`zoom_out()`), T3 (GAction + button), T4 (Ctrl+-) |
| CNAV-03 | P1: Zoom Controls (fit) | T2 (`fit_to_window()`), T3 (GAction + button), T4 (Ctrl+Shift+F) |
| CNAV-04 | P1: Zoom Controls (100%) | T2 (`zoom_100()`), T3 (GAction + button), T4 (Ctrl+0) |
| CNAV-05 | P1: Middle-Mouse Pan | T2 (`GestureDrag` button=2) |
| CNAV-06 | P1: Zoom Level Indicator | T3 (label + callback) |
| CNAV-07 | P1: Scroll-Wheel Zoom | T2 (`EventControllerScroll`) |
| CNAV-08 | P2: Image Quality | T2 (bilinear filter in draw func) |

**Coverage:** 8/8 requirements mapped ✅
