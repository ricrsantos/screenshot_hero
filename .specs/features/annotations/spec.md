# Annotations — Specification

**PRD:** `docs/product/PRD-003-annotations.md`  
**ADRs:** `docs/architecture/ADR-002-annotation-model.md`, `docs/architecture/ADR-003-canvas-rendering.md`  
**Status:** Approved

---

## Problem Statement

After capturing or loading a screenshot (PRD-001) and navigating it (PRD-002), users have no way to communicate visually. The application is currently a viewer only. Users need to draw shapes, add text, highlight areas, and apply privacy effects before sharing, documenting, or reporting bugs. Without annotations, Screenshot Hero's core workflow — Capture → **Annotate** → Paste — is blocked at the second step.

---

## Goals

- [ ] User can create all 12 annotation types defined in ADR-002
- [ ] User can select, move, resize, and delete any annotation
- [ ] All annotation operations are undoable and redoable (Ctrl+Z / Ctrl+Y)
- [ ] Annotations render identically in canvas view and exported images (ADR-003 guarantee)
- [ ] All annotation coordinates are stored in image-space (invariant under zoom and pan)

---

## Out of Scope


| Feature                    | Reason                         |
| -------------------------- | ------------------------------ |
| Export annotated image     | PRD-005 (Export and Clipboard) |
| Save/load annotations      | PRD-004 (Project Management)   |
| Multi-selection            | ADR-003 explicitly defers this |
| Copy/paste annotations     | Not in PRD-003                 |
| Layer / Z-order management | Not in PRD-003                 |
| Snap-to-grid               | Not in PRD-003                 |
| Curved arrows              | Straight arrow only for v1     |
| Annotation grouping        | Not in PRD-003                 |


---

## User Stories

### P1: Draw Basic Shapes ⭐ MVP

**User Story:** As a user, I want to draw rectangles, ellipses, arrows, and lines on a screenshot so that I can highlight and direct attention to specific areas.

**Why P1:** These are the most universally needed annotation types. They prove the full pipeline — data model → engine → canvas rendering → interaction — works end-to-end.

**Acceptance Criteria:**

1. WHEN the Rectangle tool is active and the user drags on the canvas THEN system SHALL create a rectangle from drag-start to drag-end position
2. WHEN the Ellipse tool is active and the user drags THEN system SHALL create an ellipse bounded by the drag rectangle
3. WHEN the Arrow tool is active and the user drags THEN system SHALL create a line with an arrowhead at the drag-end point
4. WHEN the Line tool is active and the user drags THEN system SHALL create a straight line from drag-start to drag-end
5. WHEN the user releases the drag THEN system SHALL finalize the annotation and add it to the annotation list
6. WHEN drag distance is less than 4px in both axes THEN system SHALL discard the annotation (accidental-click protection)

**Independent Test:** Select Rectangle tool, drag 100px diagonally on canvas over a loaded image, release → rectangle renders over the image.

---

### P1: Select, Move, Resize, Delete ⭐ MVP

**User Story:** As a user, I want to select any annotation and reposition, resize, or delete it so that I can refine annotations without redoing them from scratch.

**Why P1:** Editing is as important as creation. Without it, every mistake requires a full redo.

**Acceptance Criteria:**

1. WHEN the Select tool is active and the user clicks on an annotation THEN system SHALL show selection handles (corner squares) around the annotation's bounding box
2. WHEN the Select tool is active and the user clicks on empty canvas THEN system SHALL deselect any current selection
3. WHEN an annotation is selected and the user drags its body THEN system SHALL move the annotation by the drag delta (in image-space)
4. WHEN an annotation is selected and the user drags a corner handle THEN system SHALL resize the annotation, keeping the opposite corner fixed
5. WHEN an annotation is selected and the user presses Delete THEN system SHALL remove the annotation from the canvas
6. WHEN the user presses Escape THEN system SHALL deselect the current annotation without modifying it

**Independent Test:** Draw a rectangle. Switch to Select tool. Click it — handles appear. Drag 50px right — it moves. Press Delete — it disappears.

---

### P1: Undo / Redo ⭐ MVP

**User Story:** As a user, I want to undo and redo annotation operations so that I can correct mistakes without fear.

**Why P1:** Non-destructive editing is a core architectural promise (ADR-003). Users must feel safe to annotate freely.

**Acceptance Criteria:**

1. WHEN the user presses Ctrl+Z THEN system SHALL undo the last annotation operation
2. WHEN the user presses Ctrl+Y or Ctrl+Shift+Z THEN system SHALL redo the last undone operation
3. WHEN there is nothing to undo THEN system SHALL do nothing (no error, no crash)
4. WHEN there is nothing to redo THEN system SHALL do nothing (no error, no crash)
5. WHEN a new annotation is created after an undo THEN system SHALL clear the redo stack
6. WHEN undo/redo buttons exist in the UI THEN system SHALL disable them when the respective stack is empty

