# Tech Stack

**Analyzed:** 2026-06-15

## Core

- Framework: GTK4 (`gtk4` 0.9) + Libadwaita (`libadwaita` 0.7)
- Language: Rust (edition 2021, stable toolchain)
- Runtime: Linux desktop (GNOME/XDG Portal environment)
- Package manager: Cargo

## Desktop Platform Integration

- Screenshot portal: `ashpd` 0.9 (`org.freedesktop.portal.Screenshot`)
- Settings persistence: GSettings via `gio::Settings` + schema `com.screenshot_hero.ScreenshotHero`
- Icons/resources: GResource + icon theme setup (`src/resources.rs`)
- Distribution/runtime packaging: Flatpak (`org.gnome.Platform//50`, `org.gnome.Sdk//50`)

## Data and Serialization

- JSON serialization: `serde` + `serde_json`
- IDs: `uuid` (with `serde` feature)
- Time metadata: `chrono`

## Image and Rendering

- Image buffers: `gdk-pixbuf` 0.20
- 2D drawing: Cairo (via GTK stack)
- Text rendering: `pango` + `pangocairo`
- Export formats: PNG and JPEG via `Pixbuf::savev`

## Logging and Diagnostics

- Logging facade: `log`
- Logger backend: `env_logger`

## Testing

- Unit tests: built-in Rust test harness (`cargo test --lib`)
- No dedicated UI/E2E framework in repository at this time

## Development and Build Tools

- Build: `cargo build`, `cargo build --release`
- Native run: `cargo run` / `cargo run -- --capture`
- Flatpak build: `flatpak-builder` with vendored Cargo sources (`build/cargo-sources.json`)
