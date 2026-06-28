# ADR-002 - Annotation Model

Status: Accepted

---

## Context

Screenshot Hero must support multiple annotation types while maintaining:

- Consistent behavior
- Simple rendering
- Efficient serialization
- Undo/Redo support

The system requires a common model that represents all annotations.

---

## Decision

All annotations will implement a common Annotation model.

```rust
pub struct Annotation {
    pub id: Uuid,
    pub kind: AnnotationKind,
    pub bounds: Rect,
    pub style: AnnotationStyle,
}
```

---

## Annotation Types

Supported annotation types:

```rust
pub enum AnnotationKind {
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Freehand,
    Text,
    Blur,
    Pixelate,
    Redaction,
    Timestamp,
    NumberMarker,
    Callout,
}
```

---

## Common Properties

Every annotation must support:

- Unique identifier
- Position
- Size
- Selection state
- Visibility

---

## Styling

```rust
pub struct AnnotationStyle {
    pub stroke_width: f32,
    pub stroke_color: Color,
    pub fill_color: Option<Color>,
}
```

---

## Text Annotations

Additional properties:

```rust
pub struct TextAnnotation {
    pub text: String,
    pub font_size: f32,
}
```

---

## Blur and Pixelate

Blur and Pixelate annotations do not store pixels.

They store only:

```rust
pub struct EffectAnnotation {
    pub area: Rect,
}
```

Effects are generated during rendering.

---

## Redaction

Redaction permanently hides content visually.

Internally it is rendered as a filled rectangle.

---

## Numbered Markers

Store:

```rust
pub struct NumberMarker {
    pub number: u32,
}
```

Numbers are generated sequentially.

---

## Callouts

Store:

- Text
- Anchor point
- Bubble position

---

## Serialization

Annotations must be serializable.

Preferred format:

```rust
serde
```

---

## Consequences

Benefits:

- Simple rendering pipeline
- Easy persistence
- Easy undo/redo

Tradeoffs:

- Some annotation-specific fields require additional structures

Accepted.