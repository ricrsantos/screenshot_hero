# Code Conventions

## Naming Conventions

**Files:**
- Rust module files use `snake_case` (for example `auto_export.rs`, `tool_palette.rs`).
- GTK subclass implementation files follow `imp.rs` convention (`src/canvas/imp.rs`, `src/ui/window/imp.rs`).

**Types and enums:**
- `PascalCase` (`ProjectManager`, `LaunchOptions`, `ColorSchemePreference`).

**Functions/methods:**
- `snake_case` with action-oriented verbs (`build_project_snapshot`, `copy_canvas_to_clipboard`, `format_load_error`).

**Constants:**
- `SCREAMING_SNAKE_CASE` for module constants (`SCHEMA_ID`, `APP_ICON_NAME`, `APP_VERSION`).

## Code Organization

**Imports:**
- Typical order: std imports first, then crate/external imports; grouped by source.
- Prelude imports are common in GTK modules (`use gtk::prelude::*;`).

**File structure:**
- Public structs/enums and impls at top.
- Private helpers in lower half.
- Tests colocated at file end in `#[cfg(test)] mod tests`.

## Type Safety and Data Modeling

- Domain data uses explicit structs/enums (annotation kinds, persistence schema structs).
- Serialization is derive-based (`Serialize`, `Deserialize`) instead of ad-hoc maps.
- Optional runtime dependencies use `Option` with graceful fallback (`AppSettings::try_new`).

## Error Handling

- Module-specific error enums are preferred (`CaptureError`, `PersistenceError`, `ExportError`).
- UI-facing flows convert errors into user dialogs; background automation logs warnings/errors.
- Cancellation paths are handled explicitly (for example file dialog cancel returns early without error dialogs).

## UI and Runtime Patterns

- User interactions are registered as `gio::SimpleAction` in window construction.
- Keyboard shortcuts are centralized in app startup.
- Async GTK work uses `glib::spawn_future_local`.
- Settings-backed values are read with safe fallbacks when schema is unavailable.

## Comments and Documentation

- Comments are sparse and practical, usually clarifying non-obvious constraints (for example portal staging path handling and source-id removal safety).
- Spec documentation in `.specs/` captures planning and project memory; code comments focus on implementation caveats.
