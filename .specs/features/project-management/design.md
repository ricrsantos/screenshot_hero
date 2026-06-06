# Project Management Design

**Spec**: `.specs/features/project-management/spec.md`  
**Status**: Draft

---

## Architecture Overview

A new `src/persistence/` module handles all save/load logic. It is pure Rust with no GTK dependency, making it unit-testable without a display server. The `Canvas` GObject gains pub accessor/mutator methods to expose state for serialization and accept restored state after deserialization. Integration is wired at the Window level (`src/ui/window/imp.rs`).

```
┌──────────────────────────────────────────────────────────┐
│                       GTK4 UI Layer                       │
│                                                          │
│  MainWindow (src/ui/window/)                             │
│  ├── GAction: win.save-project                           │
│  ├── GAction: win.save-project-as                        │
│  ├── GAction: win.open-project                           │
│  └── auto-save hook in on_annotation_changed callback    │
│                      │                                   │
│                       ▼                                   │
│  Canvas (src/canvas/)                                    │
│  ├── pan_offset() → (f64, f64)         [NEW]             │
│  ├── all_annotations() → Vec<Annotation>  [NEW]          │
│  ├── source_image_path() → Option<PathBuf>  [NEW]        │
│  ├── source_image_dimensions() → Option<(u32,u32)> [NEW] │
│  ├── restore_annotations(Vec<Annotation>)  [NEW]         │
│  └── restore_zoom_pan(zoom, pan_x, pan_y)  [NEW]         │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
┌──────────────────────────────────────────────────────────┐
│              Application Layer                           │
│                                                          │
│  ProjectManager (src/persistence/manager.rs)             │
│  ├── current_path: Option<PathBuf>                       │
│  ├── auto_save_enabled: bool                             │
│  ├── created_at: Option<String>                          │
│  ├── save(project: SheroProject) → Result                │
│  ├── save_as(path, project) → Result                     │
│  ├── open(path) → Result<SheroProject>                   │
│  └── maybe_auto_save(project: SheroProject)              │
└──────────────────────────────────────────────────────────┘
                         │
          ┌──────────────┴──────────────┐
          ▼                             ▼
┌──────────────────────┐   ┌───────────────────────────┐
│     Serializer       │   │      Deserializer          │
│ src/persistence/     │   │   src/persistence/         │
│   serializer.rs      │   │     deserializer.rs        │
│                      │   │                            │
│ save_project(path,   │   │ load_project(path)         │
│   &SheroProject)     │   │   → Result<SheroProject>   │
│   → Result<()>       │   │                            │
└──────────────────────┘   └───────────────────────────┘
          │                             │
          └──────────────┬──────────────┘
                         ▼
┌──────────────────────────────────────────────────────────┐
│              Persistence Data Model                      │
│              src/persistence/model.rs                    │
│                                                          │
│  SheroProject { version, source_image, annotations,      │
│                 view_state, metadata }                   │
└──────────────────────────────────────────────────────────┘
                         │
                         ▼
                    .shero (JSON)
```

**Auto-save trigger flow**: Every annotation command (add/remove/move/resize/style/text) calls `canvas.notify_annotation_changed()`, which fires the `on_annotation_changed` callback stored in the Window. The Window's callback already updates undo/redo button state; it will also call `ProjectManager::maybe_auto_save()` with a freshly snapshotted `SheroProject`.

---

## Code Reuse Analysis

### Existing Components to Leverage

| Component | Location | How to Use |
|-----------|----------|------------|
| `Annotation`, `AnnotationKind`, all model structs | `src/annotations/model.rs` | Add `#[derive(Serialize, Deserialize)]` — used directly as `Vec<Annotation>` in `SheroProject` |
| `uuid::Uuid` | `uuid` crate (already has `serde` feature in Cargo.toml) | `Annotation.id` serializes transparently |
| `FileLoader::load_from_path` | `src/capture/loader.rs` | Reused in Open Project to load the source image from the saved path |
| `show_error_dialog` | `src/ui/dialogs.rs` | Reused for save/open error feedback |
| `gtk::FileDialog` async pattern | `src/ui/window/imp.rs` (open-file action) | Same pattern for `.shero` file picker |
| `glib::spawn_future_local` | Used for existing async portal/file dialogs | Same async GTK pattern for save/open dialogs |
| `chrono` | `Cargo.toml` (already present) | `chrono::Utc::now().to_rfc3339()` for timestamp strings |
| `serde_json` | `Cargo.toml` (already present) | `serde_json::to_string_pretty` / `from_str` for `.shero` I/O |

### Integration Points

