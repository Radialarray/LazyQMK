# Plan: TUI Architecture & Core Refactor

## Overview

This plan describes an incremental refactor of the keyboard-configurator application
focused on:

- Improving maintainability and stability
- Reducing complexity in `main.rs` and `src/tui/mod.rs`
- De-duplicating QMK/geometry logic
- Preparing the TUI code for more advanced architectures (Component Trait or
  Message-based) without committing to a full rewrite upfront

The work is organized into phases that can be executed and reviewed
independently. Each phase is designed to be low-risk and to keep the
application behavior unchanged as much as possible.

Feature branch: `017-tui-architecture-refactor`

Related proposals:

- Proposal 1: Handlers Extraction (Minimal Disruption)
- Proposal 2: Component Trait Pattern (Medium Refactor)
- Proposal 3: Message-Based Architecture (Comprehensive)

This plan commits to Proposal 1 and core cleanup, while explicitly leaving
Proposals 2 and 3 as potential follow-up work.

---

## Phase A – Core Cleanup (Non-TUI)

Goal: Improve stability and remove duplication in the non-UI parts of the
application. These changes are largely orthogonal to how the TUI is
structured.

### A.1 Entrypoint Refactor (`main.rs`)

**Goal:** Turn `src/main.rs` into a thin orchestration layer that:

- Parses CLI arguments
- Chooses between high-level modes (init, open existing layout, first-run wizard)
- Delegates to dedicated orchestration functions in a new `app` module

**Current issues:**

- `main.rs` is ~650 lines and mixes:
  - CLI parsing
  - Onboarding wizard event loop
  - Layout picker event loop
  - Geometry and QMK integration logic
  - Layout creation & saving
  - TUI initialization and running

**Changes:**

1. Create `src/app/` module with submodules:
   - `src/app/mod.rs` – re-exports and top-level helpers
   - `src/app/onboarding.rs` – onboarding wizard orchestration
   - `src/app/layout_picker.rs` – layout selection orchestration
   - `src/app/launch.rs` – editor launch / layout creation orchestration

2. Move from `main.rs` into `app` modules:
   - `run_onboarding_wizard` → `app::onboarding::run_onboarding_wizard_terminal`
   - `run_new_layout_wizard` → `app::onboarding::run_new_layout_wizard_terminal`
   - `run_layout_picker` → `app::layout_picker::run_layout_picker_terminal`
   - `launch_editor_with_default_layout` → `app::launch::launch_editor_with_default_layout`
   - `create_default_layer` → either `models::layout_utils` or
     `app::launch::create_default_layer`

3. Simplify `main()` to:

   - Parse `Cli` with `clap`
   - For `--init` → call `app::onboarding::run_initial_setup` (which internally
     runs onboarding and launches editor as needed)
   - For `layout_path` argument → call
     `app::launch::open_existing_layout_and_run_editor`
   - For no args → call `app::launch::run_without_args`, which decides between
     first-run wizard and layout picker based on config

**Success criteria:**

- `src/main.rs` < 200 lines and only contains:
  - `Cli` struct
  - `main()` function
  - Calls into `app::*` functions
- No QMK/geometry-specific logic remains in `main.rs`.

---

### A.2 Geometry/QMK Service

**Goal:** De-duplicate and centralize the logic that builds keyboard geometry
and visual mapping from QMK `info.json` + optional variant `keyboard.json`.

**Current issues:**

- Similar logic appears in:
  - `launch_editor_with_default_layout` (in `main.rs`)
  - `run_layout_picker` (when loading existing layouts)
  - `tui::AppState::rebuild_geometry`
- Fallback to minimal geometry is implemented ad-hoc in multiple places.

**Changes:**

1. Add a new module, e.g. `src/services/geometry.rs` (or
   `src/firmware/geometry_service.rs`):

   ```rust
   pub struct GeometryContext<'a> {
       pub config: &'a Config,
       pub layout_metadata: &'a LayoutMetadata,
   }

   pub struct GeometryResult {
       pub geometry: KeyboardGeometry,
       pub mapping: VisualLayoutMapping,
       pub updated_metadata: LayoutMetadata,
   }

   pub fn build_geometry_for_layout(
       ctx: &GeometryContext<'_>,
   ) -> anyhow::Result<GeometryResult>;
   ```

2. Move all QMK integration logic into this module:

   - Use `parser::keyboard_json::{parse_keyboard_info_json, parse_variant_keyboard_json,
     build_keyboard_geometry_with_rgb, build_matrix_to_led_map}`
   - Use `config.build.determine_keyboard_variant` to resolve variant path
   - Encapsulate the fallback policy:
     - If QMK path is missing or JSON parse fails → return minimal geometry and
       a fresh `VisualLayoutMapping`

3. Update call sites to use the service:

   - `app::launch::launch_editor_with_default_layout`
   - `app::layout_picker` when opening an existing layout
   - `tui::AppState::rebuild_geometry`

