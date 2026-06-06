# Project Management Tasks

**Design**: `.specs/features/project-management/design.md`  
**Status**: Draft

---

## Execution Plan

### Phase 1: Data Layer (Sequential)

```
T1 → T2
```

### Phase 2: Persistence Core (Parallel)

After T2 completes:

```
     ┌→ T3 ─┐
T2 ──┤       ├──→ T5
     └→ T4 ─┘
```

### Phase 3: Project Manager (Sequential)

```
T5 → T6
```

Wait — T5 depends on T3 + T4; T6 depends on T5.

### Phase 4: UI Integration (Sequential)

```
T6 → T7 → T8
```

T7 and T8 are sequential because both modify `src/ui/window/imp.rs`.

---

```
Phase 1:
  T1 ──→ T2

Phase 2 (Parallel after T2):
  T2 ──┬──→ T3 [P]
       └──→ T4 [P]

Phase 3 (after T3 + T4):
  T3, T4 ──→ T5

Phase 4 (Sequential after T5):
  T5 ──→ T6 ──→ T7 ──→ T8
```

---

## Task Breakdown

### T1: Add `Serialize`/`Deserialize` to annotation model structs

**What**: Add `#[derive(Serialize, Deserialize)]` to all structs and enums in `src/annotations/model.rs`: `Annotation`, `AnnotationKind`, `AnnotationStyle`, `Rect`, `Point`, `Color`, `ArrowData`, `FreehandData`, `TextData`, `NumberMarkerData`, `CalloutData`.  
**Where**: `src/annotations/model.rs` (modify)  
**Depends on**: None  
**Reuses**: `serde` derive already in `Cargo.toml`; `uuid` already has `serde` feature  
**Requirement**: PROJ-03

**Done when**:

- [ ] All 11 structs/enums have `#[derive(Serialize, Deserialize)]` added alongside existing derives
- [ ] `cargo test --lib` passes with same test count as before (no silent deletions — currently 14 tests in `annotations` module)
- [ ] `cargo build` compiles without errors

**Tests**: unit  
**Gate**: quick  
**Commit**: `feat(annotations): add serde derives to annotation model`

---

### T2: Create persistence data model and module structure

**What**: Create `src/persistence/model.rs` with `SheroProject`, `SourceImageRecord`, `ViewState`, `ProjectMetadata`; create `src/persistence/error.rs` with `PersistenceError`; create `src/persistence/mod.rs` re-exporting public types; register `pub mod persistence;` in `src/lib.rs`.  
**Where**:
- `src/persistence/model.rs` (new)
- `src/persistence/error.rs` (new)
- `src/persistence/mod.rs` (new)
- `src/lib.rs` (modify — add `pub mod persistence;`)

**Depends on**: T1  
**Reuses**: `crate::annotations::model::Annotation` (after T1 derives applied)  
**Requirement**: PROJ-01, PROJ-03, PROJ-05

**Done when**:

- [ ] `SheroProject` struct defined with all 5 fields (`version: u32`, `source_image: SourceImageRecord`, `annotations: Vec<Annotation>`, `view_state: ViewState`, `metadata: ProjectMetadata`)
- [ ] All structs implement `#[derive(Debug, Serialize, Deserialize)]`
- [ ] `PersistenceError` enum covers `Io`, `Json`, `UnsupportedVersion(u32)`, `MissingSourceImage(String)` variants with `Display` impl
- [ ] `src/persistence/mod.rs` re-exports `ProjectManager`, `SheroProject`, `PersistenceError`
- [ ] Unit test: `SheroProject::default()` or a manually constructed instance round-trips through `serde_json::to_string` + `from_str` without data loss
- [ ] `cargo test --lib` passes (total test count ≥ previous + 1 new test)

**Tests**: unit  
**Gate**: quick  
**Commit**: `feat(persistence): add SheroProject data model and persistence module structure`

---

### T3: Implement project serializer [P]

**What**: `pub fn save_project(path: &Path, project: &SheroProject) -> Result<(), PersistenceError>` in `src/persistence/serializer.rs`. Uses atomic write: serialize to `.shero.tmp` in the same directory, then `fs::rename` to final path.  
**Where**: `src/persistence/serializer.rs` (new)  
**Depends on**: T2  
**Reuses**: `serde_json::to_string_pretty`, `std::fs`, `PersistenceError` from T2  
**Requirement**: PROJ-01

