# Settings and Preferences Tasks

**Design**: `.specs/features/settings-and-preferences/design.md`  
**Status**: Implemented in codebase (task checklist pending backfill)

---

## Execution Plan

### Phase 1: Foundation (Sequential)

```
T1 → T2
```

T1 (schema) has no dependencies. T2 (AppSettings module) depends on T1 being finalized so types are stable.

### Phase 2: Preferences UI (Sequential)

```
T2 → T3 → T4 → T5 → T6
```

T3 creates the `PreferencesWindow` file; T4–T6 add preference groups to it. Sequential because all four tasks modify the same source file incrementally.

### Phase 3: Runtime Integration (Parallel after T2)

```
T2 ──┬──→ T7 [P]
     └──→ T8 [P]
```

T7 and T8 touch different files (`application.rs` and `window/imp.rs`) and have no shared mutable state. Both depend only on T2.

### Phase 4: Wiring (Sequential)

```
T6 + T7 + T8 → T9
```

T9 adds the `win.show-preferences` action and header button, and is the final integration point. Depends on the UI being complete (T6) and runtime integration done (T7, T8).

### Phase 5: Build System (Independent, after T1)

```
T1 → T10
```

T10 can run at any point after T1 (schema file exists). Does not block any other phase.

### Full Diagram

```
T1 ──→ T2 ──→ T3 ──→ T4 ──→ T5 ──→ T6 ─┐
       │                                  ├──→ T9
       ├──→ T7 [P] ───────────────────────┤
       └──→ T8 [P] ───────────────────────┘
T1 ──→ T10  (independent)
```

---

## Task Breakdown

### T1: Create GSettings schema file

**What**: Create `data/com.screenshot_hero.ScreenshotHero.gschema.xml` with all 8 settings keys, types, defaults, and enum choices  
**Where**: `data/com.screenshot_hero.ScreenshotHero.gschema.xml` (new file)  
**Depends on**: None  
**Reuses**: Schema structure defined in `design.md` → GSettings Schema section  
**Requirement**: SETT-01, SETT-02, SETT-03, SETT-04, SETT-05, SETT-08, SETT-09, SETT-10, SETT-11, SETT-12

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] Schema file exists at `data/com.screenshot_hero.ScreenshotHero.gschema.xml`
- [x] Contains 8 keys: `color-scheme`, `timestamp-enabled`, `timestamp-format`, `auto-save-enabled`, `auto-export-enabled`, `auto-export-suffix`, `auto-clipboard-enabled`, `log-level`
- [x] Enum keys (`color-scheme`, `log-level`) have `<choices>` constraints
- [x] Default values match PRD-006 and ADR-001: color-scheme=follow-system, auto-save=true, auto-export=false, auto-clipboard=true, log-level=info, timestamp=false
- [x] `glib-compile-schemas data/` succeeds without errors
- [x] Gate check passes: `glib-compile-schemas data/ && echo OK`

**Tests**: none  
**Gate**: build

---

### T2: Create AppSettings module

**What**: Create `src/settings/mod.rs` with the `AppSettings` struct wrapping `gio::Settings`, typed getters/setters for all 8 keys, `ColorSchemePreference` enum, and `try_new()` for graceful degradation  
**Where**: `src/settings/mod.rs` (new file); register in `src/lib.rs`  
**Depends on**: T1  
**Reuses**: `gio::Settings` API; `log::LevelFilter` from existing `log` crate  
**Requirement**: SETT-01, SETT-02, SETT-03, SETT-04, SETT-05, SETT-08, SETT-09, SETT-10, SETT-12

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `src/settings/mod.rs` exists and compiles
- [x] `pub mod settings` added to `src/lib.rs`
- [x] `AppSettings::try_new() -> Option<Self>` returns `None` when schema is unavailable (no panic)
- [x] `AppSettings::new() -> Self` convenience wrapper (panics on missing schema — for use after schema is guaranteed installed)
- [x] All 8 typed getters implemented: `color_scheme()`, `timestamp_enabled()`, `timestamp_format()`, `auto_save_enabled()`, `auto_export_enabled()`, `auto_export_suffix()`, `auto_clipboard_enabled()`, `log_level()`
- [x] All 8 typed setters implemented
- [x] `ColorSchemePreference` enum with `FollowSystem | Light | Dark` and conversions: `as_str()`, `from_str()`, `to_adw_color_scheme()` → `libadwaita::ColorScheme`
- [x] `log_level()` maps GSettings string to `log::LevelFilter`; unknown strings default to `LevelFilter::Info`
- [x] `connect_changed<F: Fn(&str) + 'static>(&self, f: F)` forwards to `gio::Settings::connect_changed`
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

