# Behavior Settings Extend - Tasks

**Design:** `.specs/features/behavior-settings-extend/design.md`  
**Status:** Implemented

---

## Task Breakdown

### T1 - Extend GSettings schema
- [x] Add 7 new keys for capture behavior, temporary mode, exit-after-paste, and window policy.
- [x] Keep defaults aligned with requirements.
- [x] Validate schema compilation.

**Gate:** `glib-compile-schemas data/`

---

### T2 - Extend typed settings wrapper
- [x] Add getters/setters for all new keys in `AppSettings`.
- [x] Add effective temporary-mode resolver with auto-expiration reset.
- [x] Keep backward-compatible defaults/fallback behavior when schema is unavailable.

**Gate:** `cargo build`

---

### T3 - Extend Preferences UI
- [x] Add new "Capture Behavior" group.
- [x] Add permanent disable switch.
- [x] Add temporary disable switch.
- [x] Add minutes/seconds duration inputs (default 1m0s).
- [x] Add open-new-window switch.
- [x] Add exit-after-paste switch.

**Gate:** `cargo build`

---

### T4 - Wire `--capture` runtime behavior
- [x] Apply effective disable logic in capture-first startup flow.
- [x] Ensure capture portal still opens before app exits in disable mode.
- [x] Reuse existing window when option "open new window on capture" is disabled.
- [x] Keep explicit new-window behavior when option is enabled.

**Gate:** `cargo build`

---

### T5 - Wire Exit After Paste runtime behavior
- [x] In `--capture` edit session, copy image to clipboard automatically.
- [x] Monitor clipboard changes and quit on change.
- [x] Restrict behavior to startup capture flow with editor open.

**Gate:** `cargo build`

---

### T6 - Verification
- [x] `cargo test --lib`
- [x] `cargo build`
- [ ] Manual UI validation (`cargo run`, `cargo run -- --capture`) for all new toggles

---

## Executed Commands

- `glib-compile-schemas data/ && cargo build`
- `cargo test --lib`