4. Add tests for the service to cover:

   - Successful geometry build with RGB matrix present
   - Missing RGB matrix
   - Missing QMK path
   - Invalid `info.json`
   - Variant resolution based on layout key count

**Success criteria:**

- Only `services::geometry` directly uses `parse_keyboard_info_json` and
  `parse_variant_keyboard_json`.
- All geometry/mapping + keyboard/variant metadata updates go through the
  service.

---

### A.3 Layout Service

**Goal:** Centralize layout file I/O, metadata manipulation, and renaming so
that TUI and app code don\'t interact with the filesystem and parser directly.

**Changes:**

1. Add `src/services/layouts.rs` with operations such as:

   ```rust
   pub struct LayoutService;

   impl LayoutService {
       pub fn load(path: &Path) -> Result<Layout>;
       pub fn save(layout: &Layout, path: &Path) -> Result<()>;
       pub fn rename_file_if_needed(
           old_path: &Path,
           new_name: &str,
       ) -> Result<Option<PathBuf>>; // returns new path if renamed
   }
   ```

2. Refactor:

   - `main.rs` (now `app::launch`) layout loading and saving to use
     `LayoutService`.
   - Metadata editor file rename behavior to call
     `LayoutService::rename_file_if_needed`.

3. Add tests for `LayoutService` behavior, including rename edge cases.

**Success criteria:**

- Direct uses of `parser::parse_markdown_layout` and
  `parser::save_markdown_layout` are localized to `LayoutService`.
- File renaming logic is tested in isolation.

---

## Phase B – TUI Refactor Tier 1: Proposal 1 (Handlers Extraction)

Goal: Reduce the size and complexity of `src/tui/mod.rs` by extracting
input-handling functions into dedicated modules, without changing behavior or
`AppState` structure.

This is a structural refactor aligned with **Proposal 1: Handlers
Extraction (Minimal Disruption)**.

### B.1 New TUI Handlers Module Structure

Create a new directory:

```text
src/tui/
  mod.rs              # AppState, run_tui, top-level render & dispatch
  handlers/
    mod.rs            # Re-exports
    main.rs           # handle_main_input and related helpers
    popups.rs         # handle_popup_input and generic popup dispatch
    actions.rs        # dispatch_action + high-level action handlers
    settings.rs       # Settings manager handlers
    category.rs       # Category manager handlers
    layer.rs          # Layer manager handlers
    templates.rs      # Template browser/save handlers
```

### B.2 Extraction Targets

From `src/tui/mod.rs`, move the following into the new handlers modules:

- Settings manager:
  - `handle_settings_manager_input()` and any helper functions
    → `tui/handlers/settings.rs`

- Category manager:
  - `handle_category_manager_input()`
    → `tui/handlers/category.rs`

- Layer manager:
  - `handle_layer_manager_input()`
    → `tui/handlers/layer.rs`

- Template browser/save dialogs:
  - `handle_template_browser_input()`, `handle_template_save_dialog_input()`
    (and similar) → `tui/handlers/templates.rs`

- Build log, help overlay, metadata editor, layout picker, setup wizard,
  parameterized keycode flows, etc.:
  - Extract these input handlers to the most appropriate handler module,
    keeping signatures unchanged.

- High-level action dispatch (if present):
  - Any `dispatch_action()`-style function and its helpers
    → `tui/handlers/actions.rs`

### B.3 Wiring Changes in `tui/mod.rs`

1. Adjust `tui/mod.rs` to import the new handlers:

   ```rust
   mod handlers;

   use handlers::{
       handle_main_input,
       handle_popup_input,
       // ...etc.
   };
   ```

2. Keep `handle_key_event` and `handle_popup_input` in `tui/mod.rs` as thin
   delegates where helpful, or move their bodies fully into `handlers`.

3. Ensure all signatures remain the same, e.g.:

   ```rust
   pub fn handle_main_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool>;
   pub fn handle_popup_input(state: &mut AppState, key: event::KeyEvent) -> Result<bool>;
   ```

### B.4 Testing and Validation

- Run `cargo test` and `cargo clippy` after each significant extraction.
- Optionally add small unit tests for individual handlers by constructing a
  minimal `AppState` and asserting behavior of the handler functions.

**Success criteria:**

- `src/tui/mod.rs` shrinks significantly (target: reduce by ~50%+).
- No logical or behavioral changes in TUI; all tests pass.
- Handlers live in logically grouped files that are easier to navigate.

---

## Phase C – TUI Refactor Tier 2: Future Options

Phase C is **not** part of the initial implementation for this feature branch,
but we document it here for future decision-making.

Depending on ongoing pain points and priorities, we can later choose between
(or combine) the following deeper refactors.

### C.1 Option: Component Trait Pattern (Proposal 2)

**Goal:** Encapsulate state, rendering, and input handling for each popup or
component using a `Component` / `PopupComponent` trait, reducing `AppState`
size and improving testability.

