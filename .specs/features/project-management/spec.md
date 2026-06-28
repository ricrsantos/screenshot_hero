# Project Management Specification

**Feature:** PRD-004 — Project Management  
**Status:** Draft

---

## Problem Statement

Users currently lose all annotation work when they close Screenshot Hero. There is no way to save an in-progress annotation session and resume editing later. This blocks use cases that span multiple sessions or require iterative refinement of annotations.

## Goals

- [ ] Users can save annotated screenshots as editable `.shero` project files and reopen them to continue editing
- [ ] Auto-save protects work automatically after every modification (execute, undo, redo)
- [ ] Full project state is restored on open: annotations (all 12 types), canvas view (zoom/pan), and project metadata

## Out of Scope

| Feature | Reason |
|---------|--------|
| Recent files list | Requires GSettings + file history management; deferred to PRD-006 |
| Project version history | Beyond v1 scope |
| Multiple open projects / tabs | Single-project workflow for v1 |
| Cloud sync or backup | Offline-first architecture (ADR-001) |
| Image embedding in `.shero` | Source image path stored, not embedded (ADR-004) |

---

## User Stories

### P1: Save Project ⭐ MVP

**User Story**: As a user, I want to save my annotated screenshot as a `.shero` file so I can resume editing later.

**Why P1**: Without save, all annotation work is lost on close. This is the core value proposition of project management.

**Acceptance Criteria**:

1. WHEN user triggers Save (Ctrl+S) AND no save path is established, THEN system SHALL show a file chooser dialog scoped to `.shero` files for the user to choose location and name.
2. WHEN user triggers Save AND a save path already exists, THEN system SHALL overwrite the `.shero` file without prompting.
3. WHEN user triggers Save As (header menu), THEN system SHALL always show the file chooser dialog, regardless of existing save path.
4. WHEN save succeeds, THEN system SHALL update the window title to show the project file name.
5. WHEN save fails (I/O error, permission denied), THEN system SHALL show an error dialog with the failure reason. In-memory project state SHALL remain unmodified.
6. WHEN project is saved, THEN the `.shero` file SHALL be valid JSON containing: `version`, `source_image` (path + dimensions), `annotations` (all types), `view_state` (zoom, pan_x, pan_y), and `metadata` (created_at, modified_at, app_version) — per ADR-004.

**Independent Test**: Open app, capture a screenshot, add 2-3 annotations, press Ctrl+S, inspect the `.shero` file in a text editor to verify valid JSON with all expected sections.

---

### P1: Open Project ⭐ MVP

**User Story**: As a user, I want to open a `.shero` project file so I can continue annotation editing from where I left off.

**Why P1**: Save is only useful if open works correctly and restores full state.

**Acceptance Criteria**:

1. WHEN user triggers Open Project (header menu), THEN system SHALL show a file picker filtered to `.shero` files.
2. WHEN user selects a valid `.shero` file, THEN system SHALL restore: all annotations (correct types, positions, styles), canvas zoom level, canvas pan offset, and update window title to the project file name.
3. WHEN open succeeds, THEN system SHALL reset the undo/redo history (no undoing into the pre-open state).
4. WHEN the `.shero` file cannot be parsed (invalid JSON, schema mismatch, unsupported version), THEN system SHALL show an error dialog. Current project state in memory SHALL NOT be cleared.
5. WHEN the `.shero` references a source image path that no longer exists, THEN system SHALL show an error dialog explaining the missing file. The open operation SHALL be aborted.

**Independent Test**: Save a project, restart the app, use Open Project to load the `.shero` file, verify annotations, zoom level, and pan are identical to the saved state.

---

### P1: Restore Full State ⭐ MVP

**User Story**: As a user, I want my complete annotation state restored when I reopen a project, including canvas position.

**Why P1**: Partial restoration degrades UX; users expect to resume exactly where they left off.

**Acceptance Criteria**:

1. WHEN a project is opened, THEN all annotation types present in the file (Rectangle, Ellipse, Arrow, Line, Freehand, Text, Blur, Pixelate, Redaction, Timestamp, NumberMarker, Callout) SHALL be deserialized and rendered correctly on the canvas.
2. WHEN a project is opened, THEN canvas zoom SHALL be set to the value in `view_state.zoom`.
3. WHEN a project is opened, THEN canvas pan offset SHALL be set to the values in `view_state.pan_x` and `view_state.pan_y`.
4. WHEN a project is opened, THEN annotation styles (stroke_color, stroke_width, fill_color) SHALL match the saved values exactly.