| System | Integration Method |
|--------|--------------------|
| Canvas annotation state | New pub methods on `Canvas` GObject: `all_annotations()`, `restore_annotations()` |
| Canvas view state | New pub methods: `pan_offset()`, `restore_zoom_pan()` |
| Canvas source image | New pub methods: `source_image_path()`, `source_image_dimensions()` |
| Window title | `window.set_title(Some(filename))` after save/open |
| History reset on open | `canvas.imp().history.borrow_mut().clear()` — requires `History::clear()` method |
| Auto-save trigger | Extended `on_annotation_changed` closure in Window |

---

## Components

### `src/persistence/model.rs` (new)

- **Purpose**: Data structures for the `.shero` JSON format (ADR-004). Derives `Serialize`/`Deserialize`. No GTK dependency.
- **Location**: `src/persistence/model.rs`
- **Key types**:
  ```rust
  pub struct SheroProject {
      pub version: u32,           // Must be 1 for this implementation
      pub source_image: SourceImageRecord,
      pub annotations: Vec<Annotation>, // direct reuse of annotations::model
      pub view_state: ViewState,
      pub metadata: ProjectMetadata,
  }

  pub struct SourceImageRecord {
      pub path: String,           // absolute path to the source PNG/JPEG
      pub width: u32,
      pub height: u32,
  }

  pub struct ViewState {
      pub zoom: f64,
      pub pan_x: f64,
      pub pan_y: f64,
  }

  pub struct ProjectMetadata {
      pub created_at: String,     // RFC 3339 / ISO 8601 UTC
      pub modified_at: String,    // RFC 3339 / ISO 8601 UTC
      pub app_version: String,    // e.g. "0.1.0" from Cargo.toml
  }
  ```
- **Dependencies**: `serde`, `crate::annotations::model::Annotation`
- **Reuses**: `Annotation` and all its nested types directly

---

### `src/persistence/serializer.rs` (new)

- **Purpose**: Serialize a `SheroProject` to a `.shero` file (pretty-printed JSON).
- **Location**: `src/persistence/serializer.rs`
- **Interfaces**:
  - `pub fn save_project(path: &Path, project: &SheroProject) -> Result<(), PersistenceError>` — writes `serde_json::to_string_pretty(project)` to `path`, using atomic write (write to temp file + rename) to prevent corruption.
- **Dependencies**: `serde_json`, `std::fs`, `std::io`
- **Reuses**: `SheroProject` from model.rs

---

### `src/persistence/deserializer.rs` (new)

- **Purpose**: Deserialize a `.shero` file into a `SheroProject`.
- **Location**: `src/persistence/deserializer.rs`
- **Interfaces**:
  - `pub fn load_project(path: &Path) -> Result<SheroProject, PersistenceError>` — reads file, parses JSON, validates `version == 1`.
- **Error cases**: file not found, invalid JSON, unsupported version, I/O error.
- **Dependencies**: `serde_json`, `std::fs`
- **Reuses**: `SheroProject` from model.rs

---

### `src/persistence/manager.rs` (new)

- **Purpose**: Coordinates save/open operations. Holds current project path and auto-save state. Has no GTK dependency.
- **Location**: `src/persistence/manager.rs`
- **Key struct**:
  ```rust
  pub struct ProjectManager {
      pub current_path: Option<PathBuf>,
      pub auto_save_enabled: bool,
      pub created_at: Option<String>, // preserved across saves
  }
  ```
- **Interfaces**:
  - `pub fn new() -> Self`
  - `pub fn save(&mut self, path: &Path, project: SheroProject) -> Result<(), PersistenceError>` — updates `current_path`, sets/preserves `created_at`, updates `modified_at`, then calls serializer.
  - `pub fn open(&mut self, path: &Path) -> Result<SheroProject, PersistenceError>` — calls deserializer, updates `current_path` and `created_at` on success.
  - `pub fn maybe_auto_save(&self, project: SheroProject)` — saves silently if `auto_save_enabled && current_path.is_some()`; logs `warn!` on failure.
- **Dependencies**: `serializer`, `deserializer`, `model`, `chrono`, `log`

---

### `src/persistence/mod.rs` (new)

- **Purpose**: Module root; re-exports `ProjectManager`, `SheroProject`, `PersistenceError`.
- **Location**: `src/persistence/mod.rs`

---

### `src/persistence/error.rs` (new)

- **Purpose**: Unified error type for persistence operations.
- **Location**: `src/persistence/error.rs`
- **Key type**:
  ```rust
  pub enum PersistenceError {
      Io(std::io::Error),
      Json(serde_json::Error),
      UnsupportedVersion(u32),
      MissingSourceImage(String),
  }
  ```

---

### Canvas additions — `src/canvas/mod.rs` (modify)

New pub methods needed for state access and restoration:

