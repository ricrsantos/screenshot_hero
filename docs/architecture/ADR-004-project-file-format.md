# ADR-004 - Project File Format

Status: Accepted

---

## Context

Users must be able to:

- Save work
- Resume work later
- Continue editing annotations

A dedicated editable format is required.

---

## Decision

Screenshot Hero will use:

```text
.shero
```

as the editable project format.

---

## Format

The format will be JSON-based.

---

## Example

```json
{
  "version": 1,
  "source_image": "Screenshot From 2026-06-05.png",
  "annotations": [],
  "settings": {},
  "metadata": {}
}
```

---

## Required Sections

### Metadata

```json
{
  "created_at": "...",
  "modified_at": "...",
  "app_version": "..."
}
```

---

### Source Image

Stores:

```json
{
  "path": "...",
  "width": 1920,
  "height": 1080
}
```

---

### Annotations

Stores all annotation data.

---

### View State

Stores:

```json
{
  "zoom": 1.0,
  "pan_x": 0,
  "pan_y": 0
}
```

---

## Versioning

Every project must contain:

```json
{
  "version": 1
}
```

Future migrations must use version upgrades.

---

## Human Readability

The file must remain readable and diffable.

Binary formats are not allowed.

---

## Consequences

Benefits:

- Easy debugging
- Easy migration
- Git-friendly

Accepted.