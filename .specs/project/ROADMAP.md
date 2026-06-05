# Screenshot Hero - Roadmap

## Milestone 1: Screenshot Capture and Loading ← current
**Feature:** PRD-001  
**Spec:** `.specs/features/capture-and-loading/`  
**Status:** Planning

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
**Status:** Not started

**Deliverables:**
- Zoom (10% – 800%), scroll-wheel zoom
- Pan (middle mouse button)
- Fit-to-window / 100% zoom shortcuts
- Zoom level indicator in header

---

## Milestone 3: Annotations
**Feature:** PRD-003  
**Status:** Not started

**Deliverables:**
- Annotation engine (Command Pattern for undo/redo)
- Tools: rectangle, arrow, ellipse, line, freehand, text, blur, pixelate, redaction, timestamp, numbered marker, callout
- Selection, move, resize, delete
- Undo/Redo

---

## Milestone 4: Project Management
**Feature:** PRD-004  
**Status:** Not started

**Deliverables:**
- Save/load `.shero` project files
- Auto-save on change
- Project metadata (created_at, modified_at, app_version)

---

## Milestone 5: Export and Clipboard
**Feature:** PRD-005  
**Status:** Not started

**Deliverables:**
- Export PNG/JPEG
- Auto clipboard update (debounced 300ms)
- Auto export option (same-directory suffix)

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
