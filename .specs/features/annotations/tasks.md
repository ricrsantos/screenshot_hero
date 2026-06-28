# Annotations — Tasks

**Design:** `.specs/features/annotations/design.md`  
**Status:** Approved

---

## Execution Plan

```
Phase 1 — Data Foundation (Sequential):
  T1 → T2 → T3 → T4 → T5

Phase 2a — Canvas (Sequential after T5):
  T5 → T6 → T7 → T8 → T9 → T10 → T11 → T12

Phase 2b — Tool Palette (Parallel [P] after T4):
  T4 → T13 [P] → T14

Phase 2c — Undo/Redo Actions (Parallel [P] after T6):
  T6 → T15 [P]

Phase 3 — Integration (after T12 + T14 + T15):
  T12 + T14 + T15 → T16 → T17 → T18
```

```
T1 ──→ T2 ──→ T3 ──→ T4 ──→ T5 ──→ T6 ──→ T7 ──→ T8 ──→ T9 ──→ T10 ──→ T11 ──→ T12 ──┐
                      │                    │                                              │
                      └──→ T13 [P] ──→ T14 ┘                                             ├──→ T16 ──→ T17 ──→ T18
                                           └──→ T15 [P] ──────────────────────────────────┘
```

---

## Task Breakdown

### T1: Annotation Data Model

**What:** Create `src/annotations/model.rs` with all annotation types, coordinate primitives, and style struct as defined in design.md.

**Where:** `src/annotations/model.rs` (new file)

**Depends on:** None (first task)

**Reuses:** Nothing — pure new domain code

**Requirement:** ANNO-01 through ANNO-20 (all requirements depend on this model)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `Annotation { id: Uuid, kind: AnnotationKind, bounds: Rect, style: AnnotationStyle }` defined
- [x] `AnnotationKind` enum with all 12 variants: `Rectangle`, `Ellipse`, `Arrow(ArrowData)`, `Line(ArrowData)`, `Freehand(FreehandData)`, `Text(TextData)`, `Blur`, `Pixelate`, `Redaction`, `Timestamp(TextData)`, `NumberMarker(NumberMarkerData)`, `Callout(CalloutData)`
- [x] `AnnotationStyle { stroke_color: Color, stroke_width: f32, fill_color: Option<Color> }` defined
- [x] Coordinate types defined: `Rect { x, y, width, height: f64 }`, `Point { x, y: f64 }`, `Color { r, g, b, a: f64 }`
- [x] Sub-structs defined: `ArrowData { start, end: Point }`, `FreehandData { points: Vec<Point> }`, `TextData { text: String, font_size: f32 }`, `NumberMarkerData { number: u32 }`, `CalloutData { text: String, anchor: Point }`
- [x] `uuid` crate added to `Cargo.toml` (features: `v4`)
- [x] `AnnotationStyle::default()` returns stroke_color red (1,0,0,1), stroke_width 2.0, fill_color None
- [x] Unit tests cover: construct each variant, verify field access, `AnnotationStyle::default()` values
- [x] Gate check passes: `cargo test --lib`

**Tests:** unit  
**Gate:** quick (`cargo test --lib`)

**Verify:**

```bash
cargo test --lib
# Expected: all tests pass; new tests for model included
```

**Commit:** `feat(annotations): add annotation data model`

---

### T2: Undo/Redo History

**What:** Create `src/annotations/history.rs` with `AnnotationCommand` enum and `History` struct implementing the command-pattern undo/redo mechanism.

**Where:** `src/annotations/history.rs` (new file)

**Depends on:** T1 (`AnnotationCommand` references `Annotation`, `Rect`, `AnnotationStyle`)

**Reuses:** Nothing

**Requirement:** ANNO-09, ANNO-10

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `AnnotationCommand` enum defined with variants: `Add(Annotation)`, `Remove(Annotation)`, `UpdateBounds { id: Uuid, old_bounds: Rect, new_bounds: Rect }`, `UpdateStyle { id: Uuid, old_style: AnnotationStyle, new_style: AnnotationStyle }`, `UpdateText { id: Uuid, old_text: String, new_text: String }`
- [x] `History { undo_stack: Vec<AnnotationCommand>, redo_stack: Vec<AnnotationCommand> }` defined with `new() -> Self`
- [x] `push(cmd)` appends to undo_stack and clears redo_stack
- [x] `undo(engine)` pops from undo_stack, applies inverse via engine, pushes inverse to redo_stack; returns `true` if operation was performed
- [x] `redo(engine)` pops from redo_stack, re-applies via engine, pushes to undo_stack; returns `true`
- [x] `can_undo()` / `can_redo()` return correct booleans
- [x] Undo inverse mapping is correct: `Add` → engine.remove; `Remove` → engine.add; `UpdateBounds` → engine.update_bounds(old); `UpdateStyle` → engine.update_style(old); `UpdateText` → engine.update_text(old)
- [x] Unit tests cover: push/undo/redo cycle, double-undo past empty (returns false), new-push clears redo stack
- [x] Gate check passes: `cargo test --lib`

**Tests:** unit  
**Gate:** quick (`cargo test --lib`)

**Verify:**

```bash
cargo test --lib
# Expected: all prior tests pass; new history tests included
```

**Commit:** `feat(annotations): add undo/redo history with command pattern`

---

### T3: Annotation Engine

**What:** Create `src/annotations/engine.rs` with `AnnotationEngine` providing CRUD, selection, hit-testing, and number-marker counter.

**Where:** `src/annotations/engine.rs` (new file)

**Depends on:** T1, T2 (`AnnotationEngine` is referenced by `History::undo/redo`)

