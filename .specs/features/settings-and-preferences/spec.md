# Settings and Preferences Specification

**PRD:** `docs/product/PRD-006-settings-and-preferences.md`  
**Status:** Draft

---

## Problem Statement

The application hardcodes behavior such as auto-clipboard, auto-export, color scheme, and logging. Users have no way to personalize these behaviors between sessions. All runtime flags (e.g. `auto_clipboard_enabled`, `auto_export_enabled`) are in-memory defaults that reset on every launch, making the product feel unfinished.

## Goals

- [ ] User-configurable preferences persisted between sessions via GSettings
- [ ] Preferences accessible from a dedicated UI window (GNOME HIG-compliant)
- [ ] Appearance, automation, timestamps, and logging are all configurable
- [ ] Migration from in-memory Cell<bool> flags to GSettings-backed live bindings

## Out of Scope

| Feature | Reason |
|---------|--------|
| Recent files list | Requires separate file history mechanism; deferred idea logged in STATE.md |
| Per-project settings | Settings are application-wide only |
| Import/export of settings | Not in PRD-006 |
| Settings reset to defaults button | Nice-to-have, excluded for v1 |

---

## User Stories

### P1: Color Scheme Selection ⭐ MVP

**User Story**: As a user, I want to choose between Follow System, Light, and Dark themes so that the application matches my desktop preference.

**Why P1**: Core appearance setting; missing this makes the app feel unpolished on dark-themed desktops.

**Acceptance Criteria**:

1. WHEN user opens Preferences THEN system SHALL display a color scheme selector with three options: Follow System, Light, Dark
2. WHEN user selects "Follow System" THEN system SHALL delegate color scheme to `adw::StyleManager::set_color_scheme(ColorScheme::Default)`
3. WHEN user selects "Light" THEN system SHALL apply `adw::StyleManager::set_color_scheme(ColorScheme::ForceLight)`
4. WHEN user selects "Dark" THEN system SHALL apply `adw::StyleManager::set_color_scheme(ColorScheme::ForceDark)`
5. WHEN application restarts THEN system SHALL restore the previously selected color scheme immediately on startup
6. WHEN color scheme changes THEN system SHALL apply it immediately without requiring an app restart

**Requirement IDs**: SETT-01

**Independent Test**: Open Preferences → select Dark → close app → reopen → verify dark theme is active

---

### P1: Auto-Clipboard Toggle ⭐ MVP

**User Story**: As a user, I want to enable or disable automatic clipboard updates so that I control when my clipboard is overwritten.

**Why P1**: Auto-clipboard is currently hardcoded to enabled; some workflows require disabling it.

**Acceptance Criteria**:

1. WHEN user opens Preferences THEN system SHALL display an Auto Clipboard switch (default: ON)
2. WHEN user toggles Auto Clipboard OFF THEN system SHALL stop debounced clipboard updates on annotation changes
3. WHEN user toggles Auto Clipboard ON THEN system SHALL resume debounced clipboard updates on annotation changes
4. WHEN application restarts THEN system SHALL restore the previously saved auto-clipboard setting

**Requirement IDs**: SETT-02

**Independent Test**: Toggle off → annotate → verify clipboard is NOT updated

---

### P1: Auto-Export Toggle + Suffix Configuration ⭐ MVP

**User Story**: As a user, I want to enable auto-export and configure its suffix so that annotated screenshots are automatically saved beside the original with a recognizable name.

**Why P1**: Auto-export behavior is already implemented but currently locked behind an in-memory default (disabled). Preferences must wire it properly.

**Acceptance Criteria**:

1. WHEN user opens Preferences THEN system SHALL display an Auto Export switch (default: OFF)
2. WHEN user enables Auto Export THEN system SHALL trigger export on every annotation change
3. WHEN user disables Auto Export THEN system SHALL stop auto-export
4. WHEN user opens Preferences THEN system SHALL display an Auto Export Suffix text field (default: `_shero`)
5. WHEN user changes the suffix THEN system SHALL use the new suffix for all subsequent auto-exports
6. WHEN application restarts THEN system SHALL restore auto-export enabled state and suffix

**Requirement IDs**: SETT-03, SETT-04

**Independent Test**: Enable auto-export → annotate → verify `original_shero.png` appears beside original

---

### P1: Auto-Save Toggle ⭐ MVP

**User Story**: As a user, I want to control whether the project auto-saves after each annotation change so that I can decide when to commit changes.

**Why P1**: Auto-save is currently hardcoded in `ProjectManager`; PRD-006 requires it to be user-configurable.

**Acceptance Criteria**:

1. WHEN user opens Preferences THEN system SHALL display an Auto Save switch (default: ON)
2. WHEN user disables Auto Save THEN system SHALL stop `ProjectManager::maybe_auto_save()` from firing
3. WHEN user enables Auto Save THEN system SHALL resume auto-saving on annotation changes
4. WHEN application restarts THEN system SHALL restore the previously saved auto-save setting

**Requirement IDs**: SETT-05

**Independent Test**: Disable auto-save → annotate → reopen project file → verify annotations are NOT persisted automatically

---

### P1: Preferences Window Accessible from UI ⭐ MVP

**User Story**: As a user, I want a dedicated Preferences window accessible from the main window so that I can view and change all settings in one place.