| Method | Signature | Purpose |
|--------|-----------|---------|
| `pan_offset` | `pub fn pan_offset(&self) -> (f64, f64)` | Expose pan state for serialization |
| `all_annotations` | `pub fn all_annotations(&self) -> Vec<Annotation>` | Snapshot all annotations for serialization |
| `source_image_path` | `pub fn source_image_path(&self) -> Option<PathBuf>` | Get path of loaded source image |
| `source_image_dimensions` | `pub fn source_image_dimensions(&self) -> Option<(u32, u32)>` | Get source image width/height |
| `restore_annotations` | `pub fn restore_annotations(&self, annotations: Vec<Annotation>)` | Replace engine contents on open |
| `restore_zoom_pan` | `pub fn restore_zoom_pan(&self, zoom: f64, pan_x: f64, pan_y: f64)` | Set zoom + pan directly (no anchor math) |

---

### `src/annotations/history.rs` (modify)

Add `clear()` method:

```rust
pub fn clear(&mut self) {
    self.undo_stack.clear();
    self.redo_stack.clear();
}
```

This is called on project open to reset history without triggering callbacks.

---

### `src/ui/window/imp.rs` (modify)

- Add `project_manager: RefCell<ProjectManager>` field to `MainWindow` struct.
- Register `win.save-project`, `win.save-project-as`, `win.open-project` GActions.
- Add "Save" and "Open Project" buttons/items to the header bar.
- Extend the `on_annotation_changed` callback to call `maybe_auto_save` via `project_manager`.
- Handler logic for save (collect canvas state → build `SheroProject` → file dialog if needed → `manager.save()`).
- Handler logic for open (file dialog → `load_project()` → `FileLoader` for image → `restore_*` → `history.clear()` → update title).

---

## Data Models

### SheroProject (`.shero` JSON)

Follows ADR-004 exactly:

```json
{
  "version": 1,
  "source_image": {
    "path": "/home/user/Pictures/Screenshot_2026-06-06.png",
    "width": 1920,
    "height": 1080
  },
  "annotations": [
    {
      "id": "a1b2c3d4-...",
      "kind": { "Rectangle": null },
      "bounds": { "x": 100.0, "y": 200.0, "width": 300.0, "height": 150.0 },
      "style": {
        "stroke_color": { "r": 1.0, "g": 0.0, "b": 0.0, "a": 1.0 },
        "stroke_width": 2.0,
        "fill_color": null
      }
    }
  ],
  "view_state": {
    "zoom": 1.25,
    "pan_x": 40.0,
    "pan_y": -20.0
  },
  "metadata": {
    "created_at": "2026-06-06T14:00:00Z",
    "modified_at": "2026-06-06T14:30:00Z",
    "app_version": "0.1.0"
  }
}
```

**Annotation serde note**: `serde` serializes Rust enums with associated data as `{ "VariantName": { ...data } }` by default. The `AnnotationKind` variants without data (Rectangle, Ellipse, Blur, Pixelate, Redaction) serialize as `"Rectangle"` (unit variant string). This is handled transparently once `#[derive(Serialize, Deserialize)]` is added to the annotation model types.

---

## Error Handling Strategy

| Error Scenario | Handling | User Impact |
|----------------|----------|-------------|
| Save: I/O error (disk full, permission denied) | `PersistenceError::Io` → `show_error_dialog` | Blocking error dialog; in-memory state preserved |
| Open: file not found | `PersistenceError::Io` | Blocking error dialog; current state unchanged |
| Open: invalid JSON | `PersistenceError::Json` | Blocking error dialog; current state unchanged |
| Open: unsupported version | `PersistenceError::UnsupportedVersion(v)` | Error: "Saved with a newer version of Screenshot Hero (v{v})" |
| Open: source image missing | `PersistenceError::MissingSourceImage(path)` | Error: "Source image not found: {path}"; open aborted |
| Auto-save: any failure | Log `warn!`; no dialog | Silent; user continues editing uninterrupted |

---

## Tech Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Serde on existing Annotation model | Add derives directly to `src/annotations/model.rs` | No DTO indirection needed; model is pure Rust; avoids redundant struct definitions |
| Timestamps as `String` (RFC 3339) | `chrono::Utc::now().to_rfc3339()` | No need for `chrono` serde feature; strings are portable and human-readable in JSON |
| Auto-save trigger via `on_annotation_changed` | Extend existing callback in Window | Callback already fires on every execute/undo/redo; zero new mechanism needed |
| Atomic write (temp + rename) for save | `write to .shero.tmp` then `fs::rename` | Prevents corrupt `.shero` file if process is killed mid-write |
| `ProjectManager` is pure Rust, stored in `RefCell<ProjectManager>` in Window imp | No GTK in manager | Keeps persistence logic unit-testable without display server |
| `const APP_VERSION: &str = env!("CARGO_PKG_VERSION")` | Compile-time version injection | Guarantees `metadata.app_version` matches built binary; no runtime lookup |
