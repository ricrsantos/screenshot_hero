# Screenshot Hero - Roadmap

**Last updated:** 2026-06-15  
**Current program status:** v1 scope implemented; focus moved to stabilization, validation, and documentation hygiene.

## Milestone 1: Screenshot Capture and Loading
**Feature:** PRD-001  
**Spec:** `.specs/features/capture-and-loading/`  
**Status:** Implemented

**Delivered:**
- Cargo project with GTK4 + Libadwaita + ashpd
- Application shell (main window, header bar, GActions)
- Screenshot capture via GNOME Screenshot Portal
- Open PNG/JPEG from file picker
- Initial canvas load/fit behavior and error handling

---

## Milestone 2: Canvas and Navigation
**Feature:** PRD-002  
**Spec:** `.specs/features/canvas-and-navigation/`  
**Status:** Implemented

**Delivered:**
- Zoom controls (buttons, shortcuts, and mouse wheel)
- Middle-mouse pan
- Fit-to-window and 100% zoom actions
- Zoom percentage indicator

---

## Milestone 3: Annotations
**Feature:** PRD-003  
**Spec:** `.specs/features/annotations/`  
**Status:** Implemented

**Delivered:**
- `src/annotations/` domain module (model, engine, history, tool state)
- Canvas renderers and interactions for annotation drawing/editing
- Tool palette integration (selection, color, stroke width)
- Undo/redo and keyboard shortcuts
- Text and advanced annotation variants integrated into rendering flow

---

## Milestone 4: Project Management
**Feature:** PRD-004  
**Spec:** `.specs/features/project-management/`  
**Status:** Implemented

**Delivered:**
- Save/open `.shero` project files
- Auto-save on annotation change when enabled
- Project metadata lifecycle (`created_at`, `modified_at`, `app_version`)
- Persistence module with serializer/deserializer/manager separation

---

## Milestone 5: Export and Clipboard
**Feature:** PRD-005  
**Spec:** `.specs/features/export-and-clipboard/`  
**Status:** Implemented

**Delivered:**
- `src/export/` module (`renderer`, `exporter`, `clipboard`, `auto_export`)
- Manual export actions: `win.export-png`, `win.export-jpeg`
- Manual clipboard action: `win.copy-to-clipboard`
- Auto-clipboard debounce (300ms)
- Auto-export path strategy with configurable suffix

---

## Milestone 6: Settings and Preferences
**Feature:** PRD-006  
**Spec:** `.specs/features/settings-and-preferences/`  
**Status:** Implemented

**Delivered:**
- GSettings schema and runtime binding (`com.screenshot_hero.ScreenshotHero`)
- Preferences window with Appearance, Timestamps, Automation, and Developer groups
- Runtime application of theme and log-level preferences
- Settings-backed automation toggles consumed by window behavior

---

## Milestone 7: Behavior Settings Extend
**Feature:** behavior-settings-extend  
**Spec:** `.specs/features/behavior-settings-extend/`  
**Status:** Implemented

**Delivered:**
- Capture behavior preferences for post-capture editing, temporary disable window, and window reuse policy
- `--capture` runtime policy to capture-and-exit without opening editor when configured
- Clipboard-exit automation (`Exit after paste`) for startup capture sessions
- Default capture reuse behavior with optional per-capture new-window mode

---

## Next Track

- Stabilize Flatpak and native runtime behavior across environments
- Expand manual UAT checklist for full end-to-end workflow
- Keep `.specs` and implementation state aligned during future prompt-driven edits