**Key ideas:**

- Define a trait:

  ```rust
  pub trait PopupComponent {
      type Event;
      fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event>;
      fn render(&self, f: &mut Frame, area: Rect, theme: &Theme);
  }
  ```

- Store active popup as:

  ```rust
  pub struct AppState {
      // ... core data ...
      pub active_popup: Option<PopupType>,
      pub active_component: Option<Box<dyn PopupComponent<Event = PopupEvent>>>,
      // ... shared context (config, geometry, keycode_db, etc.) ...
  }
  ```

- Component-specific events mapped to higher-level actions via a
  `process_component_event(state: &mut AppState, event: PopupEvent)` function.

**Migration strategy:**

- Start with a simple popup (e.g. help overlay or color picker) as a pilot.
- Gradually move more popups into this pattern as touched.

### C.2 Option: Message-Based Architecture (Proposal 3)

**Goal:** Introduce an Elm-style architecture with:

- `AppState` as a single immutable state structure
- `Message` enum representing all possible events
- A pure `update(state, message)` function
- Separate `view()` and `command` (side-effect) handling

**Key artifacts:**

- `src/tui/state.rs` – `AppState` definition
- `src/tui/message.rs` – `Message` and sub-message enums
- `src/tui/update.rs` – `fn update(AppState, Message) -> (AppState, Option<Command>)`
- `src/tui/view.rs` – rendering functions
- `src/tui/command.rs` – side-effect execution

**Migration strategy:**

- Plan as a dedicated sprint/branch
- Reuse geometry + layout services from Phase A and handlers from Phase B as
  reference for message definitions and update logic

---

## Implementation Order & Checklist

### Order

1. **Phase A.1** – Entrypoint refactor (`main.rs` → `app` module)
2. **Phase A.2** – Geometry/QMK service
3. **Phase A.3** – Layout service
4. **Phase B** – TUI handlers extraction (Proposal 1)

Future work (separate branch/decision):

5. **Phase C.1** – Component Trait Pattern (optional)
6. **Phase C.2** – Message-Based Architecture (optional)

### Checklist

#### Phase A.1 – Entrypoint refactor

- [ ] Create `src/app/` with `mod.rs`, `onboarding.rs`, `layout_picker.rs`,
      `launch.rs`
- [ ] Move onboarding and layout picker loops to `app` module
- [ ] Move editor launch & default layer creation into `app::launch`
- [ ] Simplify `main()` to delegate into `app` functions
- [ ] Run `cargo test` and `cargo clippy`

#### Phase A.2 – Geometry/QMK service

- [ ] Create `services::geometry` (or `firmware::geometry_service`)
- [ ] Move QMK → geometry/mapping logic into service
- [ ] Update `app::launch`, `app::layout_picker`, and
      `tui::AppState::rebuild_geometry` to use the service
- [ ] Add unit tests for geometry service edge cases
- [ ] Run `cargo test` and `cargo clippy`

#### Phase A.3 – Layout service

- [ ] Create `services::layouts` with `load`, `save`, `rename_file_if_needed`
- [ ] Refactor layout loading/saving in `app::launch` to use `LayoutService`
- [ ] Refactor metadata editor file rename to use `LayoutService`
- [ ] Add tests for rename/file handling
- [ ] Run `cargo test` and `cargo clippy`

#### Phase B – Handlers Extraction (Proposal 1)

- [ ] Create `src/tui/handlers/` structure
- [ ] Move settings manager handlers to `tui/handlers/settings.rs`
- [ ] Move category manager handlers to `tui/handlers/category.rs`
- [ ] Move layer manager handlers to `tui/handlers/layer.rs`
- [ ] Move template browser/save handlers to `tui/handlers/templates.rs`
- [ ] Move or introduce `actions.rs` for high-level dispatch
- [ ] Update `tui/mod.rs` to use new handlers
- [ ] Optional: add unit tests for critical handlers
- [ ] Run `cargo test` and `cargo clippy`

---

## Rollback Plan

Because each phase is mostly structural and isolated, rollback is straightforward:

1. If a phase introduces issues, revert the corresponding commits or reset the
   branch to a known good point.
2. `main.rs` and `tui/mod.rs` can be restored from `main` if necessary.
3. Geometry and layout services can be temporarily unused while leaving the
   modules in place.

No changes to persistent user data formats are planned in this refactor; only
code structure and internal architecture are affected.

---

## Success Criteria

1. **Maintainability:**
   - `src/main.rs` is small and easy to read.
   - `src/tui/mod.rs` is significantly smaller and focused on wiring,
     not detailed input logic.

2. **Stability:**
   - All existing tests pass.
   - QMK/geometry behavior is consistent across all paths (wizard, layout
     picker, metadata editor).

3. **Extensibility:**
   - Adding new popups or TUI features requires touching fewer, more focused
     files.
   - The codebase is ready for optional future adoption of Component Trait or
     Message-based architectures without large-scale rewrites.