**Reuses:** Nothing

**Requirement:** ANNO-01 through ANNO-08, ANNO-19

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `AnnotationEngine` struct holds `annotations: Vec<Annotation>`, `selected_id: Option<Uuid>`, `next_number: u32`
- [x] `add(ann: Annotation)` pushes to the list
- [x] `remove(id: Uuid) -> Option<Annotation>` removes by id and returns it (needed by history for undo)
- [x] `update_bounds(id, new_bounds: Rect)` finds annotation by id and updates its `bounds`
- [x] `update_style(id, new_style: AnnotationStyle)` updates style
- [x] `update_text(id, new_text: String)` updates text inside `TextData`/`CalloutData` variants (no-op for non-text types)
- [x] `select(id: Uuid)` / `deselect()` update `selected_id`
- [x] `selected_id() -> Option<Uuid>` / `get_selected() -> Option<&Annotation>` accessors
- [x] `hit_test(p: Point) -> Option<Uuid>` — iterates annotations in reverse order (topmost = last-added wins); returns id of first annotation whose `bounds` contains `p` (simple AABB test for v1)
- [x] `next_number() -> u32` returns current counter then increments it
- [x] `reset_number_counter()` sets counter back to 1
- [x] `all() -> &[Annotation]` returns the slice
- [x] Unit tests cover: add/remove, hit_test (overlapping annotations returns topmost), select/deselect, next_number increments, reset_number_counter
- [x] Gate check passes: `cargo test --lib`

**Tests:** unit  
**Gate:** quick (`cargo test --lib`)

**Verify:**

```bash
cargo test --lib
# Expected: all prior tests pass; new engine tests included
```

**Commit:** `feat(annotations): add annotation engine with CRUD, selection, hit-testing`

---

### T4: Active Tool State

**What:** Create `src/annotations/tool.rs` with `ActiveTool` and `DrawingState` enums.

**Where:** `src/annotations/tool.rs` (new file)

**Depends on:** T1 (`DrawingState` references `Point`, `Rect`, `Uuid`, `HandleIndex`)

**Reuses:** Nothing

**Requirement:** ANNO-01 through ANNO-20 (all tools use these enums)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `ActiveTool` enum with 13 variants: `Select`, `Rectangle`, `Ellipse`, `Arrow`, `Line`, `Freehand`, `Text`, `Blur`, `Pixelate`, `Redaction`, `Timestamp`, `NumberMarker`, `Callout`
- [x] `ActiveTool` derives `Copy`, `Clone`, `PartialEq`, `Default` (default = `Select`)
- [x] `DrawingState` enum with variants: `Idle`, `Drawing { start: Point, current: Point }`, `Moving { id: Uuid, drag_start: Point, original_bounds: Rect }`, `ResizingHandle { id: Uuid, handle: HandleIndex, original_bounds: Rect, drag_start: Point }`, `EditingText { existing_id: Option<Uuid>, position: Point }`
- [x] `HandleIndex` enum: `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight` — derives `Copy`, `Clone`
- [x] `DrawingState` derives `Default` (default = `Idle`)
- [x] Gate check passes: `cargo build`

**Tests:** none (trivial enums; no GTK but no meaningful behavior to unit-test)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: zero errors related to T4 changes
```

**Commit:** `feat(annotations): add ActiveTool and DrawingState enums`

---

### T5: Annotations Module Exports

**What:** Create `src/annotations/mod.rs` re-exporting all public annotation types, and add `pub mod annotations;` to `src/lib.rs`.

**Where:** `src/annotations/mod.rs` (new file), `src/lib.rs` (modify)

**Depends on:** T1, T2, T3, T4

**Reuses:** Existing `src/models/mod.rs` export pattern

**Requirement:** All (enables other modules to use annotation types)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `src/annotations/mod.rs` exists with `mod model; mod history; mod engine; mod tool;` and re-exports: `pub use model::*; pub use history::*; pub use engine::*; pub use tool::*;`
- [x] `src/lib.rs` contains `pub mod annotations;`
- [x] Other modules can import annotation types via `use crate::annotations::Annotation;` etc.
- [x] Gate check passes: `cargo build`

**Tests:** none (module wiring)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: zero errors, all annotation types accessible from crate root
```

**Commit:** `feat(annotations): export annotations module`

---

### T6: Extend Canvas State and Public Annotation API

**What:** Extend `canvas/imp.rs` with annotation state fields, and add public annotation methods + coordinate-transform helpers to `canvas/mod.rs`.

**Where:** `src/canvas/imp.rs` (modify), `src/canvas/mod.rs` (modify)

**Depends on:** T5 (needs `AnnotationEngine`, `History`, `ActiveTool`, `DrawingState`, `AnnotationStyle`)

**Reuses:**

- Existing `RefCell<Option<Box<dyn Fn(f64)>>>` callback field pattern (for `annotation_changed_cb`)
- Existing `Cell<f64>` pattern (for `active_tool`)

**Requirement:** ANNO-05 through ANNO-12 (selection, undo/redo, style all flow through Canvas API)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `src/canvas/imp.rs` has new fields in `Canvas` struct:
  - `annotations: RefCell<AnnotationEngine>` — default `AnnotationEngine::new()`
  - `history: RefCell<History>` — default `History::new()`
  - `active_tool: Cell<ActiveTool>` — default `ActiveTool::Select`
  - `drawing_state: RefCell<DrawingState>` — default `DrawingState::Idle`
  - `current_style: RefCell<AnnotationStyle>` — default `AnnotationStyle::default()`
  - `annotation_changed_cb: RefCell<Option<Box<dyn Fn()>>>`