**Done when**:

- [ ] `save_project` serializes `SheroProject` to pretty-printed JSON
- [ ] Uses `{path}.tmp` + `fs::rename` for atomic write
- [ ] `Io` errors from `fs::write` / `fs::rename` are wrapped as `PersistenceError::Io`
- [ ] `Json` errors from `serde_json` are wrapped as `PersistenceError::Json`
- [ ] Unit test: `save_project` writes a valid `.shero` to a temp directory; reading it back with `fs::read_to_string` + `serde_json::from_str` returns an equivalent `SheroProject`
- [ ] `cargo test --lib` passes (total test count ≥ previous + 1 new test)

**Tests**: unit  
**Gate**: quick  
**Commit**: `feat(persistence): implement project serializer with atomic write`

---

### T4: Implement project deserializer [P]

**What**: `pub fn load_project(path: &Path) -> Result<SheroProject, PersistenceError>` in `src/persistence/deserializer.rs`. Reads file, parses JSON, rejects `version != 1` with `PersistenceError::UnsupportedVersion`.  
**Where**: `src/persistence/deserializer.rs` (new)  
**Depends on**: T2  
**Reuses**: `serde_json::from_str`, `std::fs`, `PersistenceError` from T2  
**Requirement**: PROJ-02, PROJ-03, PROJ-04, PROJ-05

**Done when**:

- [ ] `load_project` reads and parses a `.shero` JSON file into `SheroProject`
- [ ] Returns `PersistenceError::UnsupportedVersion(v)` when `project.version != 1`
- [ ] Returns `PersistenceError::Io` for file-not-found or read errors
- [ ] Returns `PersistenceError::Json` for malformed JSON
- [ ] Unit test: deserializing a valid sample `.shero` JSON string returns expected `SheroProject` fields
- [ ] Unit test: deserializing JSON with `"version": 2` returns `PersistenceError::UnsupportedVersion(2)`
- [ ] Unit test: deserializing malformed JSON returns `PersistenceError::Json`
- [ ] `cargo test --lib` passes (total test count ≥ previous + 3 new tests)

**Tests**: unit  
**Gate**: quick  
**Commit**: `feat(persistence): implement project deserializer with version validation`

---

### T5: Implement ProjectManager

**What**: `ProjectManager` struct in `src/persistence/manager.rs` with `current_path`, `auto_save_enabled`, `created_at` fields; methods: `new()`, `save()`, `save_as()`, `open()`, `maybe_auto_save()`.  
**Where**: `src/persistence/manager.rs` (new)  
**Depends on**: T3, T4  
**Reuses**: `serializer::save_project`, `deserializer::load_project`, `chrono::Utc::now().to_rfc3339()`; `env!("CARGO_PKG_VERSION")` for version  
**Requirement**: PROJ-01, PROJ-02, PROJ-05, PROJ-06, PROJ-07, PROJ-08, PROJ-09

**Done when**:

- [ ] `ProjectManager::new()` returns instance with `auto_save_enabled: true`, `current_path: None`, `created_at: None`
- [ ] `save(path, project)`: sets `created_at` on first call; always updates `modified_at`; sets `current_path`; calls serializer
- [ ] `save_as(path, project)`: same as `save` but always updates `current_path` to new path
- [ ] `open(path)`: calls deserializer; on success sets `current_path` and `created_at` from project metadata
- [ ] `maybe_auto_save(project)`: calls `save` only when `auto_save_enabled && current_path.is_some()`; logs `warn!` on error (no panic)
- [ ] Unit test: `save` + re-read verifies `created_at` is set and stable across subsequent saves; `modified_at` advances
- [ ] Unit test: `maybe_auto_save` with no path set does not write any file
- [ ] `cargo test --lib` passes (total test count ≥ previous + 2 new tests)

**Tests**: unit  
**Gate**: quick  
**Commit**: `feat(persistence): implement ProjectManager`

---

### T6: Add Canvas state accessor/mutator methods

**What**: Add 6 new pub methods to `Canvas` in `src/canvas/mod.rs`: `pan_offset()`, `all_annotations()`, `source_image_path()`, `source_image_dimensions()`, `restore_annotations()`, `restore_zoom_pan()`. Also add `History::clear()` to `src/annotations/history.rs`.  
**Where**:
- `src/canvas/mod.rs` (modify)
- `src/annotations/history.rs` (modify)