**Why P1**: Without UI entry point, all preference work has no user-facing value.

**Acceptance Criteria**:

1. WHEN application is running THEN system SHALL display a Settings button/menu item in the header bar
2. WHEN user activates `win.show-preferences` (or clicks Settings) THEN system SHALL open an `adw::PreferencesWindow`
3. WHEN PreferencesWindow is open THEN system SHALL display four preference groups: Appearance, Timestamps, Automation, Developer
4. WHEN user closes PreferencesWindow THEN system SHALL return focus to main window

**Requirement IDs**: SETT-06

**Independent Test**: Click Settings → PreferencesWindow opens → all four groups visible

---

### P2: Timestamp Auto-Add Toggle + Format

**User Story**: As a user, I want to automatically add a timestamp annotation when a screenshot is captured, with a configurable format, so that all my screenshots are date-stamped without manual effort.

**Why P2**: Timestamp annotation type already exists; this adds automation on top. Useful but not blocking core settings persistence.

**Acceptance Criteria**:

1. WHEN user opens Preferences THEN system SHALL display a Timestamp toggle (default: OFF)
2. WHEN user enables Timestamp THEN system SHALL automatically add a Timestamp annotation on every new screenshot capture
3. WHEN user disables Timestamp THEN system SHALL stop auto-adding timestamp annotations
4. WHEN user opens Preferences THEN system SHALL display a Timestamp Format text field (default: `%Y-%m-%d %H:%M:%S`)
5. WHEN user changes the timestamp format THEN system SHALL use the new format for subsequent auto-added timestamps
6. WHEN application restarts THEN system SHALL restore timestamp enabled state and format

**Requirement IDs**: SETT-08, SETT-09

**Note**: Requires Canvas to expose a method for programmatic Timestamp annotation insertion. If the Canvas API does not support this at implementation time, auto-add behavior can be deferred; the settings key and UI must still be implemented.

**Independent Test**: Enable Timestamp → capture screenshot → verify timestamp annotation appears automatically

---

### P2: Log Level Selector

**User Story**: As a developer/power-user, I want to configure the application log level so that I can diagnose issues without enabling verbose logging for all users.

**Why P2**: Logging is a developer-facing feature. Default Info level is correct for most users; selector is needed for troubleshooting.

**Acceptance Criteria**:

1. WHEN user opens Preferences → Developer section THEN system SHALL display a Log Level selector with options: Error, Warn, Info, Debug, Trace
2. WHEN user selects a log level THEN system SHALL call `log::set_max_level()` immediately (no restart required)
3. WHEN application starts THEN system SHALL initialize `env_logger` and apply the persisted log level before any logging occurs
4. WHEN application restarts THEN system SHALL restore the selected log level
5. WHEN no preference has been set THEN system SHALL default to Info

**Requirement IDs**: SETT-10, SETT-11, SETT-12

**Independent Test**: Set Debug → verify debug-level messages appear in stderr → restart → verify level is still Debug

---

## Edge Cases

- WHEN GSettings schema is not installed THEN system SHALL log an error and fall back to all hardcoded defaults (graceful degradation)
- WHEN auto-export suffix is empty string THEN system SHALL use `_shero` as fallback to prevent overwriting the original
- WHEN timestamp format string is empty THEN system SHALL use `%Y-%m-%d %H:%M:%S` as fallback
- WHEN timestamp is enabled but no image is loaded THEN system SHALL do nothing (no crash)
- WHEN color scheme changed signal fires but StyleManager is unavailable THEN system SHALL log a warning and continue

---

## Requirement Traceability

| Requirement ID | Story | PRD FRs | Phase | Status |
|---|---|---|---|---|
| SETT-01 | P1: Color Scheme | FR-001, FR-002, FR-003 | Design | Pending |
| SETT-02 | P1: Auto-Clipboard | FR-012, FR-013 | Design | Pending |
| SETT-03 | P1: Auto-Export Toggle | FR-009, FR-010 | Design | Pending |
| SETT-04 | P1: Auto-Export Suffix | FR-011 | Design | Pending |
| SETT-05 | P1: Auto-Save | FR-007, FR-008 | Design | Pending |
| SETT-06 | P1: Preferences Window | — | Design | Pending |
| SETT-08 | P2: Timestamp Toggle | FR-004, FR-005 | Design | Pending |
| SETT-09 | P2: Timestamp Format | FR-006 | Design | Pending |
| SETT-10 | P2: Log Level UI | FR-014, FR-015 | Design | Pending |
| SETT-11 | P2: Log Level Runtime | FR-015 | Design | Pending |
| SETT-12 | P2: Log Level Persistence | FR-016, FR-017 | Design | Pending |

**Coverage:** 11 requirements, 0 mapped to tasks, 11 unmapped ⚠️

---

## Success Criteria

- [ ] Preferences window opens from header bar and displays all four groups
- [ ] Color scheme change takes effect immediately and survives app restart
- [ ] Auto-clipboard, auto-export, auto-save toggles control runtime behavior and persist between sessions
- [ ] Auto-export suffix is configurable and reflected in exported filenames
- [ ] Log level is applied on startup and can be changed at runtime
- [ ] All in-memory `Cell<bool>` flags in `MainWindow` are replaced by GSettings-backed reads