**Undoable operations:** Add annotation, Delete annotation, Move annotation (final position committed on drag-end), Resize annotation (final bounds committed on drag-end), Style change on selected annotation.

**Independent Test:** Draw three annotations. Press Ctrl+Z three times — canvas is empty. Press Ctrl+Y three times — all three are back.

---

### P1: Annotation Style (Color + Stroke Width) ⭐ MVP

**User Story:** As a user, I want to set stroke color and stroke width so that I can visually distinguish annotations by intent.

**Why P1:** Monochrome annotations with fixed width cannot express priority, grouping, or emphasis.

**Acceptance Criteria:**

1. WHEN the user selects a color in the tool palette THEN system SHALL apply it to all subsequently created annotations
2. WHEN the user sets a stroke width THEN system SHALL apply it to all subsequently created annotations
3. WHEN the user selects an existing annotation and changes color THEN system SHALL update that annotation's color immediately (this is undo-able)
4. WHEN the user selects an existing annotation and changes stroke width THEN system SHALL update that annotation's stroke width immediately (this is undo-able)

**Independent Test:** Draw a rectangle with red color, stroke 2px. Select it. Change color to blue → rectangle turns blue immediately.

---

### P2: Freehand Drawing

**User Story:** As a user, I want to draw freehand paths on a screenshot so that I can make organic, informal annotations.

**Why P2:** Useful for circling areas or tracing irregular shapes, but basic geometric shapes cover most structured needs.

**Acceptance Criteria:**

1. WHEN the Freehand tool is active and the user drags THEN system SHALL trace the exact pointer path as a polyline
2. WHEN the user releases THEN system SHALL finalize the polyline as an annotation
3. WHEN the freehand path has fewer than 2 distinct sampled points THEN system SHALL discard it

**Independent Test:** Draw a squiggle — it renders exactly as drawn, following the pointer path.

---

### P2: Text Annotation

**User Story:** As a user, I want to place text labels on a screenshot so that I can explain elements in context.

**Why P2:** Important for richer annotations, but requires more complex UX (text input dialog) compared to drag-to-draw shapes.

**Acceptance Criteria:**

1. WHEN the Text tool is active and the user clicks on the canvas THEN system SHALL open a text input dialog at the click location
2. WHEN the user types text and confirms THEN system SHALL create a text annotation at the click position
3. WHEN the user double-clicks an existing text annotation in Select mode THEN system SHALL re-open the editor pre-filled with the existing text
4. WHEN the user presses Escape or cancels the dialog THEN system SHALL discard changes (no annotation created for new; no change for existing)
5. WHEN the user confirms with empty text THEN system SHALL discard the annotation

**Independent Test:** Click with Text tool, type "Hello World", confirm → label renders on canvas at the click position.

---

### P2: Privacy Effects (Blur, Pixelate, Redaction)

**User Story:** As a user, I want to blur, pixelate, or redact regions of a screenshot so that I can hide sensitive information before sharing.

**Why P2:** Privacy tools are essential for sharing screenshots in professional contexts, but secondary to shape annotations for MVP.

**Acceptance Criteria:**

1. WHEN the Blur tool is active and the user drags over an area THEN system SHALL render a blurred version of that region
2. WHEN the Pixelate tool is active and the user drags THEN system SHALL render a mosaic (pixel-grid) effect over that region
3. WHEN the Redaction tool is active and the user drags THEN system SHALL render a filled opaque rectangle covering that region
4. WHEN a blur/pixelate annotation is moved or resized THEN system SHALL re-render the effect at the new area from the original image pixels (non-destructive)
5. WHEN the original image is unchanged THEN all effects SHALL be generated at render time from original pixels (per ADR-003)

**Independent Test:** Draw Blur over a password field → content is obscured. Move the blur region → effect follows to new position.

---

### P2: Timestamp Annotation

**User Story:** As a user, I want to add a timestamp label so that screenshots are self-documenting about when they were taken.

**Why P2:** Useful for documentation workflows; the auto-formatting behavior differentiates it from plain text.

**Acceptance Criteria:**

1. WHEN the Timestamp tool is active and the user clicks THEN system SHALL create a text annotation with the current date and time
2. WHEN a timestamp format is configured (per PRD-006) THEN system SHALL use that format
3. WHEN no timestamp format is configured THEN system SHALL use the default format `YYYY-MM-DD HH:MM:SS`
4. WHEN a timestamp annotation is placed THEN its text content SHALL be immutable (not re-editable after creation)

