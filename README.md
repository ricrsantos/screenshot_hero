# Screenshot Hero

Linux-native screenshot annotation application built with **Rust**, **GTK4**, and **Libadwaita**.

Screenshot Hero lets you capture a screen region via the GNOME/XDG Screenshot Portal or open an existing PNG/JPEG file, then view the image in the app canvas — the first step in the **Capture → Annotate → Paste** workflow.

The app is **Flatpak-first**, **offline-only**, and **privacy-first**: screenshots stay on your machine.

## Requirements

### Native development (Cargo)

- Rust toolchain (stable), via [rustup](https://rustup.rs/)
- GTK 4 and Libadwaita development libraries
- A GNOME/Wayland (or X11) desktop session for running the UI
- XDG Desktop Portals (for screenshot capture)

**Fedora:**

```bash
sudo dnf install gtk4-devel libadwaita-devel gdk-pixbuf2-devel gcc pkg-config
```

**Debian / Ubuntu:**

```bash
sudo apt install libgtk-4-dev libadwaita-1-dev libgdk-pixbuf-2.0-dev build-essential pkg-config
```

**Arch Linux:**

```bash
sudo pacman -S gtk4 libadwaita gdk-pixbuf-2.0 base-devel
```

### Flatpak

- [Flatpak](https://flatpak.org/) with the Flathub remote configured
- GNOME Platform and SDK 50

```bash
flatpak install flathub org.gnome.Platform//50 org.gnome.Sdk//50
```

## Build

Clone the repository and build with Cargo:

```bash
git clone <repository-url>
cd screenshot_hero
cargo build
```

Release build:

```bash
cargo build --release
```

Run unit tests:

```bash
cargo test --lib
```

## Run with Cargo (development)

From the project root, build and run the application directly:

```bash
cargo run
```

With logging enabled:

```bash
RUST_LOG=info cargo run
```

Use **New Screenshot** to capture via the portal, or **Open File** to load a PNG/JPEG image.

> Screenshot capture requires a running portal (`org.freedesktop.portal.Desktop`) and works best on GNOME/Wayland.

## Flatpak: build, install, and run

Flatpak is the primary distribution target. The manifest lives at `build/com.screenshot_hero.ScreenshotHero.yml`.

### Update vendored Cargo sources (when dependencies change)

If you modify `Cargo.toml` or `Cargo.lock`, regenerate the offline sources before building the Flatpak:

```bash
curl -LO https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/master/cargo/flatpak-cargo-generator.py
python3 flatpak-cargo-generator.py Cargo.lock -o build/cargo-sources.json
```

### Build only (validation)

```bash
flatpak-builder --build-only build-dir build/com.screenshot_hero.ScreenshotHero.yml --force-clean
```

### Install and run

```bash
flatpak-builder --user --install build-dir build/com.screenshot_hero.ScreenshotHero.yml --force-clean
flatpak run com.screenshot_hero.ScreenshotHero
```

> Always validate the installed app with `flatpak run`, not only `flatpak-builder --build-only`. The installed runtime matches what end users get.

## Project layout

```
src/
├── main.rs              # Entry point
├── application.rs       # GApplication subclass
├── ui/window/           # Main window and actions
├── canvas/              # Image display widget
├── capture/             # Portal capture and file loading
└── models/              # ImageData model
build/
└── com.screenshot_hero.ScreenshotHero.yml   # Flatpak manifest
```

## License

MIT — see [LICENSE](LICENSE).
