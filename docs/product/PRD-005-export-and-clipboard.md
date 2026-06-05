# docs/product/PRD-005-export-and-clipboard.md

# PRD-005 - Export and Clipboard

Status: Draft

---

## Objective

Allow users to export annotated images and quickly share them.

---

## Functional Requirements

### Export

#### FR-001

Export PNG.

#### FR-002

Export JPEG.

#### FR-003

Preserve original image.

---

### Clipboard

#### FR-004

Copy image to clipboard.

#### FR-005

Copy rendered result.

---

### Auto Clipboard Update

#### FR-006

Feature enabled by default.

#### FR-007

Clipboard updated after annotation changes.

#### FR-008

Clipboard updated after undo.

#### FR-009

Clipboard updated after redo.

#### FR-010

Clipboard updates must be debounced.

---

### Auto Export

#### FR-011

Optional feature.

#### FR-012

Export automatically after modifications.

#### FR-013

Save beside original screenshot.

#### FR-014

Use configurable suffix.

Default:

```text
_shero
```

---

## Acceptance Criteria

✅ PNG export works

✅ JPEG export works

✅ Clipboard update works

✅ Auto clipboard works

✅ Auto export works