# POC-003-04 - Flatpak Screenshot Rendering Validation

## Objective

Validate that a screenshot captured through the GNOME Screenshot Portal can be rendered inside a GTK4 + Libadwaita application running as an installed Flatpak.

This POC is considered successful when:

1. The app opens a graphical window.
2. The user captures a screenshot via portal UI.
3. The captured image is displayed inside the app window immediately after capture.

---

## Scope

Project: `POC-003-Flatpak`  
Binary: `screenshot-poc`  
Language: Rust (edition 2024)

Main requirements covered:

- Libadwaita `ApplicationWindow`
- Header bar area and `Take Screenshot` button
- Interactive screenshot flow via `ashpd`
- URI to filesystem path conversion
- In-window image rendering
- Aspect ratio preservation
- Graceful user-facing error handling (no `unwrap`, no panic flow for expected failures)

---

## Implementation Summary

## 1) GUI application architecture (`src/main.rs`)

The app now runs as a GTK4 + Libadwaita desktop UI instead of a console-only flow.

Main UI components:

- `adw::Application`
- `adw::ApplicationWindow`
- `adw::HeaderBar`
- `gtk::Button` ("Take Screenshot")
- `gtk::Picture` for image preview
- `gtk::Label` for status/error messages

Flow:

1. User clicks **Take Screenshot**.
2. Button is disabled and UI status is updated.
3. A worker thread starts a Tokio runtime.
4. The worker calls `ashpd::desktop::screenshot::Screenshot` with interactive and modal options.
5. The returned URI is parsed and converted into a local path.
6. Result is sent back to the UI thread through an `mpsc` channel.
7. UI thread updates:
   - success -> picture displays image
   - failure/cancel -> label shows user-friendly message
8. Button is re-enabled.

Rendering details:

- `gtk::Picture::set_keep_aspect_ratio(true)`
- `set_hexpand(true)` and `set_vexpand(true)`
- `set_can_shrink(true)`

This ensures fit-to-available-space behavior without implementing zoom or scrolling controls.

## 2) Dependencies (`Cargo.toml`)

The dependency set was updated for a GTK/Libadwaita desktop app:

- `ashpd = "0.12"`
- `gio = "0.20"`
- `glib = "0.20"`
- `gtk4 = "0.9"`
- `libadwaita = "0.7"`
- `tokio = { version = "1", features = ["rt", "rt-multi-thread"] }`
- `url = "2.5.8"`

## 3) Flatpak runtime strategy (`io.github.screenshothero.Poc003.yml`)

To run a GTK4 + Libadwaita app correctly in Flatpak, runtime was aligned to GNOME:

- `runtime: org.gnome.Platform`
- `runtime-version: "50"`
- `sdk: org.gnome.Sdk`

Portal and desktop permissions used:

- `--share=ipc`
- `--socket=session-bus`
- `--talk-name=org.freedesktop.portal.Desktop`
- `--filesystem=home`
- `--socket=x11`
- `--socket=wayland`

The manifest installs a prebuilt `target/release/screenshot-poc` binary into `/app/bin/screenshot-poc`.

---

## Commands Executed

### Runtime/SDK checks

```bash
flatpak list --runtime
flatpak install -y flathub org.gnome.Sdk//50
flatpak run --command=sh org.gnome.Sdk//50 -c 'export PATH=/usr/lib/sdk/rust-stable/bin:$PATH; which cargo; pkg-config --modversion gtk4; pkg-config --modversion libadwaita-1'
```

### Build release binary inside GNOME SDK

```bash
flatpak run --share=network --filesystem="/home/ricardo/data/projects/screenshot_hero/screenshot_hero_poc/POC-003-Flatpak" --command=sh org.gnome.Sdk//50 -c 'export PATH=/usr/lib/sdk/rust-stable/bin:$PATH; cd /home/ricardo/data/projects/screenshot_hero/screenshot_hero_poc/POC-003-Flatpak; cargo build --release'
```

### Install Flatpak app and run

```bash
flatpak-builder --user --install --force-clean build-dir io.github.screenshothero.Poc003.yml
flatpak run io.github.screenshothero.Poc003
```

---

## Technical Analysis

### Issue A - Installed app was still CLI behavior

The app was initially running old console behavior because the manifest copied a prebuilt binary (`target/release/screenshot-poc`) that had not yet been rebuilt from the new GTK source.

Resolution:

- Rebuild release binary from current source.
- Reinstall Flatpak app.

### Issue B - Freedesktop SDK lacked required GTK/adwaita development setup for this GUI flow

Host-side build and Freedesktop SDK checks showed missing GTK4/pkg-config coverage for this scenario.

Resolution:

- Move Flatpak runtime/SDK to GNOME 50 (`org.gnome.Platform` + `org.gnome.Sdk`).

### Issue C - `AdwApplicationWindow` titlebar API misuse

`gtk_window_set_titlebar()` is not supported for `AdwApplicationWindow`, which caused runtime abort when using `window.set_titlebar(...)`.

Resolution:

- Remove `set_titlebar`.
- Place `adw::HeaderBar` as a top widget in window content layout.

### Issue D - API mismatch for `ToolbarView`

`adw::ToolbarView` was not available in the selected crate version.

Resolution:

