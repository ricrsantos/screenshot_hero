# Export and Clipboard Specification

**PRD**: `docs/product/PRD-005-export-and-clipboard.md`
**Status**: Draft

---

## Problem Statement

Screenshot Hero's core workflow is **Capture → Annotate → Paste**. Without clipboard and export support, users annotate images but have no way to share or use them. This feature closes the workflow loop by enabling direct clipboard copy (auto-updated on every change) and file export in PNG/JPEG formats.

## Goals

- [ ] Users can export annotated images as PNG or JPEG to any location they choose
- [ ] The clipboard always contains the latest annotated result — ready to paste without any action
- [ ] Auto-export optionally writes the annotated image beside the original on every change

## Out of Scope

| Feature | Reason |
|---------|--------|
| Settings persistence (GSettings) for auto-clipboard / auto-export toggles | PRD-006 responsibility — toggles default correctly in PRD-005 |
| Copy original image (without annotations) | Not a stated use case; always export the rendered result |
| TIFF / WebP / other formats | PNG and JPEG cover stated requirements |
| Cloud upload | Out of scope by design (Privacy First, Offline First — ADR-001) |
| Share sheet / OS share integration | Flatpak portal for sharing is future scope |

---

## User Stories

### P1: Export PNG ⭐ MVP

**User Story**: As a user, I want to export the annotated screenshot as PNG so I can save it to disk and share it.

**Why P1**: PNG is the primary screenshot format. The export workflow completes the "Capture → Annotate → Save" path.

**Acceptance Criteria**:

1. WHEN user triggers "Export PNG" THEN system SHALL open a save file dialog pre-filtered to `.png` files
2. WHEN user confirms a save path THEN system SHALL render the source image + all annotations at 1:1 resolution and write a valid PNG file to disk
3. WHEN the export succeeds THEN system SHALL log an info message; no success dialog is required
4. WHEN the export fails (disk full, permission denied) THEN system SHALL show an error dialog with the reason
5. WHEN no image is loaded THEN system SHALL disable the Export PNG action

**Independent Test**: Load any screenshot, add annotations, trigger Export PNG, confirm file exists on disk and opens correctly in an image viewer.

---

### P1: Manual Clipboard Copy ⭐ MVP

**User Story**: As a user, I want to copy the annotated image to the clipboard so I can paste it into any application.

**Why P1**: "Paste" is the primary delivery mechanism. Manual copy is needed when auto-clipboard is disabled or a one-off copy is required.

**Acceptance Criteria**:

1. WHEN user triggers "Copy to Clipboard" THEN system SHALL render source + annotations at 1:1 and write the result to the system clipboard as an image
2. WHEN no image is loaded THEN system SHALL disable the Copy to Clipboard action
3. WHEN the clipboard write succeeds THEN system SHALL not show any dialog (silent success)

**Independent Test**: Load screenshot, add annotation, trigger Copy, paste into GIMP / text editor — annotated image appears.

---

### P1: Auto-Clipboard Update ⭐ MVP

**User Story**: As a user, I want the clipboard to automatically refresh after every annotation change so I can always paste the latest version without any extra action.

**Why P1**: This is the core Capture → Annotate → **Paste** workflow. ADR-001 declares clipboard as a first-class feature enabled by default.

**Acceptance Criteria**:

1. WHEN a new image is loaded and auto-clipboard is enabled THEN system SHALL NOT automatically push to clipboard (clipboard update is annotation-change-triggered)
2. WHEN an annotation is created, moved, resized, or deleted and auto-clipboard is enabled THEN system SHALL schedule a debounced clipboard update (300 ms delay)
3. WHEN undo is performed and auto-clipboard is enabled THEN system SHALL schedule a debounced clipboard update
4. WHEN redo is performed and auto-clipboard is enabled THEN system SHALL schedule a debounced clipboard update
5. WHEN multiple annotation changes happen within 300 ms THEN system SHALL coalesce them into a single clipboard write (debounce resets on each change)
6. WHEN auto-clipboard is enabled by default THEN no user configuration is required to activate it

**Independent Test**: Load screenshot, add rectangle annotation, wait 400ms, paste into another app — annotated image (with rectangle) appears.

---

### P2: Export JPEG

