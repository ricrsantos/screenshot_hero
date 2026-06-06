# Annotations — Design

**Spec:** `.specs/features/annotations/spec.md`  
**Status:** Approved

---

## Architecture Overview

The Annotations feature introduces three new subsystems and extends the existing Canvas widget:

1. **`src/annotations/`** — Pure-Rust domain layer: data model, CRUD engine, undo/redo history, and tool state machine. No GTK dependency.
2. **`src/canvas/`** — Extended with Cairo annotation rendering, coordinate transforms, and new GTK4 event controllers for draw/select/move/resize.
3. **`src/ui/tool_palette.rs`** — New GTK4 widget for tool selection and style controls.

```
User Input (mouse/keyboard)
        ↓
Canvas Event Controllers
        ↓ screen-space → image-space (coord transform)
AnnotationEngine  ←→  History (undo/redo stack)
        ↓
Canvas::queue_draw()
        ↓
Canvas DrawFunc
    ├── cr.translate(pan_x, pan_y); cr.scale(zoom, zoom)
    ├── draw source image (existing)
    └── renderer::draw_all(cr, annotations, selected_id, pixbuf)
            ├── draw effects (Blur, Pixelate, Redaction) first
            ├── draw shapes (Rect, Ellipse, Arrow, Line, Freehand)
            ├── draw text-based types (Text, Timestamp, NumberMarker, Callout)
            └── draw_selection_handles(cr, selected_bounds)

ToolPalette (GTK4 widget)
    → on_tool_changed  → canvas.set_active_tool(tool)
    → on_color_changed → canvas.set_current_style(...)
    → on_stroke_changed→ canvas.set_current_style(...)

Window GActions
    win.undo → canvas.undo()
    win.redo → canvas.redo()
```

---

## Code Reuse Analysis

### Existing Components to Leverage

| Component | Location | How to Use |
|---|---|---|
| `Canvas` GTK4 widget | `src/canvas/imp.rs`, `src/canvas/mod.rs` | Extend `imp` with new fields; extend `new()` with gesture controllers; extend draw closure |
| `GestureDrag` (button=2) pan pattern | `src/canvas/mod.rs` | Apply identical pattern for GestureDrag (button=1) for annotation draw/move/resize |
| Zoom-to-cursor math (pan_x, pan_y, zoom) | `src/canvas/mod.rs` | Reuse `pan_offset` and `zoom` fields in coord-transform helpers |
| GAction + `SimpleActionGroup` pattern | `src/ui/window/imp.rs` | Register `win.undo`, `win.redo` using the exact same pattern as existing zoom actions |
| `set_accels_for_action` in `startup()` | `src/application.rs` | Append Ctrl+Z, Ctrl+Y, Ctrl+Shift+Z to the existing `startup()` override |
| `ImageData::pixbuf()` | `src/models/image.rs` | Pass to Blur/Pixelate renderers for source pixel access |
| `on_zoom_changed` callback pattern | `src/canvas/mod.rs` | Apply same `RefCell<Option<Box<dyn Fn()>>>` pattern for `on_annotation_changed` |

### Integration Points

| System | Integration Method |
|---|---|
| Canvas draw function | Extend existing closure: call `renderer::draw_all` after painting the source image |
| Window action group | Add `win.undo`, `win.redo` to existing `SimpleActionGroup` in `constructed()` |
| Application startup | Append undo/redo accels to existing `startup()` override |
| Tool palette ↔ Canvas | Callback closures stored on Canvas (same as `zoom_changed_cb`) |

---

## Components

### 1. Annotation Data Model

- **Purpose:** All annotation structs and coordinate types. Pure Rust — zero GTK dependency.
- **Location:** `src/annotations/model.rs`
- **Key types:**
  - `Annotation { id: Uuid, kind: AnnotationKind, bounds: Rect, style: AnnotationStyle }` — per ADR-002
  - `AnnotationKind` — 12-variant enum; variant-specific data stored inline
  - `AnnotationStyle { stroke_color: Color, stroke_width: f32, fill_color: Option<Color> }`
  - `Rect { x: f64, y: f64, width: f64, height: f64 }` — image-space coordinates
  - `Point { x: f64, y: f64 }` — image-space coordinates
  - `Color { r: f64, g: f64, b: f64, a: f64 }` — Cairo-compatible RGBA
- **Dependencies:** `uuid` crate (`Uuid::new_v4()`)

### 2. Annotation Engine

