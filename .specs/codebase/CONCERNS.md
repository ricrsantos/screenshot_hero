# Codebase Concerns

**Analyzed:** 2026-06-15

## High Priority

### 1) Planning docs diverged from implementation status

**Evidence:**  
- `.specs/project/ROADMAP.md` previously reported PRD-004/005/006 as not started or planning-only while code already contained `src/persistence/`, `src/export/`, and `src/settings/` implementations.  
- Feature task files remain in `Draft` state while implementation exists.

**Risk:**  
Future planning can duplicate completed work or produce incorrect sequencing/estimates.

**Suggested fix:**  
- Keep roadmap/state updated after every implementation burst.  
- Add a lightweight "docs sync checkpoint" at the end of each feature execution session.

## Medium Priority

### 2) Manual-only validation for UI/desktop flows

**Evidence:**  
- Core workflows depend on GTK display and portal behavior (`src/ui/`, `src/capture/service.rs`, `src/ui/window/imp.rs`).  
- Repository currently relies mainly on `cargo test --lib` for automated confidence.

**Risk:**  
Behavior regressions in UI actions, dialogs, and portal interactions can pass unit tests unnoticed.

**Suggested fix:**  
- Define and version-control a repeatable manual UAT checklist for key flows (capture/open, annotate, save/open project, export, clipboard, preferences).  
- Consider adding smoke checks under virtual display/CI when practical.

### 3) Unsafe GLib source removal path in debounce cleanup

**Evidence:**  
- `src/ui/window/imp.rs` uses `unsafe { glib::ffi::g_source_remove(id.as_raw()) }` in `cancel_clipboard_debounce`.

**Risk:**  
Unsafe FFI usage increases maintenance burden and can become fragile if GLib behavior assumptions change.

**Suggested fix:**  
- Re-evaluate if a safe wrapper API can now support this scenario without panic risk.  
- Keep this behavior documented with clear rationale and constraints (already partially documented inline).

## Low Priority

### 4) README project layout section is stale

**Evidence:**  
- `README.md` "Project layout" currently lists only a subset (`main.rs`, `application.rs`, `ui/window`, `canvas`, `capture`, `models`) and omits now-core modules (`annotations`, `export`, `persistence`, `settings`).

**Risk:**  
New contributors get an outdated mental model of module boundaries.

**Suggested fix:**  
- Refresh README layout section to align with current module set and responsibilities.