- Keep layout simple using a vertical `gtk::Box` with header + status + picture.

---

## Final Code

## `Cargo.toml` (dependencies section)

```toml
[dependencies]
ashpd = "0.12"
gio = "0.20"
glib = "0.20"
gtk4 = "0.9"
libadwaita = "0.7"
tokio = { version = "1", features = ["rt", "rt-multi-thread"] }
url = "2.5.8"
```

## `src/main.rs`

```rust
use ashpd::desktop::screenshot::Screenshot;
use adw::prelude::*;
use gio::File;
use glib::ControlFlow;
use gtk4 as gtk;
use libadwaita as adw;
use std::path::PathBuf;
use url::Url;

const APP_ID: &str = "io.github.screenshothero.Poc003";
const WINDOW_TITLE: &str = "Screenshot Hero POC 003-04";

fn main() -> glib::ExitCode {
    let app = adw::Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &adw::Application) {
    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title(WINDOW_TITLE)
        .default_width(1000)
        .default_height(700)
        .build();

    let header = adw::HeaderBar::new();
    let take_button = gtk::Button::with_label("Take Screenshot");
    header.pack_start(&take_button);

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();
    content.append(&header);

    let status_label = gtk::Label::new(Some("Click 'Take Screenshot' to capture and preview."));
    status_label.set_xalign(0.0);
    content.append(&status_label);

    let picture = gtk::Picture::new();
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_can_shrink(true);
    picture.set_keep_aspect_ratio(true);
    content.append(&picture);

    window.set_content(Some(&content));
    window.present();

    let (sender, receiver) = std::sync::mpsc::channel::<Result<PathBuf, String>>();

    let take_button_for_result = take_button.clone();
    let status_label_for_result = status_label.clone();
    let picture_for_result = picture.clone();

    glib::idle_add_local(move || {
        while let Ok(result) = receiver.try_recv() {
            take_button_for_result.set_sensitive(true);

            match result {
                Ok(path) => {
                    if path.exists() {
                        let file = File::for_path(path);
                        picture_for_result.set_file(Some(&file));
                        status_label_for_result.set_text("Screenshot captured successfully.");
                    } else {
                        status_label_for_result.set_text("Could not open the captured image file.");
                    }
                }
                Err(message) => status_label_for_result.set_text(&message),
            }
        }

        ControlFlow::Continue
    });

    let status_label_for_click = status_label.clone();
    take_button.connect_clicked(move |button| {
        button.set_sensitive(false);
        status_label_for_click.set_text("Waiting for screenshot selection...");

        let sender_for_thread = sender.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            let result = match runtime {
                Ok(rt) => rt.block_on(capture_screenshot_path()),
                Err(error) => Err(format!("Internal runtime initialization failed: {error}")),
            };

            if sender_for_thread.send(result).is_err() {
                eprintln!("Warning: could not deliver screenshot result to UI.");
            }
        });
    });
}

async fn capture_screenshot_path() -> Result<PathBuf, String> {
    let request = Screenshot::request()
        .interactive(true)
        .modal(true)
        .send()
        .await;
    let response = match request {
        Ok(r) => r,
        Err(error) => return Err(map_portal_error(error.to_string())),
    };

    let response = response
        .response()
        .map_err(|error| map_portal_error(error.to_string()))?;

    let uri = response.uri();

    let parsed_uri =
        Url::parse(uri.as_str()).map_err(|_| "Received an invalid screenshot URI.".to_string())?;

    parsed_uri
        .to_file_path()
        .map_err(|_| "Could not convert screenshot URI to a file path.".to_string())
}

fn map_portal_error(error_text: String) -> String {
    if error_text.to_lowercase().contains("cancel") {
        "Screenshot capture was canceled.".to_string()
    } else {
        format!("Screenshot capture failed: {error_text}")
    }
}
```

## `io.github.screenshothero.Poc003.yml`

```yaml
app-id: io.github.screenshothero.Poc003

runtime: org.gnome.Platform
runtime-version: "50"
sdk: org.gnome.Sdk

command: screenshot-poc

finish-args:
  - --share=ipc
  - --socket=session-bus
  - --talk-name=org.freedesktop.portal.Desktop
  - --filesystem=home
  - --socket=x11
  - --socket=wayland

modules:
  - name: screenshot-poc

    buildsystem: simple

    build-commands:
      - install -Dm755 screenshot-poc /app/bin/screenshot-poc

    sources:
      - type: file
        path: target/release/screenshot-poc
```

---

## Validation Result

Result: **PASS**

Validated behavior:

- The installed Flatpak app launches a GUI window.
- Clicking **Take Screenshot** opens GNOME interactive screenshot UI.
- After selection/capture, the image is rendered inside the app window.
- Aspect ratio is preserved in preview.
- Cancel/error cases show user-friendly messages in the UI.

Non-blocking observation:

- EGL/Mesa warnings may appear in terminal logs on this machine, but the app remains functional and does not block screenshot rendering validation.

---

## Conclusion

POC-003-04 is successful.

The application now completes the full cycle inside Flatpak:

1. capture screenshot through XDG portal,
2. receive and resolve the resulting image URI,
3. render the captured image directly in a GTK4 + Libadwaita window.

This completes the final rendering validation needed before moving to annotation tooling.