- [x] `Default` impl for `Canvas` initializes all new fields correctly
- [x] `src/canvas/mod.rs` has new public methods:
  - `set_active_tool(tool: ActiveTool)` — stores in `imp.active_tool`, resets cursor, queues draw
  - `set_current_style(style: AnnotationStyle)` — stores in `imp.current_style`
  - `undo() -> bool` — delegates to `imp.history.borrow_mut().undo(&mut imp.annotations.borrow_mut())`, calls `queue_draw()`, fires `annotation_changed_cb`
  - `redo() -> bool` — same pattern
  - `can_undo() -> bool` / `can_redo() -> bool`
  - `on_annotation_changed(cb: impl Fn() + 'static)`
- [x] Private helpers added:
  - `fn screen_to_image(&self, x: f64, y: f64) -> Point` — `Point { x: (x - pan_x) / zoom, y: (y - pan_y) / zoom }`
  - `fn image_to_screen(&self, p: Point) -> (f64, f64)` — `(p.x * zoom + pan_x, p.y * zoom + pan_y)`
  - `fn handle_at(&self, bounds: &Rect, p: Point) -> Option<HandleIndex>` — checks 8px radius around each corner handle in image-space
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: zero errors; existing tests still pass
cargo test --lib
```

**Commit:** `feat(canvas): extend Canvas state and public annotation API`

---

### T7: Annotation Renderer Module

**What:** Create `src/canvas/renderer.rs` with Cairo draw functions for all 12 annotation types and selection handles.

**Where:** `src/canvas/renderer.rs` (new file)

**Depends on:** T6 (needs model types; Canvas holds the pixbuf reference for effect renderers)

**Reuses:**

- Cairo `cr` pattern already established in `canvas/mod.rs` draw closure
- `gdk_pixbuf::Pixbuf` already in scope (used for image rendering)

**Requirement:** ANNO-01 through ANNO-20

**Tools:**

- MCP: context7 (verify `cairo-rs` API for pattern/pixbuf operations if needed)
- Skill: NONE

**Done when:**

- [x] `draw_all(cr, annotations, selected_id, source_pixbuf_opt, zoom, pan_x, pan_y)` function exists and:
  - Saves/restores Cairo state around the entire call
  - Applies `cr.translate(pan_x, pan_y); cr.scale(zoom, zoom)` so all sub-functions work in image-space
  - Renders annotations in correct layer order: effects (Blur/Pixelate/Redaction) → shapes (Rect/Ellipse/Arrow/Line/Freehand) → text-based (Text/Timestamp/NumberMarker/Callout)
  - Calls `draw_selection_handles` for the selected annotation after all annotations
- [x] Shape renderers produce correct Cairo paths:
  - `draw_rectangle`: `cr.rectangle(x, y, w, h)` with stroke only (no fill by default)
  - `draw_ellipse`: `cr.arc(cx, cy, rx, ry, ...)` with stroke
  - `draw_line`: `cr.move_to` + `cr.line_to`
  - `draw_arrow`: same as line plus filled triangle arrowhead at `end` oriented along `end–start` vector
  - `draw_freehand`: polyline through all `FreehandData::points`
- [x] Effect renderers:
  - `draw_redaction`: `cr.rectangle` with solid fill (no stroke) using `stroke_color` as fill
  - `draw_blur`: sub-pixbuf at bounds region, scaled to 1/8 then back to original size (bilinear), painted via `cr.set_source_pixbuf`
  - `draw_pixelate`: same but scale to 1/16 and back using nearest-neighbor (`Filter::Nearest`)
  - When `source_pixbuf_opt` is `None`, effects fall back to rendering bounds as a filled placeholder rect
- [x] Text-based renderers produce visible output (exact Pango layout can be rough for now — must compile and show text):
  - `draw_text` / `draw_timestamp`: `cr.move_to` + `pango` layout with `style.stroke_color`
  - `draw_number_marker`: circle stroke + centered number text
  - `draw_callout`: rounded rect + pointer triangle + text
- [x] `draw_selection_handles`: draws 8×8px filled squares at the 4 corners of the bounds (in image-space, before the zoom transform is applied, so handles are fixed-pixel-size on screen — use `cr.save()`/`cr.restore()` and inverse-scale the handle size)
- [x] All functions apply `style.stroke_color` via `cr.set_source_rgba(r, g, b, a)` and `style.stroke_width` via `cr.set_line_width`
- [x] `renderer.rs` is declared in `canvas/mod.rs` with `mod renderer;`
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas rendering layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: zero errors; renderer module compiles cleanly
```

**Commit:** `feat(canvas): add annotation renderer module with Cairo draw functions`

---

### T8: Extend Canvas Draw Function to Render Annotations

**What:** Update the `set_draw_func` closure in `Canvas::new()` to call `renderer::draw_all` after painting the source image.

**Where:** `src/canvas/mod.rs` (modify draw closure)

**Depends on:** T7 (renderer module must exist)

**Reuses:** Existing `set_draw_func` closure structure; passes `imp.image.borrow()` pixbuf reference

