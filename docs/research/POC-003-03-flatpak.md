# POC-003-03 - Flatpak Screenshot File Access Validation

## Objective

Validate that the Rust application `screenshot-poc`, running inside a Flatpak sandbox, can:

1. Request a screenshot via GNOME/XDG Screenshot Portal.
2. Receive the URI returned by the portal.
3. Convert URI to filesystem path when possible.
4. Access the generated image file from inside the sandbox.
5. Read file metadata and image dimensions safely (without panics).

This POC extends previous work where screenshot capture worked, but file access from the returned URI was inconsistent depending on runtime context.

---

## Initial Goals

- Keep interactive and modal screenshot flow using `ashpd`.
- Remove all `unwrap()` calls from `src/main.rs`.
- Gracefully handle inaccessible paths.
- Ensure Flatpak permissions are correct for portal communication and screenshot use.
- Prove that image access works in Flatpak runtime.

---

## Environment and Scope

- Project: `POC-003-Flatpak`
- Binary: `screenshot-poc`
- Runtime: `org.freedesktop.Platform` `25.08`
- SDK: `org.freedesktop.Sdk`
- Language: Rust 2024 edition
- Main dependencies:
  - `ashpd = "0.12"`
  - `image = "0.25.10"`
  - `tokio = { version = "1", features = ["full"] }`
  - `url = "2.5.8"`

---

## Problem Statement

During earlier validation, the screenshot portal could return URIs under `/run/user/1000/doc/...` when running through Flatpak tooling contexts.  
In that scenario, direct `std::fs::metadata` and `image::open` could fail due to sandbox/file-sharing behavior.

The application needed to be resilient (no panic), and the Flatpak execution path needed to be validated in the correct runtime mode.

---

## Implemented Corrections

## 1) Rust application hardening (`src/main.rs`)

- Replaced all `unwrap()` calls with explicit `match`-based error handling.
- Added URI parsing and URI->path conversion guards.
- Added existence check (`path.exists()`).
- Added graceful early return when file is unavailable.
- Kept success path for metadata and image dimensions extraction.
- Preserved asynchronous entrypoint: `#[tokio::main]`.

Behavior now:

- Always prints:
  - URI
  - Converted filesystem path (or fallback message)
  - File existence
- If file exists:
  - Prints file size in bytes
  - Prints image width and height
- If file does not exist:
  - Prints warning and exits cleanly

## 2) Flatpak manifest adjustments (`io.github.screenshothero.Poc003.yml`)

The final manifest uses:

- `--share=ipc`
- `--socket=session-bus`
- `--talk-name=org.freedesktop.portal.Desktop`
- `--filesystem=home`
- `--socket=x11`
- `--socket=wayland`

This configuration was validated with installed app runtime (`flatpak run`) and successfully allowed image access in this environment.

---

## Commands Used

The following commands were executed during implementation and validation:

```bash
# Local Rust build (dev)
cargo build

# Local Rust build (release binary for Flatpak packaging)
cargo build --release

# Flatpak image build
flatpak-builder --force-clean build-dir io.github.screenshothero.Poc003.yml

# Flatpak run in builder context (diagnostic)
flatpak-builder --run build-dir io.github.screenshothero.Poc003.yml screenshot-poc

# Diagnostic command to inspect doc mount in builder context
flatpak-builder --run build-dir io.github.screenshothero.Poc003.yml \
  sh -lc 'id; ls -ld /run/user /run/user/$(id -u) /run/user/$(id -u)/doc || true'

# Install app to user Flatpak installation (runtime-accurate validation)
flatpak-builder --user --install --force-clean build-dir io.github.screenshothero.Poc003.yml

# Run installed Flatpak app (authoritative validation)
flatpak run io.github.screenshothero.Poc003

# Compare with non-Flatpak run
cargo run
```

---

## Test Results

## Test 1 - `cargo run` (outside Flatpak)

Result: **PASS**

- Screenshot request succeeded.
- URI resolved to user screenshots directory.
- File exists = true.
- File metadata and image dimensions read successfully.

Observed output pattern:

```text
Requesting screenshot...
URI: file:///home/ricardo/Pictures/Screenshots/...
Converted filesystem path: /home/ricardo/Pictures/Screenshots/...
File exists: true
File size (bytes): ...
Image width: ...
Image height: ...
```