**Depends on**: T5  
**Reuses**: `imp().pan_offset.get()`, `imp().annotations.borrow().all()`, `imp().image.borrow()`, `imp().zoom.set()`, `imp().pan_offset.set()`  
**Requirement**: PROJ-03, PROJ-04

**Done when**:

- [ ] `canvas.pan_offset() -> (f64, f64)` returns current pan offset from `imp().pan_offset`
- [ ] `canvas.all_annotations() -> Vec<Annotation>` returns clone of all annotations from engine
- [ ] `canvas.source_image_path() -> Option<PathBuf>` returns path from `imp().image` if loaded
- [ ] `canvas.source_image_dimensions() -> Option<(u32, u32)>` returns `(width as u32, height as u32)` if image loaded
- [ ] `canvas.restore_annotations(annotations: Vec<Annotation>)` replaces engine contents; calls `queue_draw()`
- [ ] `canvas.restore_zoom_pan(zoom, pan_x, pan_y)` sets zoom + pan directly; calls `notify_zoom_changed(zoom)` + `queue_draw()`
- [ ] `History::clear()` clears both `undo_stack` and `redo_stack`
- [ ] Unit test: `History::clear()` on a non-empty history leaves `can_undo()` and `can_redo()` both false
- [ ] `cargo build` passes (UI code — no unit tests for Canvas methods; `History::clear` is unit tested)

**Tests**: unit (for `History::clear`), none (for Canvas methods — UI layer)  
**Gate**: build  
**Commit**: `feat(canvas): add state accessors, restore methods, and History::clear`

---

### T7: Add Save/Open GActions and header bar items

**What**: Register `win.save-project`, `win.save-project-as`, `win.open-project` GActions in `MainWindow::constructed()`; add `project_manager: RefCell<ProjectManager>` to `MainWindow` struct; add Save and Open Project buttons to the header bar; add `win.save-project` Ctrl+S keyboard shortcut.  
**Where**: `src/ui/window/imp.rs` (modify)  
**Depends on**: T6  
**Reuses**: `gio::SimpleAction`, existing GAction registration pattern, `crate::persistence::ProjectManager`  
**Requirement**: PROJ-01, PROJ-02

**Done when**:

- [ ] `MainWindow` struct has `project_manager: RefCell<ProjectManager>` field
- [ ] `win.save-project` GAction registered; handler collects state from canvas, calls `manager.save()` or shows file dialog + error dialog on failure; updates window title on success
- [ ] `win.save-project-as` GAction registered; handler always shows file dialog, then saves
- [ ] `win.open-project` GAction registered; handler shows `.shero`-filtered file dialog, calls `load_project()` + `FileLoader`, restores canvas, clears history, updates window title; shows error dialog on any failure
- [ ] "Save" button and "Open Project" button visible in header bar
- [ ] `win.save-project` bound to Ctrl+S keyboard accelerator in application startup
- [ ] `win.save-project` is disabled when no source image is loaded; enabled when canvas has an image
- [ ] `cargo build` passes

**Tests**: none (UI)  
**Gate**: build  
**Commit**: `feat(window): add Save/Open Project GActions and header bar items`

---

### T8: Wire auto-save and title into annotation_changed callback

**What**: Extend the `on_annotation_changed` closure in `MainWindow::constructed()` to also call `project_manager.borrow().maybe_auto_save(snapshot)` where `snapshot` is a freshly collected `SheroProject` from canvas state. Update `win.save-project` enable state when canvas image is set/cleared.  
**Where**: `src/ui/window/imp.rs` (modify)  
**Depends on**: T7  
**Reuses**: Existing `on_annotation_changed` callback wiring; canvas accessor methods from T6  
**Requirement**: PROJ-06, PROJ-07, PROJ-08, PROJ-09

**Done when**:

- [ ] `on_annotation_changed` closure calls `project_manager.borrow().maybe_auto_save(snapshot)` after updating undo/redo button states
- [ ] `snapshot` is built by collecting `all_annotations()`, `zoom_level()`, `pan_offset()`, `source_image_path()`, `source_image_dimensions()` from canvas
- [ ] Auto-save fires only when project has an established path (covered by `maybe_auto_save` internal check)
- [ ] `win.save-project` enabled/disabled state tracks canvas image presence (enabled when image loaded, disabled when not)
- [ ] Manual test: save a project, add an annotation, confirm `.shero` file updates without pressing Ctrl+S
- [ ] `cargo build` passes

