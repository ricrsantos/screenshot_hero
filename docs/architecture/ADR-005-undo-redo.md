# ADR-005 - Undo / Redo Architecture

Status: Accepted

---

## Context

Annotation editing requires reliable undo and redo.

The solution must be:

- Predictable
- Fast
- Easy to maintain

---

## Decision

Command Pattern will be used.

---

## Command Interface

```rust
pub trait Command {
    fn execute(&mut self);
    fn undo(&mut self);
}
```

---

## Examples

### Create Annotation

```rust
CreateAnnotationCommand
```

---

### Delete Annotation

```rust
DeleteAnnotationCommand
```

---

### Move Annotation

```rust
MoveAnnotationCommand
```

---

### Resize Annotation

```rust
ResizeAnnotationCommand
```

---

### Edit Text

```rust
EditTextCommand
```

---

## History Stacks

```text
Undo Stack
Redo Stack
```

---

## Execution Flow

```text
Execute Command
↓
Push To Undo Stack
↓
Clear Redo Stack
```

---

## Undo Flow

```text
Pop Undo Stack
↓
Undo Command
↓
Push To Redo Stack
```

---

## Redo Flow

```text
Pop Redo Stack
↓
Execute Command
↓
Push To Undo Stack
```

---

## Clipboard Updates

Clipboard refresh must occur after:

- Execute
- Undo
- Redo

---

## Auto Save

Auto save must occur after:

- Execute
- Undo
- Redo

---

## Future Compatibility

This architecture enables:

- Macro commands
- Batch operations
- Multi-selection commands

---

## Consequences

Benefits:

- Reliable editing
- Predictable behavior
- Extensible architecture

Accepted.