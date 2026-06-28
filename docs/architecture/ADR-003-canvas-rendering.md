# ADR-003 - Canvas Rendering

Status: Accepted

---

## Context

The application must render:

- Original screenshot
- Annotation overlays
- Selection handles
- Zoom and pan operations

Rendering must remain responsive.

---

## Decision

The rendering pipeline will be layered.

---

## Rendering Order

```text
Background
↓
Source Screenshot
↓
Blur / Pixelate Effects
↓
Shapes
↓
Text
↓
Selection Handles
↓
Temporary Interaction Overlay
```

---

## Canvas Responsibilities

The canvas must support:

- Zoom
- Pan
- Selection
- Multi-selection (future)
- Hit testing

---

## Zoom

Supported range:

```text
10% → 800%
```

Default:

```text
100%
```

---

## Pan

Mouse middle button:

```text
Pan Canvas
```

---

## Rendering Strategy

Annotations are never rasterized permanently.

The canvas always renders:

```text
Source Image
+
Annotations
```

in real time.

---

## Export Rendering

Export uses the same rendering pipeline.

This guarantees:

```text
Canvas View == Export Result
```

---

## Selection Handles

Visible only when selected.

Handles support:

- Resize
- Move

---

## Effects

Blur and Pixelate are applied during render.

Original image remains untouched.

---

## Consequences

Benefits:

- Non-destructive editing
- Consistent exports
- Easier undo/redo

Accepted.