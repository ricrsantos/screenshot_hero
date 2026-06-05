# Screenshot Hero - State

*Persistent memory across sessions: decisions, blockers, lessons, deferred ideas.*

---

## Current Focus

**Feature:** PRD-001 - Screenshot Capture and Loading  
**Phase:** Planning complete → Ready to Execute  
**Next action:** Start T1 (Cargo.toml + project scaffold)

---

## Decisions

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-06-05 | Use `gtk4::DrawingArea` + Cairo for canvas | Validated in POC-003-04; direct rendering control, required for annotation layer later |
| 2026-06-05 | Use `gdk_pixbuf::Pixbuf` for image loading in Milestone 1 | Simpler API, validated in POC-002; migrate to `gdk4::Texture` if performance requires it |
| 2026-06-05 | Async portal calls via `glib::spawn_future_local` | Keeps GTK on main thread, avoids Send bounds issues with GTK objects |
| 2026-06-05 | Canvas zoom/pan deferred to PRD-002 | PRD-001 scope is capture + display; zoom/pan is navigation (PRD-002) |
| 2026-06-05 | No automated UI tests; unit tests for pure Rust logic only | GTK4 UI testing requires display server; impractical for CI without Xvfb/Wayland setup |

---

## Blockers

*None currently.*

---

## Todos

- [ ] Verify exact crate versions for gtk4-rs ecosystem before T1 (use Context7 or crates.io)
- [ ] Confirm Flatpak runtime version (GNOME SDK) used in POC-003 matches manifest in T12
- [ ] Confirm `ashpd` async runtime compatibility (zbus + glib) during T6

---

## Deferred Ideas

| Idea | Why Deferred | Revisit At |
|------|-------------|------------|
| `gdk4::Texture` for image loading | Simpler to start with Pixbuf; Texture has better GPU path | Milestone 5 (export) |
| Drag-and-drop image loading | Not in PRD-001 scope | PRD-001 backlog |
| Recent files list | Requires GSettings + file history | PRD-004 (project management) |
| Image format validation (magic bytes) | File extension check sufficient for v1 | PRD-001 backlog |

---

## Lessons Learned

*None yet — first session.*

---

## Preferences

*None recorded yet.*