- **Purpose:** CRUD, selection state, hit-testing, number-marker counter. Owns the annotation list.
- **Location:** `src/annotations/engine.rs`
- **Key interfaces:**
  - `add(ann: Annotation)` — appends to list
  - `remove(id: Uuid) -> Option<Annotation>` — removes and returns (needed by history for undo)
  - `update_bounds(id, new_bounds: Rect)` / `update_style(id, new_style: AnnotationStyle)` / `update_text(id, new_text: String)`
  - `select(id: Uuid)` / `deselect()` / `selected_id() -> Option<Uuid>` / `get_selected() -> Option<&Annotation>`
  - `hit_test(p: Point) -> Option<Uuid>` — returns topmost annotation at point (last-added wins)
  - `next_number() -> u32` — auto-increments and returns the next marker number
  - `reset_number_counter()` — called by history when all markers are undone
  - `all() -> &[Annotation]`
- **Dependencies:** `model.rs`

### 3. Undo/Redo History

- **Purpose:** Command pattern storing reversible operations. Decoupled from GTK.
- **Location:** `src/annotations/history.rs`
- **Key types:**
  - `AnnotationCommand` enum — see Data Models section
  - `History { undo_stack: Vec<AnnotationCommand>, redo_stack: Vec<AnnotationCommand> }`
- **Key interfaces:**
  - `push(cmd: AnnotationCommand)` — appends to undo stack, clears redo stack
  - `undo(engine: &mut AnnotationEngine) -> bool` — pops undo, applies inverse, pushes inverse to redo
  - `redo(engine: &mut AnnotationEngine) -> bool` — pops redo, re-applies, pushes to undo
  - `can_undo() -> bool` / `can_redo() -> bool`
- **Dependencies:** `model.rs`, `engine.rs`

### 4. Tool State

- **Purpose:** Current active tool and gesture state machine. Pure Rust enum.
- **Location:** `src/annotations/tool.rs`
- **Key types:**
  - `ActiveTool` enum — 13 variants (Select + 12 annotation types)
  - `DrawingState` enum — see Data Models section
- **Dependencies:** `model.rs`

### 5. Annotation Renderer

- **Purpose:** Cairo draw functions for every annotation type and selection handles.
- **Location:** `src/canvas/renderer.rs` (new file)
- **Key interfaces:**
  - `draw_all(cr, annotations, selected_id, source_pixbuf, zoom, pan_x, pan_y)` — iterates in order; effects first, shapes second, text-based last, selection handles on top
  - `draw_selection_handles(cr, bounds, zoom, pan_x, pan_y)` — 8-handle pattern (corners + midpoints, v1 uses corners only)
- **Rendering strategy (per ADR-003):**
  - All coordinates are image-space. The draw function applies `cr.translate(pan_x, pan_y); cr.scale(zoom, zoom)` before calling renderer functions so renderers work in image-space.
  - Blur: extract pixbuf sub-region → scale down to 1/8 (bilinear) → scale back up (nearest-neighbor for mosaic feel; bilinear for blur) → paint onto canvas
  - Pixelate: same as blur but scale down to ~1/16 and back up with nearest-neighbor to create visible pixel grid
  - Arrow: line from `start` to `end`; arrowhead = filled triangle at `end`, oriented along the `end–start` vector
  - Number Marker: filled circle + `Pango` text centered inside (bold, white)
  - Callout: rounded-rect bubble + triangle pointer from bubble edge to `anchor` + Pango text
- **Dependencies:** `annotations/model.rs`, `gdk_pixbuf`

### 6. Extended Canvas

- **Purpose:** Add annotation state to the Canvas widget; wire all annotation interaction controllers.
- **Location:** `src/canvas/imp.rs` (new fields), `src/canvas/mod.rs` (draw func extension + new controllers + public API)
- **New fields in `Canvas` imp struct:**
  - `annotations: RefCell<AnnotationEngine>`
  - `history: RefCell<History>`
  - `active_tool: Cell<ActiveTool>` — default `ActiveTool::Select`
  - `drawing_state: RefCell<DrawingState>` — default `DrawingState::Idle`
  - `current_style: RefCell<AnnotationStyle>` — default red, stroke 2px, no fill
  - `annotation_changed_cb: RefCell<Option<Box<dyn Fn()>>>`
