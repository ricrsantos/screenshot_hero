# PRD-001 - Screenshot Capture and Loading

Status: Accepted

---

## Objective

Allow users to capture screenshots using the GNOME Screenshot Portal and immediately load the resulting image into Screenshot Hero.

---

## User Story

As a user,

I want to capture a screenshot,

so that I can immediately annotate it.

---

## Functional Requirements

### FR-001

The application shall provide a "New Screenshot" action.

---

### FR-002

The application shall invoke the XDG Screenshot Portal.

---

### FR-003

The screenshot request shall be interactive.

---

### FR-004

The screenshot request shall be modal.

---

### FR-005

The application shall receive the screenshot URI returned by the portal.

---

### FR-006

The application shall load the image into the canvas.

---

### FR-007

The application shall display the captured image.

---

### FR-008

The application shall support loading existing PNG files.

---

### FR-009

The application shall support loading existing JPEG files.

---

### FR-010

The application shall display an error when image loading fails.

---

## Non Functional Requirements

- Flatpak compatible
- Offline operation
- Linux native

---

## Acceptance Criteria

✅ User can capture a screenshot

✅ Screenshot appears in the canvas

✅ Existing PNG can be opened

✅ Existing JPEG can be opened

✅ Errors are handled gracefully