# Project Structure

**Root:** `/home/ricardo/data/projects/screenshot_hero/screenshot_hero`

## Directory Tree

```text
.
├── src
│   ├── annotations
│   ├── canvas
│   ├── capture
│   ├── export
│   ├── models
│   ├── persistence
│   ├── settings
│   └── ui
├── data
│   ├── resources
│   ├── icons
│   └── com.screenshot_hero.ScreenshotHero.gschema.xml
├── build
│   └── com.screenshot_hero.ScreenshotHero.yml
├── .specs
│   ├── project
│   ├── codebase
│   └── features
└── Cargo.toml
```

## Module Organization

### Application Lifecycle

**Purpose:** App startup, activation, command-line behavior, accelerators  
**Location:** `src/main.rs`, `src/application.rs`, `src/resources.rs`  
**Key files:** `application.rs`, `resources.rs`

### UI and Interaction

**Purpose:** Window composition, menus/actions, dialogs, preferences, tool palette  
**Location:** `src/ui/`, `src/canvas/`  
**Key files:** `src/ui/window/imp.rs`, `src/ui/preferences/mod.rs`, `src/canvas/mod.rs`

### Annotation Domain

**Purpose:** Annotation model, editing engine, undo/redo history  
**Location:** `src/annotations/`  
**Key files:** `model.rs`, `engine.rs`, `history.rs`

### File/Capture/Export/Persistence Services

**Purpose:** Input/output workflows around images and projects  
**Location:** `src/capture/`, `src/export/`, `src/persistence/`  
**Key files:** `capture/service.rs`, `export/renderer.rs`, `persistence/manager.rs`

### Settings

**Purpose:** Typed access to persisted preferences  
**Location:** `src/settings/mod.rs`  
**Key files:** `mod.rs`

## Where Things Live

**Capture and image loading:**
- UI trigger: `src/ui/window/imp.rs`
- Business logic: `src/capture/service.rs`, `src/capture/loader.rs`
- Data model: `src/models/image.rs`
- Config/runtime dependencies: portal permissions in `build/com.screenshot_hero.ScreenshotHero.yml`

**Annotation workflow:**
- UI interaction and callbacks: `src/canvas/`, `src/ui/tool_palette.rs`
- Business logic: `src/annotations/`
- Export rendering coupling: `src/export/renderer.rs` reuses canvas renderer behavior

**Project save/load:**
- UI actions/dialogs: `src/ui/window/imp.rs`
- Business logic and data format: `src/persistence/`
- Data contract: `SheroProject` in `src/persistence/model.rs`

**Preferences/runtime behavior:**
- Settings API: `src/settings/mod.rs`
- Preferences UI: `src/ui/preferences/mod.rs`
- Runtime consumption: `src/application.rs`, `src/ui/window/imp.rs`

## Special Directories

**`data/`:**
- Purpose: app metadata/resources (desktop file, icons, gschema, gresource manifest)
- Examples: `data/com.screenshot_hero.ScreenshotHero.gschema.xml`

**`build/`:**
- Purpose: Flatpak manifest and build integration
- Examples: `build/com.screenshot_hero.ScreenshotHero.yml`

**`.specs/`:**
- Purpose: spec-driven planning, roadmap, state memory, and codebase documentation
- Examples: `.specs/project/STATE.md`, `.specs/features/*`