**Independent Test**: Add one instance of each of the 12 annotation types, save, reopen, compare visual output and model fields.

---

### P2: Auto Save

**User Story**: As a user, I want my project to be saved automatically after every modification so I don't lose work.

**Why P2**: Auto-save is essential for daily use but requires P1 (established save path) to be functional first. Enabled by default per ADR-001.

**Acceptance Criteria**:

1. WHEN Auto Save is enabled AND the project has an established save path, THEN system SHALL auto-save after every annotation Command execution (add, remove, move, resize, style change, text edit).
2. WHEN Auto Save is enabled AND project has a save path, THEN system SHALL auto-save after Undo.
3. WHEN Auto Save is enabled AND project has a save path, THEN system SHALL auto-save after Redo.
4. WHEN Auto Save is enabled AND the project has NOT been saved yet (no established path), THEN system SHALL NOT auto-save and SHALL NOT prompt the user.
5. WHEN Auto Save is disabled, THEN system SHALL stop auto-saving regardless of whether a save path exists.
6. WHEN auto-save fails (e.g., disk full, permission revoked), THEN system SHALL log a `warn!` message. No blocking error dialog is shown — auto-save failures must not interrupt the annotation workflow.

**Independent Test**: Save a project (establishing path), add an annotation, verify the `.shero` file's `modified_at` timestamp and annotation count are updated without any manual user action.

---

### P3: Project Metadata

**User Story**: As a user, I want project files to contain creation and modification timestamps so I can identify projects in the filesystem.

**Why P3**: Useful for file management but not critical for the editing workflow.

**Acceptance Criteria**:

1. WHEN a project is saved for the first time, THEN `metadata.created_at` SHALL be set to the current UTC timestamp in ISO 8601 / RFC 3339 format.
2. WHEN a project is updated (any save after the first), THEN `metadata.modified_at` SHALL be updated to the current UTC timestamp; `created_at` SHALL be preserved from the original save.
3. WHEN a project file is inspected, THEN `metadata.app_version` SHALL match the current application version string (from `Cargo.toml`).

**Independent Test**: Save, re-save after a change; open `.shero` and verify `created_at` is stable and `modified_at` advances.

---

## Edge Cases

- WHEN save is triggered to a read-only filesystem path, THEN system SHALL surface the I/O error in an error dialog.
- WHEN opening a `.shero` file with `"version": 2` (or higher) while the app only supports version 1, THEN system SHALL show an error: "This project was saved with a newer version of Screenshot Hero and cannot be opened."
- WHEN auto-save fires and the destination file is temporarily locked, THEN system SHALL log the failure and skip silently (no crash).
- WHEN the user triggers Save while no source image is loaded, THEN Save SHALL be disabled (greyed out).

---

## Requirement Traceability

| Requirement ID | Story | PRD Ref | Status |
|----------------|-------|---------|--------|
| PROJ-01 | P1: Save Project | FR-001, FR-003 | Pending |
| PROJ-02 | P1: Open Project | FR-002 | Pending |
| PROJ-03 | P1: Restore State — annotations | FR-004 | Pending |
| PROJ-04 | P1: Restore State — canvas view | FR-005, FR-006 | Pending |
| PROJ-05 | P3: Project Metadata | FR-007 | Pending |
| PROJ-06 | P2: Auto save on execute | FR-008, FR-009 | Pending |
| PROJ-07 | P2: Auto save on undo | FR-010 | Pending |
| PROJ-08 | P2: Auto save on redo | FR-011 | Pending |
| PROJ-09 | P2: Auto save toggle | FR-008 | Pending |

**Coverage:** 9 total, 0 mapped to tasks, 9 unmapped ⚠️

---

## Success Criteria

- [ ] User can save a project, restart the app, reopen, and all annotations and view state are identical to what was saved
- [ ] Auto-save writes the `.shero` file after every annotation change with no visible UI freeze
- [ ] Save/open round-trip is lossless for all 12 annotation types
- [ ] Invalid or missing `.shero` files produce a clear error message without clearing current work
- [ ] Window title reflects the current project file name after save or open