**Independent Test:** Click with Timestamp tool → annotation shows the current date and time in the configured format.

---

### P2: Number Marker

**User Story:** As a user, I want to place auto-numbered circle markers so that I can create step-by-step numbered reference diagrams.

**Why P2:** The auto-increment behavior is a key differentiator from plain text; it supports common documentation and bug-report workflows.

**Acceptance Criteria:**

1. WHEN the Number Marker tool is active and the user clicks THEN system SHALL place a circle with the next sequential number inside it
2. WHEN this is the first marker THEN the number SHALL be 1
3. WHEN subsequent markers are placed THEN each number SHALL increment by 1
4. WHEN a marker is deleted THEN remaining markers SHALL NOT renumber (gaps are acceptable for v1)
5. WHEN all markers are undone THEN the counter SHALL reset and the next placement SHALL be 1

**Independent Test:** Place three markers → they show 1, 2, 3. Delete the second → remaining show 1 and 3 (no renumber).

---

### P3: Callout

**User Story:** As a user, I want to add callout bubbles with a pointer arrow so that I can annotate with rich, self-contained labels.

**Why P3:** Text + Arrow together achieves most of the same value; callout is a polish feature.

**Acceptance Criteria:**

1. WHEN the Callout tool is active and the user drags THEN system SHALL create a bubble at the drag-end position with a pointer to the drag-start (anchor point)
2. WHEN the user confirms the text input THEN system SHALL render the callout with the text inside the bubble and a pointer line to the anchor
3. WHEN the callout is moved THEN bubble and anchor SHALL move together (rigid body)

**Independent Test:** Drag callout from anchor to bubble position, enter "Note here" → bubble with pointer renders correctly.

---

## Edge Cases

- WHEN canvas has no image loaded THEN system SHALL disable all annotation tools
- WHEN drag distance < 4px in both axes THEN system SHALL discard the annotation (accidental click)
- WHEN two annotations overlap at the click point THEN system SHALL select the topmost one (highest index in the annotation list)
- WHEN undo is triggered while a drawing gesture is in progress THEN system SHALL complete or discard the active gesture first before undoing
- WHEN canvas is zoomed or panned THEN system SHALL store annotation coordinates in image-space (not screen-space)
- WHEN resize would produce negative width or height (bounds inversion) THEN system SHALL clamp to a minimum of 4×4px

---

## Requirement Traceability


| Requirement ID | Story                             | PRD Ref        | Status  |
| -------------- | --------------------------------- | -------------- | ------- |
| ANNO-01        | P1: Draw basic shapes — Rectangle | FR-001         | Pending |
| ANNO-02        | P1: Draw basic shapes — Ellipse   | FR-001         | Pending |
| ANNO-03        | P1: Draw basic shapes — Arrow     | FR-001         | Pending |
| ANNO-04        | P1: Draw basic shapes — Line      | FR-001         | Pending |
| ANNO-05        | P1: Select annotation             | FR-002         | Pending |
| ANNO-06        | P1: Move annotation               | FR-003         | Pending |
| ANNO-07        | P1: Resize annotation             | FR-004         | Pending |
| ANNO-08        | P1: Delete annotation             | FR-005         | Pending |
| ANNO-09        | P1: Undo                          | —              | Pending |
| ANNO-10        | P1: Redo                          | —              | Pending |
| ANNO-11        | P1: Style — color                 | FR-006         | Pending |
| ANNO-12        | P1: Style — stroke width          | FR-007         | Pending |
| ANNO-13        | P2: Freehand drawing              | FR-001         | Pending |
| ANNO-14        | P2: Text annotation               | FR-001, FR-008 | Pending |
| ANNO-15        | P2: Blur effect                   | FR-001         | Pending |
| ANNO-16        | P2: Pixelate effect               | FR-001         | Pending |
| ANNO-17        | P2: Redaction                     | FR-001         | Pending |
| ANNO-18        | P2: Timestamp                     | FR-001, FR-010 | Pending |
| ANNO-19        | P2: Number Marker                 | FR-001, FR-009 | Pending |
| ANNO-20        | P3: Callout                       | FR-001         | Pending |


**Coverage:** 20 total, 0 mapped to tasks, 20 unmapped ⚠️

---

## Success Criteria

- [ ] All 12 annotation types can be created on a loaded screenshot
- [ ] Select, move, resize, and delete work for all annotation types
- [ ] Ctrl+Z / Ctrl+Y undo and redo all annotation operations
- [ ] Color and stroke width apply to new annotations and update selected annotations immediately
- [ ] All annotation coordinates are image-space (invariant under zoom/pan)
- [ ] `cargo test --lib` passes with no regressions
- [ ] App runs without crashes under `cargo run` with all annotation types exercised manually