> **Dev setup note**: To compile and run locally, schema must be compiled first:  
> `glib-compile-schemas data/`  
> `GSETTINGS_SCHEMA_DIR=data/ cargo run`

---

### T3: Create PreferencesWindow with Appearance group

**What**: Create `src/ui/preferences/mod.rs` with the `PreferencesWindow` struct backed by `adw::PreferencesWindow`, containing the Appearance preference group with a `ComboRow` for color scheme  
**Where**: `src/ui/preferences/mod.rs` (new file); register `pub mod preferences` in `src/ui/mod.rs`  
**Depends on**: T2  
**Reuses**: `AppSettings` from T2; `adw::PreferencesWindow`, `adw::PreferencesGroup`, `adw::ComboRow`  
**Requirement**: SETT-01, SETT-06

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `PreferencesWindow::new(settings: &gio::Settings) -> adw::PreferencesWindow` builds and returns a valid window
- [x] Window title: "Preferences"
- [x] Appearance group present with title "Appearance"
- [x] `ComboRow` for Color Scheme with model items: "Follow System", "Light", "Dark"
- [x] ComboRow initial selection reflects current `color-scheme` GSettings value
- [x] ComboRow `connect_selected_notify` writes updated value back to GSettings
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T4: Add Timestamps group to PreferencesWindow

**What**: Add the Timestamps preference group to `PreferencesWindow` with a `SwitchRow` for timestamp enabled and an `EntryRow` for timestamp format  
**Where**: `src/ui/preferences/mod.rs` (modify)  
**Depends on**: T3  
**Reuses**: `adw::SwitchRow` (v1_4), `adw::EntryRow` (v1_4); GSettings bind pattern from T3  
**Requirement**: SETT-08, SETT-09

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] Timestamps group visible in PreferencesWindow with title "Timestamps"
- [x] `SwitchRow` "Auto-Add Timestamp" bound to `timestamp-enabled` GSettings key via `gio::Settings::bind()`
- [x] `EntryRow` "Timestamp Format" bound to `timestamp-format` GSettings key via `gio::Settings::bind()`
- [x] `EntryRow` is insensitive (grayed out) when `timestamp-enabled` is false; sensitive when true
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T5: Add Automation group to PreferencesWindow

**What**: Add the Automation preference group with four controls: Auto Save, Auto Export, Export Suffix, Auto Clipboard  
**Where**: `src/ui/preferences/mod.rs` (modify)  
**Depends on**: T4  
**Reuses**: `adw::SwitchRow`, `adw::EntryRow`; GSettings bind pattern  
**Requirement**: SETT-02, SETT-03, SETT-04, SETT-05

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] Automation group visible in PreferencesWindow with title "Automation"
- [x] `SwitchRow` "Auto Save" bound to `auto-save-enabled`
- [x] `SwitchRow` "Auto Export" bound to `auto-export-enabled`
- [x] `EntryRow` "Export Suffix" bound to `auto-export-suffix`; insensitive when `auto-export-enabled` is false
- [x] `SwitchRow` "Auto Clipboard" bound to `auto-clipboard-enabled`
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T6: Add Developer (Logging) group to PreferencesWindow

**What**: Add the Developer preference group with a `ComboRow` for log level selection  
**Where**: `src/ui/preferences/mod.rs` (modify)  
**Depends on**: T5  
**Reuses**: `adw::ComboRow`; GSettings bind pattern from T3  
**Requirement**: SETT-10, SETT-11

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] Developer group visible in PreferencesWindow with title "Developer"
- [x] `ComboRow` "Log Level" with model items: "Error", "Warn", "Info", "Debug", "Trace"
- [x] ComboRow initial selection reflects current `log-level` GSettings value
- [x] ComboRow `connect_selected_notify` writes updated string back to GSettings
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T7: Application startup — env_logger init + theme + log level integration [P]

**What**: Initialize `env_logger` in `Application::startup()`, read initial `color-scheme` and `log-level` from GSettings, apply to `StyleManager` and `log::set_max_level()`, and connect `settings.connect_changed` for live updates  
**Where**: `src/application.rs` (modify `imp::Application::startup()`)  
**Depends on**: T2  
**Reuses**: `AppSettings` from T2; `libadwaita::StyleManager::default()`; `log::set_max_level()`; `env_logger::Builder`  
**Requirement**: SETT-01, SETT-10, SETT-11, SETT-12

