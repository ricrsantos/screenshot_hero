# ADR-001 - System Architecture

Status: Accepted

---

## Context

Screenshot Hero is a Linux-native screenshot annotation tool.

The primary workflow is:

Capture
→ Annotate
→ Paste

The architecture must support:

* Flatpak distribution
* Offline operation
* Fast startup
* Future extensibility

---

## Decision

The application will be implemented using:

* Rust
* GTK4
* Libadwaita
* ashpd
* XDG Desktop Portals

The application will follow a Flatpak-first architecture and use XDG Desktop Portals for desktop integration whenever applicable.

The application will operate primarily offline and process all screenshots locally.

The application will use a centralized logging subsystem.

Supported log levels:

* Trace
* Debug
* Info
* Warn
* Error

The active log level shall be configurable through the Settings screen.

The default log level shall be Info.

Logs are intended for diagnostics and troubleshooting only and must never be shown directly to end users.

---

## High-Level Architecture

```text
+--------------------+
|   GTK4 UI Layer    |
+--------------------+
          |
          v
+--------------------+
| Application Layer  |
+--------------------+
          |
          v
+--------------------+
| Annotation Engine  |
+--------------------+
          |
          v
+--------------------+
| Rendering Engine   |
+--------------------+
          |
          v
+--------------------+
| Persistence Layer  |
+--------------------+
```

---

## Main Modules

### Capture Module

Responsibilities:

* Request screenshots
* Receive portal responses
* Load source images

---

### Canvas Module

Responsibilities:

* Display images
* Display annotations
* Manage zoom
* Manage pan

---

### Annotation Engine

Responsibilities:

* Create annotations
* Edit annotations
* Remove annotations
* Selection management

Supported annotation types:

* Rectangle
* Arrow
* Ellipse
* Line
* Freehand
* Text
* Blur
* Pixelate
* Redaction
* Timestamp
* Numbered Marker
* Callout

---

### Rendering Engine

Responsibilities:

* Render source image
* Render annotations
* Generate export image
* Generate clipboard image

---

### Persistence Layer

Responsibilities:

* Save projects
* Load projects
* Export PNG
* Export JPEG

---

## Project File Format

Internal editable format:

```text
.shero
```

Must contain:

* Original image reference
* Annotation data
* View state
* Project metadata

Detailed definition will be documented in ADR-004.

---

## Clipboard Strategy

Clipboard is a first-class feature.

Default behavior:

* Auto Clipboard Update = Enabled

Every significant project change should refresh clipboard contents.

Clipboard updates must be debounced.

Suggested debounce:

300 ms

---

## Auto Save Strategy

Default:

Enabled

Save editable project automatically after modifications.

---

## Auto Export Strategy

Default:

Disabled

When enabled:

Generate:

```text
original_name_shero.png
```

in the same directory as the original screenshot.

---

## Settings Architecture

Supported settings:

### Appearance

* Follow System
* Light
* Dark

### Timestamp

* Enabled
* Disabled

### Timestamp Format

User configurable.

### Auto Save Project

Enabled / Disabled

### Auto Export Image

Enabled / Disabled

### Auto Export Suffix

Default:

```text
_shero
```

### Auto Clipboard Update

Enabled / Disabled

Default:

Enabled

---

## Non-Functional Requirements

### Offline First

No network dependency.

### Privacy First

No screenshot upload.

### Flatpak First

Flatpak is the primary distribution target.

### Linux First

Linux is the primary supported platform.

---

## Consequences

Benefits:

* Native GNOME experience
* Strong Flatpak integration
* Good security model
* Maintainable architecture
* Extensible annotation system

Tradeoffs:

* Linux-focused implementation
* GTK4 dependency
* Portal dependency

These tradeoffs are accepted.
