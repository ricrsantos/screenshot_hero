# Screenshot Hero

**Vision:** A Linux-native screenshot annotation tool that enables users to capture, annotate, and share screenshots without leaving their workflow.
**For:** Linux/GNOME desktop users who frequently capture and annotate screenshots for documentation, bug reports, or communication.
**Solves:** The friction of needing multiple tools to go from capturing a screenshot to having a shareable annotated image — Screenshot Hero collapses the workflow into a single native app.

---

## Goals

- Provide a seamless Capture → Annotate → Clipboard workflow in a single application
- Deliver a native GNOME experience with Flatpak distribution, offline-only, privacy-first

---

## Tech Stack

**Core:**
- Language: Rust (stable)
- UI Framework: GTK4 + Libadwaita
- Desktop Integration: ashpd + XDG Desktop Portals
- Distribution: Flatpak

**Key dependencies:** `gtk4`, `libadwaita`, `ashpd`, `serde` + `serde_json`, `uuid`

---

## Scope

**v1 includes:**
- Screenshot capture via GNOME Screenshot Portal (XDG)
- Loading existing PNG/JPEG files
- Canvas with zoom and pan
- Annotation tools: rectangle, arrow, ellipse, line, freehand, text, blur, pixelate, redaction, timestamp, numbered marker, callout
- Project save/load (`.shero` JSON format)
- Export PNG/JPEG
- Auto clipboard update (debounced, 300ms)
- Settings: appearance, auto-save, auto-export, clipboard, log level, timestamp

**Explicitly out of scope:**
- Cloud sync or screenshot upload
- Windows/macOS support
- Collaborative editing
- Video capture or recording
- OCR or AI-based features

---

## Constraints

- **Technical:** Flatpak-first, offline-only, no network access at runtime
- **Platform:** Linux + GNOME (XDG Portals required)
- **Distribution:** Flatpak is the primary release channel