**User Story**: As a user, I want to export the annotated screenshot as JPEG so I can produce smaller files for web sharing.

**Why P2**: JPEG is useful but lossy; PNG is the preferred format. JPEG is a secondary option.

**Acceptance Criteria**:

1. WHEN user triggers "Export JPEG" THEN system SHALL open a save file dialog pre-filtered to `.jpg` files
2. WHEN user confirms a save path THEN system SHALL render source + annotations at 1:1 and write a valid JPEG file
3. WHEN export fails THEN system SHALL show an error dialog

**Independent Test**: Load screenshot, add annotations, export JPEG, verify file is valid JPEG.

---

### P2: Auto-Export

**User Story**: As a user, I want the app to automatically save an annotated image file alongside the original screenshot so I always have an up-to-date export without manual steps.

**Why P2**: Convenient but optional. ADR-001 defines this as disabled by default to avoid cluttering the user's screenshot folder.

**Acceptance Criteria**:

1. WHEN auto-export is disabled (default) THEN system SHALL NOT write any file on annotation changes
2. WHEN auto-export is enabled AND the project has a source image path THEN system SHALL write `{original_stem}{suffix}.png` in the same directory as the source image after every annotation change
3. WHEN auto-export is enabled AND the suffix is `_shero` (default) THEN a source named `screenshot.png` SHALL produce `screenshot_shero.png`
4. WHEN an annotation change triggers auto-export THEN the export SHALL use the same off-screen rendering pipeline as manual export (ADR-003: canvas view == export result)
5. WHEN auto-export write fails THEN system SHALL log a warning but NOT show an error dialog (background operation)

**Independent Test**: Enable auto-export in code, add annotation, verify `screenshot_shero.png` appears in the same folder as the source.

---

## Edge Cases

- WHEN source image dimensions are 0 or invalid THEN system SHALL skip export/clipboard silently and log a warning
- WHEN auto-export target path already exists THEN system SHALL overwrite it (no prompt)
- WHEN project has no source image path (image was just captured but not saved as project yet) THEN auto-export SHALL be skipped (no path to save beside)
- WHEN clipboard is unavailable (no display, Wayland compositor issue) THEN system SHALL log the error and continue without crashing
- WHEN a very rapid burst of annotation changes occurs THEN auto-clipboard SHALL write exactly once after the final change settles (debounce guarantee)

---

## Requirement Traceability

| Requirement ID | PRD Ref | Story | Phase | Status |
|---|---|---|---|---|
| EXPRT-01 | FR-001 | P1: Export PNG | Design | Pending |
| EXPRT-02 | FR-002 | P2: Export JPEG | Design | Pending |
| EXPRT-03 | FR-003 | P1: Export PNG / P2: JPEG | Design | Pending |
| EXPRT-04 | FR-004 | P1: Manual Clipboard Copy | Design | Pending |
| EXPRT-05 | FR-005 | P1: Manual Clipboard Copy | Design | Pending |
| EXPRT-06 | FR-006 | P1: Auto-Clipboard Update | Design | Pending |
| EXPRT-07 | FR-007 | P1: Auto-Clipboard Update | Design | Pending |
| EXPRT-08 | FR-008 | P1: Auto-Clipboard Update | Design | Pending |
| EXPRT-09 | FR-009 | P1: Auto-Clipboard Update | Design | Pending |
| EXPRT-10 | FR-010 | P1: Auto-Clipboard Update | Design | Pending |
| EXPRT-11 | FR-011 | P2: Auto-Export | Design | Pending |
| EXPRT-12 | FR-012 | P2: Auto-Export | Design | Pending |
| EXPRT-13 | FR-013 | P2: Auto-Export | Design | Pending |
| EXPRT-14 | FR-014 | P2: Auto-Export | Design | Pending |

**Coverage**: 14 total, 0 mapped to tasks, 14 unmapped ⚠️

---

## Success Criteria

- [ ] User can export a PNG file and open it in an image viewer — annotations are visible
- [ ] User can paste the annotated image from clipboard into any app immediately after annotating
- [ ] No user action required: clipboard auto-refreshes within 300ms of the last annotation change
- [ ] Original source image is never modified (non-destructive throughout)
- [ ] Auto-export (when enabled) writes files silently without blocking the UI