> Note (2026-06-15): Two criteria remain open because current implementation intentionally differs: logger init currently uses `try_init()` with `Info` default, and startup does not early-return when schema is unavailable.

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [ ] `env_logger::Builder::new().filter_level(log::LevelFilter::Trace).init()` called once in `startup()` (ceiling at Trace; effective level controlled by `log::set_max_level`)
- [ ] `AppSettings::try_new()` called; if `None`, logs warning and returns (graceful degradation — hardcoded defaults remain)
- [x] Initial `log_level()` read from settings and applied via `log::set_max_level()`
- [x] Initial `color_scheme()` read from settings and applied via `StyleManager::default().set_color_scheme()`
- [x] `settings.connect_changed` handler: on `"log-level"` change → `log::set_max_level(settings.log_level())`
- [x] `settings.connect_changed` handler: on `"color-scheme"` change → `StyleManager::default().set_color_scheme(settings.color_scheme().to_adw_color_scheme())`
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T8: MainWindow — replace Cell<bool> flags with GSettings reads [P]

**What**: Remove `auto_clipboard_enabled`, `auto_export_enabled`, `auto_export_suffix` fields from `MainWindow` struct; add `settings: OnceCell<gio::Settings>`; update `on_annotation_changed` to read directly from GSettings; gate `maybe_auto_save` behind `auto-save-enabled`  
**Where**: `src/ui/window/imp.rs` (modify)  
**Depends on**: T2  
**Reuses**: `gio::Settings` (already imported via `gtk::gio`); existing `on_annotation_changed` callback structure  
**Requirement**: SETT-02, SETT-03, SETT-04, SETT-05

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `MainWindow` struct no longer has `auto_clipboard_enabled: Cell<bool>`, `auto_export_enabled: Cell<bool>`, `auto_export_suffix: RefCell<String>` fields
- [x] `MainWindow` struct has `settings: OnceCell<gio::Settings>` field
- [x] `Default::default()` for `MainWindow` initializes `settings` to `OnceCell::new()` (value set in `constructed()`)
- [x] In `constructed()`: `AppSettings::try_new()` — if Some, store `settings` in `OnceCell`; if None, log warning
- [x] In `on_annotation_changed`: `auto-clipboard-enabled` read from `settings.boolean("auto-clipboard-enabled")` (or fallback `true` when settings unavailable)
- [x] In `on_annotation_changed`: `auto-export-enabled` read from `settings.boolean("auto-export-enabled")` (fallback `false`)
- [x] In `on_annotation_changed`: `auto-export-suffix` read from `settings.string("auto-export-suffix")` (fallback `"_shero"`)
- [x] In `on_annotation_changed`: `maybe_auto_save` gated behind `settings.boolean("auto-save-enabled")` (fallback `true`)
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T9: Add `win.show-preferences` GAction and header button

**What**: Add `win.show-preferences` `gio::SimpleAction` to `MainWindow`, wire it to open `PreferencesWindow`, and add a Settings button to the header bar  
**Where**: `src/ui/window/imp.rs` (modify `constructed()`)  
**Depends on**: T6, T7, T8  
**Reuses**: `gio::SimpleAction` pattern from existing actions; `PreferencesWindow::new()` from T3–T6  
**Requirement**: SETT-06

> Note (2026-06-15): Preferences is exposed through the file menu action and accelerator; a dedicated header gear button is not currently present.

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] `gio::SimpleAction::new("show-preferences", None)` created and added to window action group
- [x] Action activate handler: creates `PreferencesWindow::new(&settings)` and calls `window.present()` on it; sets `PreferencesWindow` transient for main window
- [ ] Header bar has a Settings button (gear icon via `gtk::Button` with `icon-name = "preferences-system-symbolic"`) with `action-name = "win.show-preferences"`
- [x] Accel `<Control>comma` registered for `win.show-preferences` in `Application::startup()`
- [x] Gate check passes: `cargo build`

**Tests**: none  
**Gate**: build

---

### T10: Flatpak manifest — schema compile step + dev setup docs

**What**: Update `build/com.screenshot_hero.ScreenshotHero.yml` to install the GSettings schema and compile it during the Flatpak build; add developer setup instructions for local development  
**Where**: `build/com.screenshot_hero.ScreenshotHero.yml` (modify); `README.md` (modify, dev setup section)  
**Depends on**: T1  
**Reuses**: Existing Flatpak manifest structure  
**Requirement**: SETT-07, SETT-12

> Note (2026-06-15): Flatpak build/install manual validation remains pending.

**Tools**:

- MCP: NONE
- Skill: NONE

**Done when**:

- [x] Flatpak manifest `build-commands` includes:
  - `install -Dm0644 data/com.screenshot_hero.ScreenshotHero.gschema.xml ${FLATPAK_DEST}/share/glib-2.0/schemas/com.screenshot_hero.ScreenshotHero.gschema.xml`
  - `glib-compile-schemas ${FLATPAK_DEST}/share/glib-2.0/schemas/`