**Tests**: none (UI)  
**Gate**: build  
**Commit**: `feat(window): wire auto-save trigger and save action enable state`

---

## Parallel Execution Map

```
Phase 1 (Sequential):
  T1 ──→ T2

Phase 2 (Parallel — both start after T2):
  T2 ──┬──→ T3 [P]  (serializer)
       └──→ T4 [P]  (deserializer)

Phase 3 (Sequential — starts after both T3 and T4 complete):
  T3 ┐
  T4 ┘──→ T5  (ProjectManager)

Phase 4 (Sequential — one per task, same file area):
  T5 ──→ T6 ──→ T7 ──→ T8
```

---

## Task Granularity Check

| Task | Scope | Status |
|------|-------|--------|
| T1: Add serde derives to annotation model | 1 file, additive change only | ✅ Granular |
| T2: Create persistence data model + module | 4 files (3 new + 1 one-liner) — cohesive module init | ✅ Granular |
| T3: Implement serializer | 1 function in 1 file | ✅ Granular |
| T4: Implement deserializer | 1 function in 1 file | ✅ Granular |
| T5: Implement ProjectManager | 1 struct in 1 file | ✅ Granular |
| T6: Add Canvas state methods + History::clear | 2 files, additive methods — cohesive state exposure task | ✅ Granular |
| T7: Add GActions + header bar items | 1 file, 1 concern (action registration + UI wiring) | ✅ Granular |
| T8: Wire auto-save callback | 1 file, 1 closure extension | ✅ Granular |

---

## Diagram-Definition Cross-Check

| Task | Depends On (task body) | Diagram Shows | Status |
|------|------------------------|---------------|--------|
| T1 | None | Starting node | ✅ Match |
| T2 | T1 | T1 → T2 | ✅ Match |
| T3 | T2 | T2 → T3 [P] | ✅ Match |
| T4 | T2 | T2 → T4 [P] | ✅ Match |
| T5 | T3, T4 | T3 → T5, T4 → T5 | ✅ Match |
| T6 | T5 | T5 → T6 | ✅ Match |
| T7 | T6 | T6 → T7 | ✅ Match |
| T8 | T7 | T7 → T8 | ✅ Match |

All match. ✅

---

## Test Co-location Validation

| Task | Code Layer Created/Modified | Matrix Requires | Task Says | Status |
|------|-----------------------------|-----------------|-----------|--------|
| T1: Add serde derives | `src/annotations/model.rs` (data model) | unit | unit | ✅ OK |
| T2: Persistence data model | `src/persistence/model.rs` (data model) | unit | unit | ✅ OK |
| T3: Serializer | `src/persistence/serializer.rs` (data model logic) | unit | unit | ✅ OK |
| T4: Deserializer | `src/persistence/deserializer.rs` (data model logic) | unit | unit | ✅ OK |
| T5: ProjectManager | `src/persistence/manager.rs` (logic) | unit | unit | ✅ OK |
| T6: Canvas methods + History::clear | `src/canvas/` (UI — none) + `src/annotations/history.rs` (logic — unit for clear()) | unit (History) + none (Canvas) | unit + none | ✅ OK |
| T7: GActions + header | `src/ui/window/` (UI component) | none | none | ✅ OK |
| T8: Auto-save wiring | `src/ui/window/` (UI component) | none | none | ✅ OK |

All pass. ✅

---

## Requirement Traceability (updated)

| Requirement ID | Story | Tasks | Status |
|----------------|-------|-------|--------|
| PROJ-01 | P1: Save Project | T2, T3, T5, T7 | Pending |
| PROJ-02 | P1: Open Project | T2, T4, T5, T7 | Pending |
| PROJ-03 | P1: Restore annotations | T1, T2, T4, T6 | Pending |
| PROJ-04 | P1: Restore canvas view | T2, T4, T6 | Pending |
| PROJ-05 | P3: Metadata | T2, T5 | Pending |
| PROJ-06 | P2: Auto save on execute | T5, T8 | Pending |
| PROJ-07 | P2: Auto save on undo | T5, T8 | Pending |
| PROJ-08 | P2: Auto save on redo | T5, T8 | Pending |
| PROJ-09 | P2: Auto save toggle | T5, T8 | Pending |

**Coverage:** 9 total, 9 mapped to tasks ✅
