# Screenshot Hero - Agent Guidelines

## Project Overview

Screenshot Hero is a Linux-native screenshot annotation application focused on:

* Technical documentation
* Software development
* Support teams
* Knowledge sharing

The application must be:

* GTK4 native
* Libadwaita based
* Flatpak-first
* Offline-first
* Privacy-first

No cloud services are required.

No AI services are required.

All screenshots remain local.

---

## Official Language

### UI Language

All user-visible text must be written in English.

Examples:

* File
* Edit
* Settings
* Export
* Save Project
* Open Project

Do not introduce additional languages.

---

### Source Code Language

All source code must be written in English.

This includes:

* File names
* Folder names
* Variables
* Functions
* Structs
* Traits
* Comments
* Documentation

Example:

Good:

```rust
fn save_project() {}
```

Bad:

```rust
fn salvar_projeto() {}
```

---

## Technology Stack

* Rust
* GTK4
* Libadwaita
* ashpd
* Flatpak
* XDG Desktop Portals

---

## Architecture References

Before implementing features consult:

* docs/architecture/ADR-001-system-architecture.md
* docs/architecture/ADR-002-annotation-model.md
* docs/architecture/ADR-003-canvas-rendering.md
* docs/architecture/ADR-004-project-file-format.md
* docs/architecture/ADR-005-undo-redo.md

Architectural decisions documented in ADRs take precedence over implementation preferences.

---

## Research References

Before implementing screenshot functionality consult:

* docs/research/poc-summary.md

PoCs are considered validated technical evidence.

Do not replace validated approaches without justification.

---

## Development Principles

Prefer:

* Simplicity
* Readability
* Maintainability

Avoid:

* Premature optimization
* Unnecessary abstractions
* Over-engineering

---

## Testing Requirements

All implemented features must include:

* Unit tests when appropriate
* Integration tests when appropriate

Critical workflows must be manually validated.

---

## Definition of Done

A feature is complete only when:

* Code compiles
* Tests pass
* Lints pass
* Documentation is updated
* Feature matches the PRD
* Feature respects ADR decisions

---

## Flatpak Requirements

Flatpak is the primary distribution target.

New features must be evaluated for Flatpak compatibility.

Portal-based APIs must be preferred over direct system access whenever possible.

---

## Documentation Structure

* docs/research = PoCs
* docs/architecture = ADRs
* docs/product = PRDs
* specs = TLC Spec Driven outputs

---

## User Experience Principles

The application should feel:

* Fast
* Lightweight
* Native
* Predictable

The primary workflow is:

Capture
→ Annotate
→ Paste

All design decisions should prioritize this workflow.

## Logging Requirements

The application must use structured logging.

Every significant operation should emit logs at the appropriate level.

Supported log levels:

- Trace
- Debug
- Info
- Warn
- Error

Guidelines:

- Error:
  unrecoverable failures, unexpected exceptions, corrupted data

- Warn:
  recoverable failures, invalid user input, fallback behavior

- Info:
  important user-visible operations
  (project opened, project saved, screenshot captured, export completed)

- Debug:
  internal workflow information useful during development

- Trace:
  very detailed execution flow

Avoid logging sensitive information.

Prefer structured logging over plain string concatenation.

Logs must never be shown to the user through the UI.

Logs are intended for diagnostics only.

User-facing errors must be presented through dialogs,
notifications or status messages.

## Documentation Lookup

When external documentation is required, prefer Context7 documentation sources when available.

Examples:

- GTK4
- Libadwaita
- ashpd
- Tokio
- Serde
- Image crate
- Rust standard library

Use Context7 to validate APIs, examples and implementation details.

Do not use external documentation to override project requirements, ADR decisions or validated PoC findings.

Priority order:

1. PRDs
2. ADRs
3. PoC Summary
4. Context7 Documentation
5. General Web Search