## Test 2 - `flatpak-builder --run ...` (builder runtime context)

Result: **FAIL (expected diagnostic behavior)**

- A URI under `/run/user/1000/doc/...` was observed in one run.
- File existence reported false in this context.
- Application did not crash and exited gracefully.

Observed output pattern:

```text
Requesting screenshot...
URI: file:///run/user/1000/doc/...
Converted filesystem path: /run/user/1000/doc/...
File exists: false
Warning: screenshot path is not accessible from current sandbox: ...
```

## Test 3 - `flatpak run io.github.screenshothero.Poc003` (installed app runtime)

Result: **PASS (target acceptance path)**

- Screenshot request succeeded inside Flatpak.
- Returned URI mapped to accessible location in this environment.
- File exists = true.
- Metadata and dimensions read successfully from inside sandboxed app.

Observed output:

```text
Requesting screenshot...
URI: file:///home/ricardo/Pictures/Screenshots/Screenshot%20From%202026-06-05%2011-54-29.png
Converted filesystem path: /home/ricardo/Pictures/Screenshots/Screenshot From 2026-06-05 11-54-29.png
File exists: true
File size (bytes): 21588
Image width: 480
Image height: 270
```

---

## Findings

1. `flatpak-builder --run` can behave differently from installed app runtime regarding file/doc portal access.
2. For this POC acceptance criterion ("app running inside Flatpak can access generated image"), the authoritative validation path is:
   - install app with `flatpak-builder --user --install`
   - run with `flatpak run <app-id>`
3. The hardened Rust code prevents panic and provides clear diagnostics in both success and failure contexts.

---

## Initial Goals vs Achieved Goals

- Interactive/modal screenshot request: **Achieved**
- URI retrieval and printing: **Achieved**
- URI->path conversion and reporting: **Achieved**
- Graceful handling of inaccessible path: **Achieved**
- Metadata + dimensions when accessible: **Achieved**
- No `unwrap()` in runtime path: **Achieved**
- Flatpak runtime image access validation: **Achieved**

---

## Final Source Code

## `src/main.rs`

```rust
use ashpd::desktop::screenshot::Screenshot;
use std::fs;
use url::Url;

#[tokio::main]
async fn main() -> ashpd::Result<()> {
    println!("Requesting screenshot...");

    let response = Screenshot::request()
        .interactive(true)
        .modal(true)
        .send()
        .await?
        .response()?;

    let uri = response.uri();
    println!("URI: {uri}");

    let path = match Url::parse(uri.as_str()) {
        Ok(parsed_url) => match parsed_url.to_file_path() {
            Ok(file_path) => {
                println!("Converted filesystem path: {}", file_path.display());
                Some(file_path)
            }
            Err(_) => {
                eprintln!("Warning: could not convert URI to a filesystem path.");
                println!("Converted filesystem path: <not available>");
                None
            }
        },
        Err(error) => {
            eprintln!("Warning: invalid URI returned by portal: {error}");
            println!("Converted filesystem path: <invalid URI>");
            None
        }
    };

    let Some(path) = path else {
        println!("File exists: false");
        return Ok(());
    };

    let file_exists = path.exists();
    println!("File exists: {file_exists}");

    if !file_exists {
        eprintln!(
            "Warning: screenshot path is not accessible from current sandbox: {}",
            path.display()
        );
        return Ok(());
    }

    match fs::metadata(&path) {
        Ok(metadata) => println!("File size (bytes): {}", metadata.len()),
        Err(error) => {
            eprintln!("Warning: failed to read file metadata: {error}");
            return Ok(());
        }
    }

    match image::open(&path) {
        Ok(img) => {
            println!("Image width: {}", img.width());
            println!("Image height: {}", img.height());
        }
        Err(error) => {
            eprintln!("Warning: failed to open image file: {error}");
        }
    }

    Ok(())
}
```

## `io.github.screenshothero.Poc003.yml`

```yaml
app-id: io.github.screenshothero.Poc003

runtime: org.freedesktop.Platform
runtime-version: "25.08"
sdk: org.freedesktop.Sdk

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

## Conclusion

POC-003-03 is successful.

The Flatpak app can request screenshots and access the generated image file in the validated runtime path (`flatpak run`), including metadata and dimensions extraction.  
The code is now robust against inaccessible paths and portal URI edge cases, preventing runtime crashes while preserving full functionality.
