# Screenshot Hero - Roadmap

## Milestone 1: Screenshot Capture and Loading
**Feature:** PRD-001  
**Spec:** `.specs/features/capture-and-loading/`  
**Status:** Implemented

**Deliverables:**
- Cargo project with GTK4 + Libadwaita + ashpd
- Application shell (main window, header bar, GActions)
- Screenshot capture via GNOME Screenshot Portal
- Load existing PNG/JPEG via file picker
- Basic canvas (image display, no zoom/pan yet)
- Error handling (invalid file, portal unavailable)
- Flatpak manifest with portal permissions

---

## Milestone 2: Canvas and Navigation
**Feature:** PRD-002  
**Spec:** `.specs/features/canvas-and-navigation/`  
**Status:** Implemented

**Deliverables:**
- Zoom (10% – 800%), scroll-wheel zoom
- Pan (middle mouse button)
- Fit-to-window / 100% zoom shortcuts
- Zoom level indicator in header

---

## Milestone 3: Annotations
**Feature:** PRD-003  
**Spec:** `.specs/features/annotations/`  
**Status:** Planning complete → Ready to Execute

**Deliverables:**
- `src/annotations/` module: data model, engine (CRUD + selection + hit-test), history (Command Pattern), tool state
- `src/canvas/renderer.rs`: Cairo renderers for all 12 annotation types + selection handles
- Canvas interaction: draw/select/move/resize/delete gestures + keyboard shortcuts
- Text annotation editor (modal dialog)
- Tool palette widget (left sidebar with tool buttons + color/stroke controls)
- Undo/Redo (Ctrl+Z / Ctrl+Y) via GActions

**Task summary:** 18 tasks (T1–T18); see `.specs/features/annotations/tasks.md`

**P1 (MVP):** Rectangle, Ellipse, Arrow, Line + Select/Move/Resize/Delete + Undo/Redo + Style (T1–T17)  
**P2:** Freehand, Text, Blur, Pixelate, Redaction, Timestamp, Number Marker (included in T1–T18)  
**P3:** Callout (included in T7, T9, T12)

---

## Milestone 4: Project Management
**Feature:** PRD-004  
**Status:** Not started

**Deliverables:**
- Save/load `.shero` project files
- Auto-save on change
- Project metadata (created_at, modified_at, app_version)

---

## Milestone 5: Export and Clipboard ← current
**Feature:** PRD-005  
**Spec:** `.specs/features/export-and-clipboard/`  
**Status:** Planning complete → Ready to Execute (9 tasks: T1–T4 parallel, T5–T9 sequential)

**Deliverables:**
- `src/export/` module: off-screen renderer, file exporter (PNG/JPEG), clipboard writer, auto-export path logic
- Manual export GActions: `win.export-png`, `win.export-jpeg`, `win.copy-to-clipboard`
- Auto-clipboard update (debounced 300ms, enabled by default)
- Auto-export (disabled by default, configurable suffix `_shero`, saves beside original)

---

## Milestone 6: Settings and Preferences
**Feature:** PRD-006  
**Status:** Not started

**Deliverables:**
- Appearance (Follow System / Light / Dark)
- Timestamp toggle + format
- Auto-save, auto-export, auto-clipboard toggles
- Log level selector
- GSettings-backed persistence
