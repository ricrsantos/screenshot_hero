# Screenshot Hero Development Guidelines

## Technology

- Rust stable
- GTK4
- gtk4-rs
- cairo-rs
- gio-rs
- glib-rs

## Architecture

- Feature-first architecture
- No global mutable state
- Prefer composition over inheritance
- Minimize unsafe Rust

## Testing

All business logic must be unit tested.

## UI

Follow GNOME Human Interface Guidelines.

## Flatpak

Application must remain Flatpak compatible.

## Performance

Application startup < 500ms.

## Product Goal

Fastest screenshot annotation workflow available on GNOME Wayland.