- **New public methods on `Canvas`:**
  - `set_active_tool(tool: ActiveTool)`
  - `set_current_style(style: AnnotationStyle)`
  - `undo() -> bool` / `redo() -> bool`
  - `can_undo() -> bool` / `can_redo() -> bool`
  - `on_annotation_changed(cb: impl Fn() + 'static)` — fired after any add/remove/move/resize/style-change
- **New private helpers:**
  - `fn screen_to_image(x: f64, y: f64) -> Point` — `(x - pan_x) / zoom`
  - `fn image_to_screen(p: Point) -> (f64, f64)` — `(p.x * zoom + pan_x, ...)`
  - `fn handle_at(bounds: &Rect, p: Point) -> Option<HandleIndex>` — hit-test for selection handles
- **New event controllers (added inside `Canvas::new()`):**
  - `GestureDrag (button=1)` — unified draw/move/resize gesture; behavior determined by `active_tool` and `drawing_state` at `drag_begin` time
  - `GestureClick (button=1, n_press=2)` — double-click to re-edit text/callout annotations
  - `EventControllerKey` — Delete (remove selected), Escape (deselect)
- **Gesture dispatch logic in `drag_begin`:**
  - If `active_tool == Select`: hit-test cursor; if on annotation body → `DrawingState::Moving`; if on handle → `DrawingState::ResizingHandle`; if empty → deselect
  - If any drawing tool: → `DrawingState::Drawing { start, current }`
  - Text/Timestamp/NumberMarker/Callout tools on click (drag_end with tiny movement) → open dialog or place immediately

### 7. Tool Palette

- **Purpose:** GTK4 widget for tool selection and style controls.
- **Location:** `src/ui/tool_palette.rs` (new file)
- **Structure:** Vertical `gtk::Box` packed into the window's left sidebar:
  - Tool toggle button group: one `gtk::ToggleButton` per tool; only one active at a time
  - `gtk::Separator`
  - Color button: `gtk::ColorButton` (or `gtk::ColorDialog` if available in the gtk4-rs version in use)
  - Stroke width: `gtk::SpinButton` (range 1.0–20.0, step 1.0, default 2.0)
- **Callbacks:**
  - `on_tool_changed(cb: impl Fn(ActiveTool) + 'static)`
  - `on_color_changed(cb: impl Fn(Color) + 'static)`
  - `on_stroke_changed(cb: impl Fn(f32) + 'static)`

### 8. Window Integration

- **Purpose:** Wire tool palette, undo/redo GActions, and layout into the main window.
- **Location:** `src/ui/window/imp.rs`, `src/ui/window/mod.rs`
- **Changes:**
  - Add `tool_palette: OnceCell<ToolPalette>` field to `MainWindow` imp
  - Pack `ToolPalette` into the window layout (left side of an `adw::OverlaySplitView` or a simple `gtk::Box` with horizontal layout)
  - Connect `tool_palette.on_tool_changed` → `canvas.set_active_tool()`
  - Connect `tool_palette.on_color_changed` + `on_stroke_changed` → `canvas.set_current_style()`
  - Register `win.undo` and `win.redo` in the existing `SimpleActionGroup`
  - Connect actions to `canvas.undo()` / `canvas.redo()`
  - Connect `canvas.on_annotation_changed()` → update undo/redo action enabled state

---

## Data Models

### Annotation Types

```rust
// src/annotations/model.rs

pub struct Annotation {
    pub id: Uuid,
    pub kind: AnnotationKind,
    pub bounds: Rect,          // bounding box in image-space pixels
    pub style: AnnotationStyle,
}

pub enum AnnotationKind {
    Rectangle,
    Ellipse,
    Arrow(ArrowData),
    Line(ArrowData),           // reuses ArrowData (start/end points)
    Freehand(FreehandData),
    Text(TextData),
    Blur,
    Pixelate,
    Redaction,
    Timestamp(TextData),       // pre-formatted; TextData::text is immutable post-creation
    NumberMarker(NumberMarkerData),
    Callout(CalloutData),
}

pub struct AnnotationStyle {
    pub stroke_color: Color,
    pub stroke_width: f32,
    pub fill_color: Option<Color>,
}

pub struct Rect  { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
pub struct Point { pub x: f64, pub y: f64 }
pub struct Color { pub r: f64, pub g: f64, pub b: f64, pub a: f64 }

pub struct ArrowData       { pub start: Point, pub end: Point }
pub struct FreehandData    { pub points: Vec<Point> }
pub struct TextData        { pub text: String, pub font_size: f32 }
pub struct NumberMarkerData{ pub number: u32 }
pub struct CalloutData     { pub text: String, pub anchor: Point }
```

