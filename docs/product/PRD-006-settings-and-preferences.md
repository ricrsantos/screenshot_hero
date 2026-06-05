# PRD-006 - Settings and Preferences

Status: Draft

---

## Objective

Provide user-configurable application behavior.

---

## Functional Requirements

### Appearance

#### FR-001

Support Follow System theme.

#### FR-002

Support Light theme.

#### FR-003

Support Dark theme.

---

### Timestamp

#### FR-004

Enable timestamp.

#### FR-005

Disable timestamp.

#### FR-006

Configure timestamp format.

---

### Auto Save Project

#### FR-007

Enable auto save.

#### FR-008

Disable auto save.

Default:

Enabled

---

### Auto Export

#### FR-009

Enable auto export.

#### FR-010

Disable auto export.

Default:

Disabled

---

### Auto Export Suffix

#### FR-011

User configurable suffix.

Default:

```text
_shero
```

---

### Auto Clipboard Update

#### FR-012

Enable auto clipboard.

#### FR-013

Disable auto clipboard.

Default:

Enabled

---

### Logging

#### FR-014
Enable logging configuration.

#### FR-015
Allow selecting log level.

Supported values:
- Error
- Warn
- Info
- Debug
- Trace

#### FR-016
Persist selected log level between sessions.

#### FR-017
Default log level shall be Info.

---

## Acceptance Criteria

✅ Theme changes work

✅ Timestamp configuration works

✅ Auto save configuration works

✅ Auto export configuration works

✅ Auto clipboard configuration works