**Requirement:** ANNO-01 through ANNO-20 (visual output depends on this)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] After the `cr.paint()` call that renders the source image, the draw closure calls `renderer::draw_all(...)` with: the Cairo context, the annotation list from `imp.annotations.borrow().all()`, `imp.annotations.borrow().selected_id()`, the source pixbuf option, and the current `zoom`, `pan_x`, `pan_y`
- [x] The empty-canvas branch (no image) still renders the dark background; annotations are not rendered when no image is loaded
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas rendering layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
cargo run
# Manual: load an image — canvas renders normally with no regressions
```

**Commit:** `feat(canvas): integrate annotation renderer into Canvas draw function`

---

### T9: Drawing and Selection/Move Gesture

**What:** Add a `GestureDrag` (button=1) to `Canvas::new()` that handles annotation creation (when a drawing tool is active) and annotation selection + move (when the Select tool is active).

**Where:** `src/canvas/mod.rs` (add inside `Canvas::new()`)

**Depends on:** T8 (draw func exists; T3 engine + T4 tool state in T6's Canvas)

**Reuses:**

- Existing `GestureDrag` (button=2) pattern from middle-mouse pan — identical structure
- `screen_to_image` helper from T6

**Requirement:** ANNO-01 through ANNO-08, ANNO-11 through ANNO-19

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `GestureDrag` with `set_button(1)` added inside `Canvas::new()`
- [x] `connect_drag_begin`:
  - Converts cursor position to image-space via `screen_to_image`
  - If `active_tool == Select`: calls `engine.hit_test(p)`:
    - If hit returns `Some(id)`: calls `handle_at(bounds, p)` — if on a handle, sets `DrawingState::ResizingHandle`; otherwise sets `DrawingState::Moving { id, drag_start, original_bounds }`
    - If hit returns `None`: calls `engine.deselect()`, sets `DrawingState::Idle`, `queue_draw()`
  - If any drawing tool: sets `DrawingState::Drawing { start: p, current: p }`
  - For text/timestamp/number-marker/callout: waits for `drag_end` with small movement to determine if click vs drag
- [x] `connect_drag_update`:
  - Computes new absolute position from drag start + offset; converts to image-space
  - If `DrawingState::Drawing`: updates `current` point; calls `queue_draw()`
  - If `DrawingState::Moving`: computes delta from `drag_start`; calls `engine.update_bounds` with `original_bounds` offset by delta; calls `queue_draw()` (no history push yet — history pushed on drag_end)
  - If `DrawingState::ResizingHandle`: handled in T10
- [x] `connect_drag_end`:
  - If `DrawingState::Drawing` and distance ≥ 4px:
    - Constructs the final `Annotation` from start/current (calls `engine.next_number()` for `NumberMarker`)
    - For `Text`/`Callout`: opens text editor dialog (see T12); annotation creation deferred to dialog confirm
    - For `Timestamp`: creates `TextData` with formatted `chrono::Local::now()` string
    - For other tools: calls `engine.add(ann)` + `history.push(AnnotationCommand::Add(ann.clone()))` + `queue_draw()` + fires `annotation_changed_cb`
  - If `DrawingState::Drawing` and distance < 4px: discard (set state to `Idle`)
  - If `DrawingState::Moving`: pushes `AnnotationCommand::UpdateBounds { old: original_bounds, new: current_bounds }` to history; fires `annotation_changed_cb`
  - Resets `drawing_state` to `Idle`
- [x] A live preview annotation is rendered during drag: the draw function uses `drawing_state` to render a dashed/ghost version of the in-progress annotation
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas interaction layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
cargo run
# Manual: select Rectangle tool; drag on loaded image → rectangle appears; release → annotation persists
# Manual: switch to Select tool; click annotation → selection handles appear; drag it → it moves
```

**Commit:** `feat(canvas): add drawing and selection/move gesture`

---

### T10: Resize Handles Gesture

**What:** Extend the `GestureDrag` `drag_update` and `drag_end` handlers (from T9) to implement resize-handle dragging for selected annotations.

**Where:** `src/canvas/mod.rs` (modify existing gesture handlers added in T9)

**Depends on:** T9 (gesture framework exists; `DrawingState::ResizingHandle` set in T9's `drag_begin`)

**Reuses:** `handle_at` helper, `screen_to_image` helper, existing `engine.update_bounds` method

**Requirement:** ANNO-07

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] In `connect_drag_update`, the `DrawingState::ResizingHandle { id, handle, original_bounds, drag_start }` branch:
  - Computes delta in image-space from `drag_start` to current
  - Computes new bounds based on which `handle` was dragged (opposite corner is fixed):
    - `TopLeft`: new x = original_x + delta_x, new y = original_y + delta_y, new width = original_w - delta_x, new height = original_h - delta_y
    - `TopRight` / `BottomLeft` / `BottomRight`: analogous
  - Clamps width and height to minimum 4px
  - Calls `engine.update_bounds(id, new_bounds)`
  - Calls `queue_draw()`
- [x] In `connect_drag_end`, the `DrawingState::ResizingHandle` branch pushes `AnnotationCommand::UpdateBounds { old: original_bounds, new: current_bounds }` to history and fires `annotation_changed_cb`
- [x] Arrow/Line annotations resize by updating `ArrowData::start` and `ArrowData::end` proportionally to the bounds change; Freehand points are scaled proportionally
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas interaction layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
cargo run
# Manual: draw a rectangle, select it, drag a corner handle → it resizes correctly with opposite corner fixed
```

**Commit:** `feat(canvas): implement annotation resize via selection handles`

---

### T11: Keyboard Controller — Delete and Escape

**What:** Add a `EventControllerKey` to `Canvas::new()` that handles Delete (remove selected annotation) and Escape (deselect).

**Where:** `src/canvas/mod.rs` (add inside `Canvas::new()`)

**Depends on:** T9 (selection state in engine; annotation_changed_cb established)

**Reuses:** No existing key controller in canvas; follows same widget controller pattern

**Requirement:** ANNO-08 (Delete), ANNO-05 (Escape deselect)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `EventControllerKey` added to canvas with `canvas.add_controller(key_ctrl)`
- [x] On `gdk::Key::Delete`:
  - If `engine.selected_id()` is Some: calls `engine.remove(id)` → stores removed annotation → pushes `AnnotationCommand::Remove(ann)` to history → `queue_draw()` → fires `annotation_changed_cb`
  - If no selection: no-op
- [x] On `gdk::Key::Escape`:
  - Calls `engine.deselect()`; resets `drawing_state` to `Idle`; `queue_draw()`
- [x] Canvas receives keyboard focus correctly (call `canvas.set_focusable(true)` in `new()`)
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas interaction layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
cargo run
# Manual: draw annotation, select it, press Delete → it disappears; Ctrl+Z → it comes back
# Manual: press Escape on selected annotation → deselects (handles disappear)
```

