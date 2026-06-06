# Canvas and Navigation — Specification

**PRD:** `docs/product/PRD-002-canvas-and-navigation.md`  
**ADR:** `docs/architecture/ADR-003-canvas-rendering.md`  
**Status:** Approved

---

## Problem Statement

After a screenshot is loaded (PRD-001), the user has no way to navigate it. Large screenshots (2560×1440, 4K, ultrawide) appear at their natural size with no zoom or pan — making it impossible to examine fine details or see the full image when it exceeds the viewport. PRD-002 adds the navigation layer that makes annotation work practical for any image size.

---

## Goals

- [ ] User can zoom in and out to examine details or fit the full image in view
- [ ] User can pan to navigate off-screen areas of large images at high zoom levels
- [ ] User always sees the current zoom percentage in the header bar
- [ ] Image rendering quality is maintained (no pixelation artifacts from bad interpolation)

---

## Out of Scope

| Feature | Reason |
|---------|--------|
| Annotation tools | PRD-003 |
| Touch/pinch zoom | Linux desktop; mouse and keyboard are the primary inputs |
| Zoom to selection | Requires annotation selection (PRD-003) |
| Mini-map / overview panel | Nice-to-have, not in PRD-002 |
| Smooth animated zoom transitions | Snap zoom is acceptable for v1 |
| Horizontal-only scroll pan | Middle-mouse drag covers all directions |

---

## User Stories

### P1: Zoom Controls ⭐ MVP

**User Story:** As a user, I want to zoom in, zoom out, fit the image to the window, and reset to 100% using buttons and keyboard shortcuts so that I can examine details or see the full image without losing my orientation.

**Why P1:** Zoom is the foundational navigation gesture. Without it, users working on large screenshots cannot see fine details or the full image at once — the annotation workflow is blocked.

**Acceptance Criteria:**

1. WHEN user activates zoom-in THEN system SHALL multiply zoom by a fixed step, clamped to 800% maximum
2. WHEN user activates zoom-out THEN system SHALL divide zoom by a fixed step, clamped to 10% minimum
3. WHEN user activates fit-to-window THEN system SHALL scale the image so it fits entirely within the canvas area and center it
4. WHEN user activates zoom-100 THEN system SHALL reset zoom to exactly 100% and center the image in the canvas area
5. WHEN user activates zoom-in at 800% THEN system SHALL remain at 800% (no change, no error)
6. WHEN user activates zoom-out at 10% THEN system SHALL remain at 10% (no change, no error)
7. WHEN no image is loaded THEN zoom actions SHALL be enabled but SHALL produce no visible effect

**Zoom actions accessible via:**
- Header bar buttons: `+`, `-`, `[]` (fit), `1:1` (100%)
- Keyboard shortcuts: `Ctrl++` / `Ctrl+=` (zoom-in), `Ctrl+-` (zoom-out), `Ctrl+Shift+F` (fit), `Ctrl+0` (100%)

**Independent Test:** Load an image. Press `Ctrl++` five times — canvas zooms in, zoom level label increments each time. Press `Ctrl+Shift+F` — image fits the window. Press `Ctrl+0` — zoom returns to 100%.

---

### P1: Scroll-Wheel Zoom ⭐ MVP

**User Story:** As a user, I want to zoom in and out by scrolling the mouse wheel over the canvas so that I can navigate without reaching for keyboard shortcuts.

**Why P1:** Scroll-wheel zoom is the primary navigation gesture in every image and map tool. Without it, navigation feels unnatural and slow.

**Acceptance Criteria:**

1. WHEN user scrolls up (wheel up) over the canvas THEN system SHALL zoom in toward the cursor position
2. WHEN user scrolls down (wheel down) over the canvas THEN system SHALL zoom out away from the cursor position
3. WHEN scroll zoom occurs THEN the image point under the cursor SHALL remain at the same canvas position before and after the zoom (zoom-to-cursor behavior)
4. WHEN scroll zoom would exceed 800% THEN system SHALL clamp to 800%
5. WHEN scroll zoom would go below 10% THEN system SHALL clamp to 10%
6. WHEN no image is loaded AND user scrolls THEN system SHALL do nothing

**Independent Test:** Load an image. Hover cursor over a distinctive pixel (e.g., a corner element). Scroll up — image zooms in and that pixel stays under the cursor. Scroll down — image zooms out with the same anchor.

---

### P1: Middle-Mouse Pan ⭐ MVP

**User Story:** As a user, I want to pan the canvas by pressing and dragging with the middle mouse button so that I can reach areas of the image that are off-screen at the current zoom level.

**Why P1:** At zoom levels above fit-to-window, portions of the image extend beyond the viewport. Without pan, those areas are inaccessible and the annotation workflow is blocked.

**Acceptance Criteria:**