**Coordinate convention:** All `Point` and `Rect` values are image-space pixels (origin = top-left of source image). Screen-space transform: `screen_x = pan_x + image_x * zoom`.

**`bounds` for each type:**
- Rectangle / Ellipse / Blur / Pixelate / Redaction: `bounds` is the exact shape area
- Arrow / Line: `bounds` is the bounding box; actual endpoints in `ArrowData`
- Freehand: `bounds` is the bounding box; actual path in `FreehandData::points` (absolute image-space)
- Text / Timestamp / Callout: `bounds.origin` is the placement anchor; `bounds.size` = rendered text size (computed at render time, updated after first render)
- NumberMarker: `bounds` is the circle bounding box (fixed size, e.g. 24×24px image-space)

### Undo/Redo Command

```rust
// src/annotations/history.rs

pub enum AnnotationCommand {
    Add(Annotation),
    Remove(Annotation),
    UpdateBounds { id: Uuid, old_bounds: Rect, new_bounds: Rect },
    UpdateStyle  { id: Uuid, old_style: AnnotationStyle, new_style: AnnotationStyle },
    UpdateText   { id: Uuid, old_text: String, new_text: String },
}
```

**Undo/redo inverse mapping:**
- `Add(ann)` ↔ engine.remove(id)
- `Remove(ann)` ↔ engine.add(ann.clone())
- `UpdateBounds { old, new }` ↔ engine.update_bounds(id, old)
- `UpdateStyle { old, new }` ↔ engine.update_style(id, old)
- `UpdateText { old, new }` ↔ engine.update_text(id, old)

### Tool / Gesture State

```rust
// src/annotations/tool.rs

pub enum ActiveTool {
    Select, Rectangle, Ellipse, Arrow, Line,
    Freehand, Text, Blur, Pixelate, Redaction,
    Timestamp, NumberMarker, Callout,
}

pub enum DrawingState {
    Idle,
    Drawing { start: Point, current: Point },
    Moving  { id: Uuid, drag_start: Point, original_bounds: Rect },
    ResizingHandle { id: Uuid, handle: HandleIndex, original_bounds: Rect, drag_start: Point },
    EditingText { existing_id: Option<Uuid>, position: Point },
}

pub enum HandleIndex { TopLeft, TopRight, BottomLeft, BottomRight }
```

---

## Error Handling Strategy

| Scenario | Handling | User Impact |
|---|---|---|
| Drag < 4px | Discard silently; no command pushed to history | No annotation; user tries again |
| Text confirmed empty | Discard silently | No annotation created |
| Hit-test on no annotation | `None` returned; engine deselects | Normal deselect |
| Undo/redo on empty stack | `can_undo`/`can_redo` = false; action disabled in UI | No-op; no crash |
| Blur/pixelate with no image loaded | Render a filled placeholder rect | Annotation appears but effect not applied |
| Resize below minimum 4px | Clamp bounds to 4×4px | Annotation reaches a floor size; no inversion |

---

## Tech Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Coordinate storage | Image-space for all annotation data | Invariant under zoom/pan; matches ADR-003 "canvas == export" guarantee |
| Rendering | Cairo in existing draw closure | No new library; consistent with image rendering already in place |
| Blur/Pixelate implementation | Pixbuf sub-region scale-down → scale-up | No external blur library; acceptable privacy-grade quality |
| Text editing UX | GTK4 modal dialog (`adw::MessageDialog` + `gtk::Entry`) | Avoids custom in-canvas text cursor; simpler to implement correctly |
| `AnnotationStyle` on Canvas | `RefCell<AnnotationStyle>` (not `Cell`) | `AnnotationStyle` contains a `Vec` (fill_color as Option isn't Copy); needs RefCell |
| Tool palette layout | Left sidebar (vertical tool strip) | Standard for drawing tools (Inkscape, GIMP, Pinta) |
| Freehand points | Absolute image-space `Vec<Point>` | Move = offset all points; no relative-coordinate complexity |
| Arrow/Line data | `ArrowData { start, end }` embedded in enum variant | Bounds = bounding box for hit-test; actual endpoints needed for direction-aware rendering |
| Number marker counter | Field in `AnnotationEngine` | Survives tool switches; resets only when full undo occurs |
| `uuid` crate | `Uuid::new_v4()` | Lightweight; needed for O(1) removal from the flat annotation list |