**Commit:** `feat(canvas): add keyboard controller for Delete and Escape`

---

### T12: Text Annotation Editor

**What:** Add text-annotation dialog support to the canvas — a modal `adw::MessageDialog` (or `gtk::AlertDialog`) with a `gtk::Entry` for text input — triggered for Text, Callout, and double-click re-edit flows.

**Where:** `src/canvas/mod.rs` (add `open_text_editor` method + hook into T9's drag_end logic) + minimal `src/ui/text_editor.rs` if dialog logic warrants a separate helper

**Depends on:** T9 (drag_end signals need-text-editor for Text/Callout tools), T10 (resize complete)

**Reuses:**

- `adw::MessageDialog` pattern (verify availability in gtk4-rs version in project)
- `canvas.root()` to get parent window for dialog modal attachment

**Requirement:** ANNO-14 (Text), ANNO-20 (Callout)

**Tools:**

- MCP: context7 (verify `adw::MessageDialog` with entry widget API in current libadwaita-rs version)
- Skill: NONE

**Done when:**

- [x] `Canvas::open_text_editor(position: Point, existing_id: Option<Uuid>)` method exists
- [x] Method creates a dialog with a `gtk::Entry` pre-filled with existing text (if `existing_id` is Some)
- [x] On confirm with non-empty text:
  - For new annotation (`existing_id` = None): creates `Annotation` with `AnnotationKind::Text(TextData { text, font_size: 16.0 })`, calls `engine.add()`, pushes `AnnotationCommand::Add`, fires `annotation_changed_cb`, `queue_draw()`
  - For edit (`existing_id` = Some): pushes `AnnotationCommand::UpdateText { old, new }`, calls `engine.update_text()`, fires `annotation_changed_cb`, `queue_draw()`
- [x] On confirm with empty text: discards silently
- [x] On cancel/Escape: discards silently (no annotation created or modified)
- [x] `GestureClick (n_press=2)` added in `Canvas::new()`:
  - On double-click: hits test at click position; if the annotation is `Text` or `Callout`, calls `open_text_editor(position, Some(id))`
- [x] Text tool drag_end (from T9) calls `open_text_editor(start_position, None)`
- [x] Gate check passes: `cargo build`

**Tests:** none (canvas + UI layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
cargo run
# Manual: select Text tool, click canvas → dialog opens; type text, confirm → label appears
# Manual: double-click existing text annotation → dialog opens pre-filled; edit and confirm → text updates
```

**Commit:** `feat(canvas): add text annotation editor dialog`

---

### T13: Tool Palette Widget [P]

**What:** Create `src/ui/tool_palette.rs` — a GTK4 vertical widget with one toggle button per annotation tool.

**Where:** `src/ui/tool_palette.rs` (new file)

**Depends on:** T4 (needs `ActiveTool` enum)

**Reuses:**

- Existing `gtk::ToggleButton` usage pattern (if any); otherwise follows GTK4-rs widget creation pattern
- `src/ui/mod.rs` to add `mod tool_palette;`

**Requirement:** ANNO-01 through ANNO-20 (tool palette enables all tools)

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `ToolPalette` struct wraps a `gtk::Box` (orientation: `Vertical`)
- [x] One `gtk::ToggleButton` per `ActiveTool` variant (13 buttons), grouped so only one is active at a time (use `set_group` chain: each button added to the previous as its group)
- [x] Buttons have descriptive labels or icon names (labels acceptable for v1: "Sel", "Rect", "Ellipse", "Arrow", "Line", "Free", "Text", "Blur", "Pix", "Red", "Time", "Num", "Call")
- [x] `ToolPalette::widget() -> &gtk::Widget` returns the underlying box (for packing into window)
- [x] `ToolPalette::on_tool_changed(cb: impl Fn(ActiveTool) + 'static)` stores callback; each toggle button's `connect_toggled` calls the callback when activated
- [x] `ToolPalette::set_active_tool(tool: ActiveTool)` programmatically activates the matching toggle button
- [x] `src/ui/mod.rs` contains `pub mod tool_palette;` and `pub use tool_palette::ToolPalette;`
- [x] Gate check passes: `cargo build`

**Tests:** none (UI layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: zero errors; ToolPalette compiles
```

**Commit:** `feat(ui): add tool palette widget`

---

### T14: Style Controls in Tool Palette

**What:** Extend `ToolPalette` with a color button and stroke-width spinner, and expose callbacks for style changes.

**Where:** `src/ui/tool_palette.rs` (modify)

**Depends on:** T13 (`ToolPalette` struct exists)

**Reuses:** `gtk::ColorButton`, `gtk::SpinButton` (standard GTK4 widgets)

**Requirement:** ANNO-11, ANNO-12

**Tools:**

- MCP: context7 (verify `gtk::ColorButton` or `gtk::ColorDialog` API in current gtk4-rs version; use whichever is available)
- Skill: NONE

**Done when:**

- [x] `gtk::Separator` added between tool buttons and style controls
- [x] Color button added (prefer `gtk::ColorButton`; fallback to label + custom if unavailable):
  - Default color: red (RGBA 1, 0, 0, 1)
  - `ToolPalette::on_color_changed(cb: impl Fn(Color) + 'static)` — fires with RGBA values when color is picked
- [x] Stroke width `gtk::SpinButton` added:
  - Range: 1.0 to 20.0, step 1.0, default 2.0
  - `ToolPalette::on_stroke_changed(cb: impl Fn(f32) + 'static)` — fires with new value
- [x] Gate check passes: `cargo build`

**Tests:** none (UI layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: style controls compile
```

**Commit:** `feat(ui): add color and stroke-width controls to tool palette`

---

### T15: Undo/Redo GActions and Keyboard Accelerators [P]

**What:** Register `win.undo` and `win.redo` GActions in the window and add their keyboard accelerators to `application.rs`.

**Where:** `src/ui/window/imp.rs` (add actions in `constructed()`), `src/application.rs` (add accels in `startup()`)

**Depends on:** T6 (Canvas has `undo()`, `redo()`, `can_undo()`, `can_redo()` public methods)

**Reuses:**

- Existing `gio::SimpleAction` + `actions.add_action()` pattern (identical to zoom actions)
- Existing `startup()` override in `application.rs` with `set_accels_for_action`

**Requirement:** ANNO-09, ANNO-10

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] In `MainWindow::constructed()` (or `setup_actions`): two new actions registered:
  - `gio::SimpleAction::new("undo", None)` — on activate: calls `canvas.undo()`
  - `gio::SimpleAction::new("redo", None)` — on activate: calls `canvas.redo()`
  - Both added to the existing `actions` group
- [x] `canvas.on_annotation_changed()` callback updates both actions' enabled state via `action.set_enabled(canvas.can_undo())` / `action.set_enabled(canvas.can_redo())`
- [x] Initial enabled state: both disabled (no annotations yet)
- [x] In `startup()` override in `application.rs`:
  - `app.set_accels_for_action("win.undo", &["<Control>z"])` registered
  - `app.set_accels_for_action("win.redo", &["<Control>y", "<Control><Shift>z"])` registered
- [x] Gate check passes: `cargo build`

**Tests:** none (UI + application layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
# Expected: zero errors
```

**Commit:** `feat(window): add undo/redo GActions and keyboard shortcuts`

---

### T16: Wire Tool Palette to Canvas in Window

**What:** Pack the `ToolPalette` widget into the main window layout and connect its tool/style callbacks to the canvas.

**Where:** `src/ui/window/imp.rs` (add `ToolPalette` field + connect callbacks in `constructed()`)

**Depends on:** T13, T14 (ToolPalette complete), T12 (Canvas has `set_active_tool`, `set_current_style`)

**Reuses:**

- Existing window layout (currently a simple `gtk::Box` or `adw::ToolbarView`); wrap in a horizontal `gtk::Box` with palette on left + canvas on right
- `canvas_weak` downgrade pattern from existing event controllers

**Requirement:** ANNO-01 through ANNO-20

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `MainWindow` imp struct has `tool_palette: OnceCell<ToolPalette>` field
- [x] `ToolPalette::new()` called in `constructed()`; stored in `self.tool_palette`
- [x] Window main layout is a horizontal `gtk::Box`: palette widget on the left side, canvas on the right (canvas fills remaining space via `set_hexpand(true)`)
- [x] `tool_palette.on_tool_changed(...)` callback calls `canvas.set_active_tool(tool)`
- [x] `tool_palette.on_color_changed(...)` and `tool_palette.on_stroke_changed(...)` callbacks construct an updated `AnnotationStyle` from current style + new value and call `canvas.set_current_style(style)`
- [x] Gate check passes: `cargo build`

**Tests:** none (UI layer)  
**Gate:** build (`cargo build`)

**Verify:**

```bash
cargo build
cargo run
# Manual: window shows tool palette on the left; clicking a tool changes the active annotation mode
```

**Commit:** `feat(window): wire tool palette to canvas`

---

### T17: Wire Undo/Redo Actions to Canvas History

**What:** Connect the `win.undo` and `win.redo` actions (added in T15) to the canvas's actual undo/redo methods and update action enabled state.

**Where:** `src/ui/window/imp.rs` (connect action callbacks in `constructed()`)

**Depends on:** T15 (actions registered), T16 (canvas accessible in window)

**Reuses:** Existing action connect pattern; canvas reference from window

**Requirement:** ANNO-09, ANNO-10

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `win.undo` action's `connect_activate` calls `canvas.undo()`; triggers `queue_draw()` (already done inside `canvas.undo()`)
- [x] `win.redo` action's `connect_activate` calls `canvas.redo()`
- [x] `canvas.on_annotation_changed()` fires after every operation (add, delete, move, resize, style change, undo, redo) and updates both actions' `set_enabled` state
- [x] Both actions start disabled and become enabled/disabled reactively as the undo/redo stacks change
- [x] Gate check passes: `cargo build && cargo test --lib`

**Tests:** none (UI layer); full gate to catch any regressions in pure-Rust unit tests  
**Gate:** full (`cargo build && cargo test --lib`)

**Verify:**

```bash
cargo build && cargo test --lib
# Expected: all existing unit tests pass
cargo run
# Manual: draw annotation → Ctrl+Z undoes it → Ctrl+Y redoes it
# Manual: undo/redo toolbar buttons (if added) enable/disable correctly
```

**Commit:** `feat(window): connect undo/redo actions to canvas history`

---

### T18: Final Integration and Full Gate

**What:** Validate the full annotation workflow end-to-end: all 12 tools create annotations, select/move/resize/delete work, undo/redo round-trips are correct, style changes apply. Fix any integration issues discovered. Run the full gate check.

**Where:** Any files needing fixes

**Depends on:** T17 (all previous tasks complete)

**Reuses:** All components built in T1–T17

**Requirement:** All ANNO-01 through ANNO-20

**Tools:**

- MCP: NONE
- Skill: NONE

**Done when:**

- [x] `cargo build && cargo test --lib` passes with zero errors and zero test regressions
- [x] Manual smoke test passes:
  - Load a screenshot
  - Draw one annotation of each of the 12 types — all render visibly on canvas
  - Select an annotation → handles appear
  - Move it → it moves correctly
  - Resize it → bounds change correctly
  - Delete it → it disappears
  - Ctrl+Z → it comes back
  - Ctrl+Y → it disappears again
  - Change color on selected annotation → color updates immediately
  - App does not crash during any of the above
- [x] No console errors or `unwrap` panics during normal usage
- [x] `cargo test --lib` test count is ≥ the count before this feature (no regressions, new annotation model/engine/history tests included)

**Tests:** unit (regression check) + manual  
**Gate:** full (`cargo build && cargo test --lib`)

**Verify:**

```bash
cargo build && cargo test --lib
# Expected: build succeeds, all tests pass
cargo run
# Manual: full smoke test per "Done when" checklist above
```

**Commit:** `feat(annotations): complete annotation system — all 12 types, undo/redo, style controls`

---

## Parallel Execution Map

```
Phase 1 (Sequential — pure Rust data foundation):
  T1 ──→ T2 ──→ T3 ──→ T4 ──→ T5

Phase 2a (Canvas work — sequential, one file at a time):
  T5 ──→ T6 ──→ T7 ──→ T8 ──→ T9 ──→ T10 ──→ T11 ──→ T12

Phase 2b (Tool palette — parallel after T4):
  T4 ──→ T13 [P] ──→ T14

Phase 2c (Undo/redo actions — parallel after T6):
  T6 ──→ T15 [P]

Phase 3 (Integration — sequential after all of Phase 2):
  T12 + T14 + T15 ──→ T16 ──→ T17 ──→ T18
```

**Parallelism notes:**

- T13 `[P]` can start as soon as T4 is complete (needs only `ActiveTool` enum). Since T4 is done in Phase 1, T13 can run while Phase 2a proceeds.
- T15 `[P]` can start as soon as T6 is complete (Canvas public API including `undo()`/`redo()`). Since T6 precedes T7, T15 can run while T7–T12 proceed.
- T14 is sequential after T13 (same file).
- Phase 2a tasks T6–T12 all modify `canvas/mod.rs` or closely related canvas files — they cannot be parallelized without conflicts.

---

## Task Granularity Check


| Task                                 | Scope                                  | Status                                                                                                             |
| ------------------------------------ | -------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| T1: Annotation data model            | 1 new file, all domain structs         | ⚠️ Cohesive (multiple structs in 1 file — all part of the same model; cannot split without creating circular deps) |
| T2: Undo/redo history                | 1 new file                             | ✅ Granular                                                                                                         |
| T3: Annotation engine                | 1 new file                             | ✅ Granular                                                                                                         |
| T4: Tool state enums                 | 1 new file                             | ✅ Granular                                                                                                         |
| T5: Module exports                   | 2 files (mod.rs + lib.rs modification) | ✅ Granular                                                                                                         |
| T6: Canvas state + public API        | 2 related files (`imp.rs` + `mod.rs`)  | ⚠️ Cohesive (state and API are tightly coupled; splitting would require interim broken state)                      |
| T7: Annotation renderer              | 1 new file, all render functions       | ⚠️ Cohesive (12 renderers in 1 file; could split but all belong to the "renderer" component)                       |
| T8: Draw function extension          | 1 file, 5–10 lines addition            | ✅ Granular                                                                                                         |
| T9: Drawing + selection/move gesture | 1 file, 1 GestureDrag controller       | ✅ Granular                                                                                                         |
| T10: Resize gesture                  | 1 file, extend T9's gesture            | ✅ Granular                                                                                                         |
| T11: Keyboard controller             | 1 file, 1 EventControllerKey           | ✅ Granular                                                                                                         |
| T12: Text editor                     | 1–2 files, 1 feature                   | ✅ Granular                                                                                                         |
| T13: Tool palette widget             | 1 new file                             | ✅ Granular                                                                                                         |
| T14: Style controls                  | 1 file (modify T13)                    | ✅ Granular                                                                                                         |
| T15: Undo/redo GActions + accels     | 2 files (window + app)                 | ✅ Granular                                                                                                         |
| T16: Wire palette to canvas          | 1 file (window imp)                    | ✅ Granular                                                                                                         |
| T17: Wire undo/redo to canvas        | 1 file (window imp)                    | ✅ Granular                                                                                                         |
| T18: Final integration               | Any files needing fixes                | ✅ Integration task                                                                                                 |


---

## Diagram-Definition Cross-Check


| Task    | Depends On (task body) | Diagram Shows              | Status  |
| ------- | ---------------------- | -------------------------- | ------- |
| T1      | None                   | No incoming arrow          | ✅ Match |
| T2      | T1                     | T1 → T2                    | ✅ Match |
| T3      | T1, T2                 | T2 → T3                    | ✅ Match |
| T4      | T1                     | T3 → T4 (via T1 dep)       | ✅ Match |
| T5      | T1, T2, T3, T4         | T4 → T5                    | ✅ Match |
| T6      | T5                     | T5 → T6                    | ✅ Match |
| T7      | T6                     | T6 → T7                    | ✅ Match |
| T8      | T7                     | T7 → T8                    | ✅ Match |
| T9      | T8                     | T8 → T9                    | ✅ Match |
| T10     | T9                     | T9 → T10                   | ✅ Match |
| T11     | T9                     | T10 → T11                  | ✅ Match |
| T12     | T9, T10                | T11 → T12                  | ✅ Match |
| T13 [P] | T4                     | T4 → T13 (parallel branch) | ✅ Match |
| T14     | T13                    | T13 → T14                  | ✅ Match |
| T15 [P] | T6                     | T6 → T15 (parallel branch) | ✅ Match |
| T16     | T13, T14, T12          | T12 + T14 → T16            | ✅ Match |
| T17     | T15, T16               | T15 + T16 → T17            | ✅ Match |
| T18     | T17                    | T17 → T18                  | ✅ Match |


---

## Test Co-location Validation


| Task | Code Layer Created/Modified                  | Matrix Requires       | Task Says           | Status                                         |
| ---- | -------------------------------------------- | --------------------- | ------------------- | ---------------------------------------------- |
| T1   | `src/annotations/model.rs`                   | unit (data models)    | unit                | ✅ OK                                           |
| T2   | `src/annotations/history.rs`                 | unit (data models)    | unit                | ✅ OK                                           |
| T3   | `src/annotations/engine.rs`                  | unit (data models)    | unit                | ✅ OK                                           |
| T4   | `src/annotations/tool.rs`                    | unit (data models)    | none (trivial enum) | ⚠️ Acceptable — no meaningful behavior to test |
| T5   | `src/annotations/mod.rs`, `src/lib.rs`       | none (wiring)         | none                | ✅ OK                                           |
| T6   | `src/canvas/imp.rs`, `src/canvas/mod.rs`     | none (UI layer)       | none                | ✅ OK                                           |
| T7   | `src/canvas/renderer.rs`                     | none (UI layer)       | none                | ✅ OK                                           |
| T8   | `src/canvas/mod.rs`                          | none (UI layer)       | none                | ✅ OK                                           |
| T9   | `src/canvas/mod.rs`                          | none (UI layer)       | none                | ✅ OK                                           |
| T10  | `src/canvas/mod.rs`                          | none (UI layer)       | none                | ✅ OK                                           |
| T11  | `src/canvas/mod.rs`                          | none (UI layer)       | none                | ✅ OK                                           |
| T12  | `src/canvas/mod.rs`                          | none (UI layer)       | none                | ✅ OK                                           |
| T13  | `src/ui/tool_palette.rs`                     | none (UI layer)       | none                | ✅ OK                                           |
| T14  | `src/ui/tool_palette.rs`                     | none (UI layer)       | none                | ✅ OK                                           |
| T15  | `src/ui/window/imp.rs`, `src/application.rs` | none (UI + app layer) | none                | ✅ OK                                           |
| T16  | `src/ui/window/imp.rs`                       | none (UI layer)       | none                | ✅ OK                                           |
| T17  | `src/ui/window/imp.rs`                       | none (UI layer)       | none                | ✅ OK                                           |
| T18  | Any                                          | unit + manual         | unit                | ✅ OK                                           |


**Note on T4:** `ActiveTool` and `DrawingState` are pure enums with no logic. The matrix requires "unit" for `src/annotations/` but T4 has no meaningful assertions to write beyond "it compiles". The T9 interaction tests (manual) implicitly validate all tool variants are handled.

---

## Requirement Coverage


| Requirement ID | Story             | Covered by Task(s)                                              |
| -------------- | ----------------- | --------------------------------------------------------------- |
| ANNO-01        | P1: Rectangle     | T1 (model), T7 (renderer), T9 (gesture)                         |
| ANNO-02        | P1: Ellipse       | T1, T7, T9                                                      |
| ANNO-03        | P1: Arrow         | T1, T7, T9                                                      |
| ANNO-04        | P1: Line          | T1, T7, T9                                                      |
| ANNO-05        | P1: Select        | T3 (engine), T9 (gesture), T11 (Escape)                         |
| ANNO-06        | P1: Move          | T3, T9                                                          |
| ANNO-07        | P1: Resize        | T3, T10                                                         |
| ANNO-08        | P1: Delete        | T3, T11                                                         |
| ANNO-09        | P1: Undo          | T2 (history), T6 (API), T15 (action), T17 (wiring)              |
| ANNO-10        | P1: Redo          | T2, T6, T15, T17                                                |
| ANNO-11        | P1: Color         | T1 (Color type), T6 (current_style), T14 (picker), T16 (wiring) |
| ANNO-12        | P1: Stroke width  | T1, T6, T14, T16                                                |
| ANNO-13        | P2: Freehand      | T1 (FreehandData), T7, T9                                       |
| ANNO-14        | P2: Text          | T1 (TextData), T7, T12                                          |
| ANNO-15        | P2: Blur          | T1, T7                                                          |
| ANNO-16        | P2: Pixelate      | T1, T7                                                          |
| ANNO-17        | P2: Redaction     | T1, T7, T9                                                      |
| ANNO-18        | P2: Timestamp     | T1 (TextData), T7, T9                                           |
| ANNO-19        | P2: Number Marker | T1 (NumberMarkerData), T3 (counter), T7, T9                     |
| ANNO-20        | P3: Callout       | T1 (CalloutData), T7, T9, T12                                   |


**Coverage:** 20/20 requirements mapped ✅