- [x] `README.md` dev setup section includes:
  ```bash
  glib-compile-schemas data/
  GSETTINGS_SCHEMA_DIR=data/ cargo run
  ```
- [ ] `flatpak-builder --install --user build-dir build/com.screenshot_hero.ScreenshotHero.yml --force-clean` completes without errors (manual validation)
- [x] Gate check passes: `cargo build` (no compilation dependency on manifest)

**Tests**: none  
**Gate**: build

---

## Parallel Execution Map

```
Phase 1 (Sequential):
  T1 ──→ T2

Phase 2 (Sequential UI build):
  T2 ──→ T3 ──→ T4 ──→ T5 ──→ T6

Phase 3 (Parallel integration, starts after T2):
  T2 complete, then:
    ├── T7 [P]  (application.rs — theme + logging)
    └── T8 [P]  (window/imp.rs — flag migration)

Phase 4 (Sequential wiring, after T6 + T7 + T8):
  T6 + T7 + T8 complete, then:
    T9

Phase 5 (Independent, after T1):
  T1 ──→ T10  (can run in parallel with Phase 2–4)
```

**Effective critical path**: T1 → T2 → T3 → T4 → T5 → T6 → T9 (7 tasks sequential)  
**Parallel opportunity**: T7 + T8 run alongside T3–T6

---

## Task Granularity Check

| Task | Scope | Status |
|---|---|---|
| T1: GSettings schema | 1 XML file | ✅ Granular |
| T2: AppSettings module | 1 Rust file, 1 struct + 1 enum | ✅ Granular |
| T3: PreferencesWindow + Appearance group | 1 file (new), 1 group, 1 control | ✅ Granular |
| T4: Timestamps group | 1 group, 2 controls added to existing file | ✅ Granular |
| T5: Automation group | 1 group, 4 controls added to existing file | ✅ Granular |
| T6: Developer/Logging group | 1 group, 1 control added to existing file | ✅ Granular |
| T7: Startup integration | 2 concerns (env_logger + theme/log) in 1 function | ✅ Cohesive (both are startup-time, same file) |
| T8: Cell<bool> migration | 1 file, 1 struct, 1 callback | ✅ Granular |
| T9: show-preferences action | 1 action + 1 button + 1 accel | ✅ Granular |
| T10: Build + docs | 1 manifest + 1 README section | ✅ Granular |

---

## Diagram-Definition Cross-Check

| Task | Depends On (task body) | Diagram Shows | Status |
|---|---|---|---|
| T1 | None | No incoming arrows | ✅ Match |
| T2 | T1 | T1 → T2 | ✅ Match |
| T3 | T2 | T2 → T3 | ✅ Match |
| T4 | T3 | T3 → T4 | ✅ Match |
| T5 | T4 | T4 → T5 | ✅ Match |
| T6 | T5 | T5 → T6 | ✅ Match |
| T7 [P] | T2 | T2 → T7 | ✅ Match |
| T8 [P] | T2 | T2 → T8 | ✅ Match |
| T9 | T6, T7, T8 | T6 + T7 + T8 → T9 | ✅ Match |
| T10 | T1 | T1 → T10 | ✅ Match |

---

## Test Co-location Validation

| Task | Code Layer Created/Modified | Matrix Requires | Task Says | Status |
|---|---|---|---|---|
| T1: GSettings schema | XML file (not a code layer) | none | none | ✅ OK |
| T2: AppSettings module | New module — closest match: data model or application entry | none | none | ✅ OK |
| T3: PreferencesWindow skeleton | `src/ui/` | none | none | ✅ OK |
| T4: Timestamps group | `src/ui/` | none | none | ✅ OK |
| T5: Automation group | `src/ui/` | none | none | ✅ OK |
| T6: Logging group | `src/ui/` | none | none | ✅ OK |
| T7: application.rs startup | `src/application.rs` → Application entry | none | none | ✅ OK |
| T8: window/imp.rs migration | `src/ui/` | none | none | ✅ OK |
| T9: show-preferences action | `src/ui/` | none | none | ✅ OK |
| T10: Flatpak + docs | `build/` | none | none | ✅ OK |

**All tasks pass test co-location validation.** ✅

> **Note on T2 (AppSettings)**: The module is a thin wrapper around `gio::Settings`. Testing it requires the schema to be compiled and reachable via `GSETTINGS_SCHEMA_DIR`. The project currently has no precedent for GSettings integration tests. Marking as `none` is consistent with `src/ui/` and `src/application.rs` layers in TESTING.md. If a test harness for GSettings is added later, revisit.
