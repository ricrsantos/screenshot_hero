# Testing Strategy - Screenshot Hero

**Stack:** Rust + GTK4 + Libadwaita + Flatpak

---

## Test Coverage Matrix

| Code Layer | Location | Test Type | Rationale |
|---|---|---|---|
| Data models | `src/models/` | unit | Pure structs, no GTK; fully testable |
| Capture service | `src/capture/service.rs` | unit | Business logic testable without portal (mock/stub) |
| File loader | `src/capture/loader.rs` | unit | File parsing logic testable with fixture files |
| UI components | `src/ui/`, `src/canvas/` | none | GTK4 widgets require a display server; manual only |
| Application entry | `src/main.rs`, `src/application.rs` | none | GApplication init requires display; manual only |
| Flatpak build | `build/` | none | Must be validated manually via `flatpak run` |

---

## Gate Check Commands

| Level | Command | When to Use |
|---|---|---|
| **quick** | `cargo test --lib` | After tasks that add/modify unit-tested code |
| **build** | `cargo build` | After tasks that add UI, wiring, or config (no tests) |
| **full** | `cargo build && cargo test --lib` | Last task in a phase or integration point |

---

## Parallelism Assessment

| Test Type | Parallel-Safe | Notes |
|---|---|---|
| unit (`cargo test --lib`) | Yes | Pure Rust, no shared state, no display required |
| build (`cargo build`) | Yes | Read-only after source changes are committed |
| none | N/A | No test runner involved |

---

## Manual Validation Protocol

For UI and Flatpak testing, use the following steps:

**Native (development):**
```bash
cargo run
```

**Flatpak (authoritative — per POC-003-03):**
```bash
flatpak-builder --install --user build-dir build/com.example.ScreenshotHero.yml --force-clean
flatpak run com.example.ScreenshotHero
```

> ⚠️ Flatpak runner runtime differs from flatpak-builder runtime. Always validate with `flatpak run`, not just `flatpak-builder --build-only`.

---

## Test File Conventions

- Test modules: inline `#[cfg(test)]` module at bottom of source file
- Test fixtures: `tests/fixtures/` for sample images and project files
- Naming: `test_<behavior>_<condition>` (e.g., `test_load_png_valid_file`)

---

## Notes

- GTK4 UI testing is deferred by design — requires Wayland/X11 compositor
- If CI is added later, use Xvfb or a headless Wayland compositor for UI smoke tests
- `ashpd` portal calls cannot be unit-tested without a running portal; test only the business logic around them
