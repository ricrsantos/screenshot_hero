# Architecture

**Pattern:** Modular desktop monolith (single binary, domain-separated modules)

## High-Level Structure

The application is a single Rust GTK/libadwaita process that composes feature modules:

- `application` bootstraps GTK app lifecycle, startup config, and accelerators
- `ui/window` orchestrates user actions and cross-module workflows
- `canvas` hosts image state, viewport state, and annotation interaction/rendering
- `annotations` provides pure domain logic (model/engine/history/tool behavior)
- `capture`, `persistence`, `export`, and `settings` provide capability modules

High-level request flow:

1. User triggers an action (`win.*` GAction)
2. `ui/window/imp.rs` collects required state from `Canvas`
3. Domain/service module executes (`capture`, `persistence`, `export`, `settings`)
4. Result updates UI state (canvas, action enablement, dialogs, title)

## Identified Patterns

### GAction-Centric UI Command Pattern

**Location:** `src/ui/window/imp.rs`, `src/application.rs`  
**Purpose:** Centralize user interactions and keyboard shortcuts  
**Implementation:** Each capability is bound to `gio::SimpleAction` (`win.open-file`, `win.save-project`, `win.export-png`, etc.) and wired once during window construction.

### Pure Domain Core + GTK Adapter

**Location:** `src/annotations/`, `src/persistence/` vs `src/ui/`, `src/canvas/`  
**Purpose:** Keep business logic testable without display server  
**Implementation:** Annotation engine/history/model and persistence serializer/deserializer/manager avoid GTK widgets directly; UI layers call into them.

### Async Portal/Dialogs on Main Thread

**Location:** `src/application.rs`, `src/ui/window/imp.rs`, `src/capture/service.rs`  
**Purpose:** Keep GTK thread-safe while supporting asynchronous operations  
**Implementation:** `glib::spawn_future_local` wraps portal capture and file dialog flows.

### Runtime Behavior via Settings

**Location:** `src/settings/mod.rs`, `src/application.rs`, `src/ui/preferences/mod.rs`, `src/ui/window/imp.rs`  
**Purpose:** Persist user preferences and apply them live  
**Implementation:** GSettings schema + typed wrapper + UI bindings + runtime observers (`connect_changed`).

## Data Flow

### Capture/Open Flow

1. User triggers `win.new-screenshot` or `win.open-file`
2. `capture::CaptureService` (portal) or `capture::FileLoader` returns `ImageData`
3. Window sets image into `Canvas` and calls fit-to-window
4. Image-dependent actions become enabled

### Annotation + Automation Flow

1. Canvas annotations mutate (draw/edit/undo/redo)
2. `on_annotation_changed` callback in window runs
3. Callback updates undo/redo action state
4. Depending on settings, callback triggers:
   - auto-save via `ProjectManager::maybe_auto_save`
   - auto-clipboard debounce and copy
   - auto-export render + PNG write

### Save/Open Project Flow

1. Save actions build snapshot (`SheroProject`) from canvas state
2. `persistence::ProjectManager` enriches metadata and delegates to serializer
3. Open action loads `SheroProject`, validates source image, restores annotations and viewport
4. UI title and action states are refreshed

## Code Organization

**Approach:** Layered by capability with reusable domain modules

**Module boundaries:**

- `src/application.rs`: app lifecycle and global startup policy
- `src/ui/`: GTK/libadwaita windows/dialog wiring
- `src/canvas/`: GTK drawing area and viewport interaction
- `src/annotations/`: annotation model + behavior core
- `src/capture/`: screenshot/file ingestion
- `src/persistence/`: `.shero` schema and I/O lifecycle
- `src/export/`: render-to-image, disk export, clipboard export
- `src/settings/`: typed API over GSettings
