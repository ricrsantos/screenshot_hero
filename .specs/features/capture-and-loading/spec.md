# Screenshot Capture and Loading — Specification

**PRD:** `docs/product/PRD-001-screenshot-capture-and-loading.md`  
**Status:** Approved

---

## Problem Statement

Users need to get screenshots into Screenshot Hero before they can annotate them. There are two entry points: capturing a new screenshot directly (via the GNOME Screenshot Portal) or opening an existing image file (PNG/JPEG). Without these two flows, the app has no content to work with. This is the foundational module — nothing else can be built until images can be loaded.

---

## Goals

- [ ] User can capture a screenshot interactively and see it appear immediately in the canvas
- [ ] User can open an existing PNG or JPEG file and see it appear in the canvas
- [ ] All failure scenarios are handled gracefully with clear user feedback

---

## Out of Scope

| Feature | Reason |
|---------|--------|
| Zoom and pan | PRD-002 (Canvas and Navigation) |
| Annotation tools | PRD-003 |
| Save/export | PRD-004, PRD-005 |
| Drag-and-drop image loading | Not in PRD-001; deferred |
| Recent files | Requires project management (PRD-004) |
| TIFF, BMP, WebP support | Only PNG/JPEG in v1 (FR-008, FR-009) |
| Full-screen / window-only capture modes | Portal handles selection; no custom modes |

---

## User Stories

### P1: New Screenshot ⭐ MVP

**User Story:** As a GNOME user, I want to capture a screenshot interactively so that I can immediately annotate it in Screenshot Hero.

**Why P1:** This is the primary workflow of the app — without it, Screenshot Hero has no reason to exist.

**Acceptance Criteria:**

1. WHEN user activates the "New Screenshot" action THEN system SHALL hide the application window and invoke the XDG Screenshot Portal in interactive mode
2. WHEN the portal returns a file URI THEN system SHALL load the image from that URI and display it in the canvas
3. WHEN user cancels the portal dialog THEN system SHALL restore the application window without loading any image
4. WHEN the portal is unavailable (not a GNOME session, permission denied) THEN system SHALL display an error dialog explaining the issue
5. WHEN the image load from the URI fails THEN system SHALL display an error dialog

**Independent Test:** Activate "New Screenshot", select a region, confirm screenshot appears in the canvas.

---

### P1: Open Existing Image ⭐ MVP

**User Story:** As a user, I want to open an existing PNG or JPEG file so that I can annotate a previously captured or received image.

**Why P1:** Required for users who already have screenshots from other tools, or who receive images to annotate.

**Acceptance Criteria:**

1. WHEN user activates the "Open File" action THEN system SHALL display a file picker filtered to PNG and JPEG files
2. WHEN user selects a PNG file THEN system SHALL load it and display it in the canvas
3. WHEN user selects a JPEG file THEN system SHALL load it and display it in the canvas
4. WHEN user cancels the file picker THEN system SHALL close the picker without changing canvas state
5. WHEN the selected file cannot be decoded (corrupt, unsupported format) THEN system SHALL display an error dialog with the filename

**Independent Test:** Open File picker, select a PNG, confirm image appears in the canvas.

---

### P1: Canvas Image Display ⭐ MVP

**User Story:** As a user, I want to see the loaded image displayed in the canvas so that I can verify it loaded correctly before annotating.

**Why P1:** Without displaying the image, there is nothing to annotate. This is the minimal canvas required for Milestone 1.

**Acceptance Criteria:**

1. WHEN an image is loaded THEN system SHALL display it in the canvas area at its natural size (no zoom/pan in this milestone)
2. WHEN no image is loaded THEN system SHALL display an empty/placeholder state in the canvas

**Independent Test:** After loading an image, canvas renders it visibly with correct proportions.

---

### P2: Error Feedback

**User Story:** As a user, I want to see a clear error message when something goes wrong so that I understand what happened and can take action.

**Why P2:** Essential for usability, but error states are secondary to the happy path.

**Acceptance Criteria:**

1. WHEN any error occurs during capture or loading THEN system SHALL display an `adw::AlertDialog` with a human-readable message
2. WHEN an error dialog is shown THEN system SHALL log the technical error at Error level
3. WHEN user dismisses the error dialog THEN system SHALL return to the previous stable state

---

## Edge Cases

- WHEN the portal returns an empty URI THEN system SHALL treat it as a cancellation (no error dialog)
- WHEN a file has a valid extension but invalid content THEN system SHALL show an error dialog (not crash)
- WHEN the application is not running in a GNOME/portal-capable session THEN system SHALL show a descriptive error when the user attempts to capture
- WHEN the canvas already has an image and user loads another THEN system SHALL replace the current image

---

## Requirement Traceability

| Requirement ID | PRD Ref | Story | Status |
|---|---|---|---|
| CAPT-01 | FR-001 | P1: New Screenshot | Pending |
| CAPT-02 | FR-002 | P1: New Screenshot | Pending |
| CAPT-03 | FR-003 | P1: New Screenshot | Pending |
| CAPT-04 | FR-004 | P1: New Screenshot | Pending |
| CAPT-05 | FR-005 | P1: New Screenshot | Pending |
| CAPT-06 | FR-006 | P1: Canvas Image Display | Pending |
| CAPT-07 | FR-007 | P1: Canvas Image Display | Pending |
| CAPT-08 | FR-008 | P1: Open Existing Image | Pending |
| CAPT-09 | FR-009 | P1: Open Existing Image | Pending |
| CAPT-10 | FR-010 | P2: Error Feedback | Pending |

**Coverage:** 10 total, 0 mapped to tasks, 10 unmapped ⚠️

---

## Success Criteria

- [ ] User can capture a screenshot in under 5 seconds from activating the action
- [ ] User can open a PNG/JPEG file and see it in the canvas
- [ ] Zero unhandled panics in the happy path or any documented error scenario
- [ ] App runs correctly inside Flatpak (`flatpak run`)