1. WHEN user presses middle mouse button over the canvas THEN system SHALL enter pan mode
2. WHEN user drags with middle mouse button held THEN system SHALL translate the canvas by the drag delta (image follows the cursor)
3. WHEN user releases middle mouse button THEN system SHALL exit pan mode and freeze the canvas at the final position
4. WHEN pan mode is active THEN the cursor SHALL change to a "grabbing" cursor
5. WHEN pan mode is inactive THEN the cursor SHALL return to the default pointer
6. WHEN panning moves the image partially or fully off-screen THEN system SHALL allow it (no boundary clamping — user can recover with fit-to-window)

**Independent Test:** Load a large image. Zoom to 200%. Middle-click-drag — image pans in the drag direction. Release — image stays at the new position.

---

### P1: Zoom Level Indicator ⭐ MVP

**User Story:** As a user, I want to see the current zoom percentage displayed in the header bar so that I always know my navigation state.

**Why P1:** Without a zoom indicator, users lose orientation — they cannot tell if they are at 50% or 200%, making it impossible to deliberately navigate.

**Acceptance Criteria:**

1. WHEN the application starts THEN system SHALL display "100%" in the header bar zoom label
2. WHEN zoom changes via any input (button, keyboard, scroll) THEN the label SHALL update immediately to the new rounded percentage
3. WHEN zoom is exactly 100% THEN label SHALL show "100%"
4. WHEN zoom is exactly 50% THEN label SHALL show "50%"
5. WHEN zoom is exactly 200% THEN label SHALL show "200%"
6. WHEN fit-to-window results in a fractional zoom (e.g., 0.6667) THEN label SHALL display the rounded integer percentage (e.g., "67%")

**Independent Test:** Observe the header zoom label. Activate zoom-in, zoom-out, fit, and 100% actions — label updates correctly after each action. Scroll wheel — label updates per notch.

---

### P2: Image Quality During Zoom

**User Story:** As a user, I want the image to remain visually sharp at any zoom level so that I can accurately annotate fine details when zoomed in.

**Why P2:** Poor interpolation (nearest-neighbor) causes blocky artifacts when zoomed in — making it hard to see and annotate small elements. Important for annotation quality but doesn't block core navigation.

**Acceptance Criteria:**

1. WHEN the image is displayed at any zoom level THEN system SHALL use bilinear (or better) interpolation — no visible pixel-grid artifacts at zoom levels below 800%
2. WHEN the image is zoomed out THEN system SHALL use bilinear downsampling — no jagged aliasing on edges

**Independent Test:** Load a screenshot with text. Zoom to 300% — text letters remain smooth and readable (no blocky pixelation).

---

## Edge Cases

- WHEN canvas widget is resized (user resizes the window) THEN system SHALL NOT auto-recompute fit — fit-to-window is a one-time explicit action, not a continuous binding
- WHEN zoom-to-cursor is computed AND the pointer is outside the canvas widget bounds THEN system SHALL use the canvas center as the zoom anchor
- WHEN image is panned fully off-screen THEN system SHALL allow it — user recovers via fit-to-window or zoom-100
- WHEN `fit_to_window` is called before canvas has been allocated (widget size = 0) THEN system SHALL do nothing (skip computation)
- WHEN zoom-in or zoom-out is called with no image THEN system SHALL change the zoom value (state updated) but the canvas appears unchanged until an image is loaded

---

## Requirement Traceability

| Requirement ID | PRD Ref | Story | Status |
|---|---|---|---|
| CNAV-01 | FR-002 | P1: Zoom Controls | Pending |
| CNAV-02 | FR-003 | P1: Zoom Controls | Pending |
| CNAV-03 | FR-004 | P1: Zoom Controls | Pending |
| CNAV-04 | FR-005 | P1: Zoom Controls | Pending |
| CNAV-05 | FR-006 | P1: Middle-Mouse Pan | Pending |
| CNAV-06 | FR-007 | P1: Zoom Level Indicator | Pending |
| CNAV-07 | FR-008 | P1: Scroll-Wheel Zoom | Pending |
| CNAV-08 | FR-009 | P2: Image Quality | Pending |

**Coverage:** 8 total, 0 mapped to tasks, 8 unmapped ⚠️

---

## Success Criteria

- [ ] User can zoom in/out via header buttons, keyboard shortcuts, and scroll wheel
- [ ] Zoom-to-cursor works: the image point under the cursor stays fixed during scroll zoom
- [ ] User can pan with middle mouse button at any zoom level
- [ ] Zoom level indicator updates in real time after every zoom action
- [ ] All zoom operations stay within the 10%–800% range (ADR-003)
- [ ] Image is displayed with bilinear interpolation at all zoom levels
- [ ] App runs correctly inside Flatpak (`flatpak run`)
