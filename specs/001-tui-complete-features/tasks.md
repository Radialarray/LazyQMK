# Tasks: Complete TUI Keyboard Layout Editor

**Input**: Design documents from `/specs/001-tui-complete-features/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: No dedicated test tasks included - tests will be written inline during implementation per TDD approach

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `- [ ] [ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- All paths are absolute from repository root

## Path Conventions

- **Single Rust project**: `src/`, `tests/` at repository root
- User config: `~/.config/layout_tools/`
- QMK submodule: `vial-qmk-keebart/`

---

## Phase 1: Setup (Shared Infrastructure) ✅ COMPLETE

**Purpose**: Project initialization and basic structure

- [X] T001 Initialize Cargo project with dependencies: ratatui 0.26, crossterm 0.27, serde 1.0, serde_json, serde_yaml, toml 0.8, regex, anyhow, clap, dirs
- [X] T002 [P] Create project directory structure: src/models/, src/parser/, src/tui/, src/keycode_db/, src/firmware/
- [X] T003 [P] Create test directory structure: tests/unit/, tests/integration/, tests/contract/
- [X] T004 [P] Configure Cargo.toml with proper workspace settings and rustfmt/clippy configuration

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data models and infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Core Data Models

- [X] T005 [P] Implement Position struct in src/models/mod.rs (row: u8, col: u8 with derives)
- [X] T006 [P] Implement RgbColor struct in src/models/rgb.rs with hex parsing/serialization and validation
- [X] T007 [P] Implement Category struct in src/models/category.rs (id, name, color with validation)
- [X] T008 Implement KeyDefinition struct in src/models/layer.rs (position, keycode, label, color_override, category_id, combo_participant)
- [X] T009 Implement Layer struct in src/models/layer.rs (number, name, default_color, category_id, keys Vec)
- [X] T010 Implement LayoutMetadata struct in src/models/layout.rs (name, description, author, created, modified, tags, is_template, version)
- [X] T011 Implement Layout struct in src/models/layout.rs with color resolution method implementing four-level priority system
- [X] T012 [P] Write unit tests for color priority resolution in tests/unit/color_priority_tests.rs

### Keyboard Geometry System

- [X] T013 [P] Implement KeyGeometry struct in src/models/keyboard_geometry.rs (matrix_position, led_index, visual_x, visual_y, width, height, rotation)
- [X] T014 Implement KeyboardGeometry struct in src/models/keyboard_geometry.rs with matrix dimension tracking
- [X] T015 Implement VisualLayoutMapping struct in src/models/visual_layout_mapping.rs with bidirectional HashMaps (led_to_matrix, matrix_to_led, matrix_to_visual, visual_to_matrix)
- [X] T016 Implement coordinate transformation methods in src/models/visual_layout_mapping.rs (led_to_matrix_pos, matrix_to_visual_pos, visual_to_matrix_pos, visual_to_led_index)
- [X] T017 Implement split keyboard column reversal logic in src/models/visual_layout_mapping.rs
- [X] T018 [P] Write unit tests for three-coordinate mapping transformations in tests/unit/coordinate_tests.rs

### Configuration System

- [X] T019 [P] Implement Config structs (PathConfig, BuildConfig, UiConfig) in src/config.rs with serde derives
- [X] T020 Implement TOML loading with platform-specific config directory resolution in src/config.rs
- [X] T021 Implement TOML validation (QMK path, keyboard, layout, output_format) in src/config.rs
- [X] T022 Implement atomic config save using temp file + rename pattern in src/config.rs
- [X] T023 [P] Write integration tests for config persistence in tests/integration/config_tests.rs

### Keycode Database

- [X] T024 Create keycodes.json file in src/keycode_db/ with 600+ QMK keycodes organized by category (Basic, Navigation, Symbols, Function, Media, Modifiers)
- [X] T025 Implement KeycodeDatabase struct in src/keycode_db/mod.rs with HashMap for O(1) lookups
- [X] T026 Implement keycode validation against database in src/keycode_db/mod.rs
- [X] T027 Implement fuzzy search for keycode picker in src/keycode_db/mod.rs with category filtering
- [X] T028 [P] Write unit tests for keycode validation in tests/unit/keycode_validation_tests.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 7 - Configuration and Keyboard Selection (Priority: P1) 🎯 FOUNDATION ✅ COMPLETE

**Goal**: Enable QMK path configuration, keyboard selection, and layout variant selection before any editing can occur

**Why First**: This is prerequisite for all other stories - cannot load geometry or generate firmware without valid configuration

**Independent Test**: Run first-time setup wizard, enter QMK firmware path, verify path validation, select keyboard from scanned list, choose layout variant, confirm configuration saves to ~/.config/layout_tools/config.toml

### QMK Path Configuration

- [X] T029 [P] [US7] Implement QMK path validation (Makefile check, keyboards/ directory check) in src/config.rs
- [X] T030 [P] [US7] Implement keyboard scanner that reads keyboards/ directory in src/parser/keyboard_json.rs
- [X] T031 [US7] Implement QMK info.json parser in src/parser/keyboard_json.rs extracting keyboard_name, manufacturer, layouts
- [X] T032 [US7] Implement layout definition extraction from info.json in src/parser/keyboard_json.rs
- [X] T033 [US7] Implement KeyboardGeometry builder from info.json layout array in src/parser/keyboard_json.rs
- [X] T034 [US7] Implement VisualLayoutMapping builder from KeyboardGeometry in src/models/visual_layout_mapping.rs
- [X] T035 [P] [US7] Write contract tests for parsing real QMK info.json files in tests/contract/qmk_info_json_tests.rs

### Configuration Dialogs

- [X] T036 [P] [US7] Implement onboarding wizard state in src/tui/onboarding_wizard.rs (current_step, inputs HashMap)
- [X] T037 [US7] Implement onboarding wizard rendering with step-by-step prompts in src/tui/onboarding_wizard.rs
- [X] T038 [US7] Implement onboarding wizard event handling (path entry, validation, navigation) in src/tui/onboarding_wizard.rs
- [X] T039 [P] [US7] Implement path configuration dialog in src/tui/config_dialogs.rs for Ctrl+P shortcut
- [X] T040 [P] [US7] Implement keyboard picker state and rendering in src/tui/config_dialogs.rs with filterable list
- [X] T041 [US7] Implement keyboard picker event handling with fuzzy search in src/tui/config_dialogs.rs for Ctrl+K shortcut
- [X] T042 [P] [US7] Implement layout picker rendering showing available variants from info.json in src/tui/config_dialogs.rs
- [X] T043 [US7] Implement layout picker event handling for Ctrl+Y shortcut in src/tui/config_dialogs.rs

**Checkpoint**: Configuration system complete - can now load keyboard geometry and proceed with editing features

---

## Phase 4: User Story 4 - File Persistence and Management (Priority: P1) 🎯 FOUNDATION

**Goal**: Enable saving and loading keyboard layouts as human-readable Markdown files

**Why Second**: File I/O is required before any editing features can be useful - need to persist work

**Independent Test**: Create new layout, make changes, verify asterisk appears in title bar, press Ctrl+S, verify file written with Markdown table structure, close and reopen file, confirm all changes preserved

### Markdown Parsing

- [ ] T044 [P] [US4] Implement YAML frontmatter parser for LayoutMetadata in src/parser/layout.rs
- [ ] T045 [P] [US4] Implement Markdown table parsing state machine in src/parser/table.rs (InFrontmatter, InLayerHeader, InLayerColor, InTable, InCategories)
- [ ] T046 [US4] Implement layer header parsing (## Layer N: Name) in src/parser/layer.rs
- [ ] T047 [US4] Implement layer color parsing (**Color**: #RRGGBB) in src/parser/layer.rs
- [ ] T048 [US4] Implement layer category parsing (**Category**: category-id) in src/parser/layer.rs
- [ ] T049 [US4] Implement table row parsing with keycode syntax extraction in src/parser/table.rs
- [ ] T050 [US4] Implement keycode syntax parsing: KC_X, KC_X{#RRGGBB}, KC_X@category-id, KC_X{#RRGGBB}@category-id in src/parser/table.rs
- [ ] T051 [US4] Implement category section parsing in src/parser/layout.rs
- [ ] T052 [US4] Implement post-parse validation (keycode existence, category references, position coverage) in src/parser/layout.rs
- [ ] T053 [P] [US4] Write unit tests for parser round-trip (Markdown → Layout → Markdown) in tests/unit/parser_tests.rs

### Markdown Generation

- [ ] T054 [P] [US4] Implement YAML frontmatter serialization in src/parser/template_gen.rs
- [ ] T055 [US4] Implement layer section generation (header, color, category, table) in src/parser/template_gen.rs
- [ ] T056 [US4] Implement table generation with 12-column or 14-column format based on layout in src/parser/template_gen.rs
- [ ] T057 [US4] Implement keycode syntax serialization with color/category annotations in src/parser/template_gen.rs
- [ ] T058 [US4] Implement category section generation in src/parser/template_gen.rs
- [ ] T059 [US4] Implement atomic file write using temp file + rename pattern in src/parser/template_gen.rs

### File Operations

- [ ] T060 [P] [US4] Implement layout loading from file path in src/main.rs
- [ ] T061 [P] [US4] Implement save operation (Ctrl+S) with dirty flag clearing in src/tui/mod.rs
- [ ] T062 [US4] Implement unsaved changes prompt on quit (Ctrl+Q) in src/tui/mod.rs
- [ ] T063 [US4] Implement dirty flag tracking in AppState with asterisk display in title bar in src/tui/mod.rs
- [ ] T064 [P] [US4] Write integration tests for full save/load cycle in tests/integration/file_io_tests.rs

**Checkpoint**: File persistence complete - can save and load layouts reliably

---

## Phase 5: User Story 1 - Visual Keyboard Layout Editing (Priority: P1) 🎯 MVP CORE

**Goal**: Enable visual keyboard display, navigation, and keycode assignment in terminal interface

**Independent Test**: Load keyboard layout, navigate to key using arrow keys, press Enter to open keycode picker, select keycode, verify key displays new assignment with color

### Terminal UI Foundation

- [ ] T065 [P] [US1] Implement AppState struct in src/tui/mod.rs (layout, source_path, dirty, current_layer, selected_position, active_popup, component states, system resources)
- [ ] T066 [P] [US1] Implement terminal initialization and cleanup in src/tui/mod.rs with Ratatui and Crossterm
- [ ] T067 [US1] Implement main event loop with 100ms poll timeout in src/tui/mod.rs
- [ ] T068 [US1] Implement event routing (main UI vs popup) in src/tui/mod.rs

### Keyboard Widget Rendering

- [ ] T069 [P] [US1] Implement keyboard widget struct in src/tui/keyboard.rs
- [ ] T070 [US1] Implement key rendering with terminal coordinate conversion (keyboard units → terminal chars) in src/tui/keyboard.rs
- [ ] T071 [US1] Implement key label formatting (keycode abbreviation, color indicators i/k/L/d) in src/tui/keyboard.rs
- [ ] T072 [US1] Implement selected key highlighting in yellow in src/tui/keyboard.rs
- [ ] T073 [US1] Implement color resolution display using Layout.resolve_key_color method in src/tui/keyboard.rs
- [ ] T074 [US1] Implement viewport culling for keys outside terminal bounds in src/tui/keyboard.rs

### Navigation System

- [ ] T075 [P] [US1] Implement arrow key navigation in src/tui/mod.rs (↑↓←→ move cursor)
- [ ] T076 [P] [US1] Implement VIM-style navigation (hjkl) in src/tui/mod.rs
- [ ] T077 [US1] Implement layer switching with Tab/Shift+Tab in src/tui/mod.rs
- [ ] T078 [US1] Implement key clearing with 'x' or Delete (set to KC_TRNS) in src/tui/mod.rs
- [ ] T079 [US1] Implement terminal resize handling with reflow in src/tui/mod.rs

### Keycode Picker

- [ ] T080 [P] [US1] Implement KeycodePickerState struct in src/tui/keycode_picker.rs (search, selected, active_category)
- [ ] T081 [US1] Implement keycode picker rendering with categorized list in src/tui/keycode_picker.rs
- [ ] T082 [US1] Implement fuzzy search with real-time filtering in src/tui/keycode_picker.rs
- [ ] T083 [US1] Implement category switching with number keys in src/tui/keycode_picker.rs
- [ ] T084 [US1] Implement keycode selection and assignment to key in src/tui/keycode_picker.rs
- [ ] T085 [US1] Integrate keycode picker with Enter key event in src/tui/mod.rs

### Status Bar

- [ ] T086 [P] [US1] Implement status bar widget in src/tui/status_bar.rs showing mode, position, layer
- [ ] T087 [US1] Implement contextual help messages in status bar in src/tui/status_bar.rs
- [ ] T088 [US1] Implement error message display in red with actionable guidance in src/tui/status_bar.rs

**Checkpoint**: MVP core editing complete - can visually edit keyboard layouts in terminal

---

## Phase 6: User Story 2 - Color Management System (Priority: P1) 🎯 MVP ENHANCEMENT

**Goal**: Enable visual organization of keys using four-level color priority system

**Independent Test**: Create category with color, assign to key, verify key displays category color with 'k' indicator, set individual color override, verify it takes precedence with 'i' indicator

### Color Picker

- [ ] T089 [P] [US2] Implement ColorPickerState struct in src/tui/color_picker.rs (r, g, b, active_channel)
- [ ] T090 [US2] Implement RGB color picker rendering with three channel sliders in src/tui/color_picker.rs
- [ ] T091 [US2] Implement slider navigation with arrow keys in src/tui/color_picker.rs
- [ ] T092 [US2] Implement live color preview with hex code display in src/tui/color_picker.rs
- [ ] T093 [US2] Implement color application to selected key (Shift+C) in src/tui/mod.rs
- [ ] T094 [US2] Implement layer default color setting (c key) in src/tui/mod.rs

### Category Assignment

- [ ] T095 [P] [US2] Implement CategoryPickerState struct in src/tui/category_picker.rs (selected index)
- [ ] T096 [US2] Implement category picker rendering with color previews in src/tui/category_picker.rs
- [ ] T097 [US2] Implement category selection and assignment to key (Shift+K) in src/tui/category_picker.rs
- [ ] T098 [US2] Implement layer category assignment in src/tui/category_picker.rs

### Color Indicator Display

- [ ] T099 [US2] Implement color source indicator rendering in key top-right corner in src/tui/keyboard.rs
- [ ] T100 [US2] Update keyboard widget to display 'i' for individual override in src/tui/keyboard.rs
- [ ] T101 [US2] Update keyboard widget to display 'k' for key category in src/tui/keyboard.rs
- [ ] T102 [US2] Update keyboard widget to display 'L' for layer category in src/tui/keyboard.rs
- [ ] T103 [US2] Update keyboard widget to display 'd' for layer default in src/tui/keyboard.rs

**Checkpoint**: Color management complete - keys can be organized visually with four-level priority

---

## Phase 7: User Story 3 - Category Management (Priority: P2)

**Goal**: Enable CRUD operations on categories for organizing keys by logical function

**Independent Test**: Press Ctrl+T to open category manager, create category named "Navigation" with green color, assign to multiple keys, verify all display green with 'k' indicator

### Category Manager UI

- [ ] T104 [P] [US3] Implement CategoryManagerState struct in src/tui/category_manager.rs (categories Vec, selected, mode)
- [ ] T105 [US3] Implement category manager rendering with category list in src/tui/category_manager.rs
- [ ] T106 [US3] Implement category manager opening with Ctrl+T in src/tui/mod.rs

### Category CRUD Operations

- [ ] T107 [P] [US3] Implement category creation ('n' key) with name prompt in src/tui/category_manager.rs
- [ ] T108 [US3] Implement category creation color picker integration in src/tui/category_manager.rs
- [ ] T109 [US3] Implement category rename ('r' key) with current name prefill in src/tui/category_manager.rs
- [ ] T110 [US3] Implement category color change ('c' key) opening color picker in src/tui/category_manager.rs
- [ ] T111 [US3] Implement category deletion ('d' key) with confirmation prompt in src/tui/category_manager.rs
- [ ] T112 [US3] Implement category reference cleanup on deletion in src/tui/category_manager.rs
- [ ] T113 [US3] Implement layer category assignment (Shift+L) in src/tui/category_manager.rs

**Checkpoint**: Category management complete - categories can be created, edited, and assigned

---

## Phase 8: User Story 6 - Firmware Generation and Building (Priority: P1) 🎯 MVP OUTPUT

**Goal**: Generate QMK firmware C code and Vial JSON, compile firmware in background with progress tracking

**Why Now**: This is the output goal - users need to generate firmware to use their layouts on hardware

**Independent Test**: Press Ctrl+G to generate firmware, verify keymap.c and vial.json created, press Ctrl+B to build, watch progress in status bar, view complete build log with Ctrl+L

### Firmware Generation

- [ ] T114 [P] [US6] Implement firmware validator in src/firmware/validator.rs (invalid keycodes, matrix coverage checks)
- [ ] T115 [US6] Implement keymap.c generator in src/firmware/generator.rs with PROGMEM keymap arrays
- [ ] T116 [US6] Implement layer-by-layer keymap generation using LED index order in src/firmware/generator.rs
- [ ] T117 [US6] Implement vial.json generator in src/firmware/generator.rs with layout definition
- [ ] T118 [US6] Implement generation trigger (Ctrl+G) with pre-validation in src/tui/mod.rs
- [ ] T119 [P] [US6] Write integration tests for firmware generation pipeline in tests/integration/firmware_gen_tests.rs

### Background Building

- [ ] T120 [P] [US6] Implement BuildState struct in src/firmware/builder.rs (status, message channel receiver, log accumulator)
- [ ] T121 [US6] Implement background build thread spawning in src/firmware/builder.rs
- [ ] T122 [US6] Implement QMK make command execution with output capture in src/firmware/builder.rs
- [ ] T123 [US6] Implement build progress message channel (BuildProgress, BuildLog, BuildComplete) in src/firmware/builder.rs
- [ ] T124 [US6] Implement build trigger (Ctrl+B) spawning background thread in src/tui/mod.rs
- [ ] T125 [US6] Implement build status display in status bar (Idle/Validating/Compiling/Success/Failed) in src/tui/status_bar.rs
- [ ] T126 [US6] Implement main loop polling of build channel with UI updates in src/tui/mod.rs

### Build Log Viewer

- [ ] T127 [P] [US6] Implement BuildLogState struct in src/tui/build_log.rs (log_lines Vec, scroll_offset)
- [ ] T128 [US6] Implement build log rendering with scrollable history in src/tui/build_log.rs
- [ ] T129 [US6] Implement log level color coding (INFO/OK/ERROR) in src/tui/build_log.rs
- [ ] T130 [US6] Implement build log opening with Ctrl+L in src/tui/mod.rs
- [ ] T131 [US6] Implement log scrolling with arrow keys and Home/End in src/tui/build_log.rs

**Checkpoint**: Firmware generation complete - can generate and compile QMK firmware from layouts

---

## Phase 9: User Story 5 - Template System (Priority: P2)

**Goal**: Enable saving, browsing, and loading reusable layout templates

**Independent Test**: Press Shift+T to save current layout as template, enter name/description/tags, verify template appears in ~/.config/layout_tools/templates/, press 't' to browse, select template, confirm it loads into session

### Template Storage

- [ ] T132 [P] [US5] Implement template save dialog state in src/tui/mod.rs (metadata prompts)
- [ ] T133 [US5] Implement template save dialog rendering (name, description, author, tags) in src/tui/mod.rs
- [ ] T134 [US5] Implement template save dialog event handling (Shift+T) in src/tui/mod.rs
- [ ] T135 [US5] Implement template directory creation (~/.config/layout_tools/templates/) in src/parser/template_gen.rs
- [ ] T136 [US5] Implement template file generation with is_template=true in frontmatter in src/parser/template_gen.rs

### Template Browser

- [ ] T137 [P] [US5] Implement TemplateBrowserState struct in src/tui/template_browser.rs (templates Vec, search, selected)
- [ ] T138 [US5] Implement template scanning from directory in src/tui/template_browser.rs
- [ ] T139 [US5] Implement template browser rendering with metadata display in src/tui/template_browser.rs
- [ ] T140 [US5] Implement template search/filter by name and tags in src/tui/template_browser.rs
- [ ] T141 [US5] Implement template selection and loading into session in src/tui/template_browser.rs
- [ ] T142 [US5] Implement template browser opening with 't' key in src/tui/mod.rs
- [ ] T143 [US5] Implement unsaved changes warning before loading template in src/tui/template_browser.rs

**Checkpoint**: Template system complete - templates can be saved, browsed, and loaded

---

## Phase 10: User Story 8 - Help System and Documentation (Priority: P2)

**Goal**: Provide comprehensive help documentation accessible via '?' key

**Independent Test**: Press '?' to open help overlay, verify all shortcuts documented and organized, scroll with arrow keys, close with Escape

### Help Overlay

- [ ] T144 [P] [US8] Implement HelpOverlayState struct in src/tui/help_overlay.rs (scroll_offset)
- [ ] T145 [US8] Create help content with all keyboard shortcuts organized by category in src/tui/help_overlay.rs
- [ ] T146 [US8] Implement help overlay rendering as centered modal (60% width, 80% height) in src/tui/help_overlay.rs
- [ ] T147 [US8] Implement help content scrolling with arrow keys, Home, End in src/tui/help_overlay.rs
- [ ] T148 [US8] Implement scrollbar indicator for long content in src/tui/help_overlay.rs
- [ ] T149 [US8] Implement help toggle with '?' key in src/tui/mod.rs

### Help Categories

- [ ] T150 [US8] Document Navigation shortcuts in help content in src/tui/help_overlay.rs
- [ ] T151 [US8] Document Editing shortcuts in help content in src/tui/help_overlay.rs
- [ ] T152 [US8] Document File Operations shortcuts in help content in src/tui/help_overlay.rs
- [ ] T153 [US8] Document Build shortcuts in help content in src/tui/help_overlay.rs
- [ ] T154 [US8] Document Configuration shortcuts in help content in src/tui/help_overlay.rs
- [ ] T155 [US8] Document System shortcuts in help content in src/tui/help_overlay.rs

**Checkpoint**: Help system complete - all features documented and discoverable

---

## Phase 11: User Story 9 - Metadata Management (Priority: P3)

**Goal**: Enable editing layout metadata through dedicated dialog

**Independent Test**: Press Ctrl+E to open metadata editor, modify name and description fields, add tags as comma-separated values, confirm changes, verify metadata in YAML frontmatter when saved

### Metadata Editor

- [ ] T156 [P] [US9] Implement MetadataEditorState struct in src/tui/metadata_editor.rs (active_field, values HashMap)
- [ ] T157 [US9] Implement metadata editor rendering as form with fields in src/tui/metadata_editor.rs
- [ ] T158 [US9] Implement field navigation with Tab in src/tui/metadata_editor.rs
- [ ] T159 [US9] Implement text input with Backspace in src/tui/metadata_editor.rs
- [ ] T160 [US9] Implement comma-separated tag input parsing in src/tui/metadata_editor.rs
- [ ] T161 [US9] Implement metadata confirmation updating Layout.metadata with modified timestamp in src/tui/metadata_editor.rs
- [ ] T162 [US9] Implement metadata editor opening with Ctrl+E in src/tui/mod.rs
- [ ] T163 [US9] Implement metadata editor cancellation with Escape in src/tui/metadata_editor.rs

**Checkpoint**: Metadata management complete - layout metadata can be edited through UI

---

## Phase 12: User Story 10 - Multi-Layout Support (Priority: P2)

**Goal**: Enable switching between keyboard layout variants at runtime

**Independent Test**: Press Ctrl+Y with keyboard that has multiple layouts, select different variant, verify keyboard rendering updates to show correct key count and positions without restart

### Layout Switching

- [ ] T164 [P] [US10] Implement layout variant detection from info.json in src/parser/keyboard_json.rs
- [ ] T165 [US10] Enhance layout picker to display key counts for each variant in src/tui/mod.rs
- [ ] T166 [US10] Implement geometry rebuild on layout switch in src/tui/mod.rs
- [ ] T167 [US10] Implement visual layout mapping rebuild on layout switch in src/tui/mod.rs
- [ ] T168 [US10] Implement keyboard rendering update with new geometry in src/tui/keyboard.rs

### Layout Format Handling

- [ ] T169 [US10] Implement 12-column vs 14-column table format detection in src/parser/table.rs
- [ ] T170 [US10] Implement EX key column handling (col 6 and 13) in src/parser/table.rs
- [ ] T171 [US10] Implement dynamic column count adjustment in table generation in src/parser/template_gen.rs
- [ ] T172 [US10] Implement split keyboard left/right half rendering with visual offset in src/tui/keyboard.rs

**Checkpoint**: Multi-layout support complete - can switch between layout variants at runtime

---

## Phase 13: Polish & Cross-Cutting Concerns

**Purpose**: Final improvements and validation

### Error Handling Enhancement

- [ ] T173 [P] Improve error messages throughout application with file paths, line numbers, specific problems, suggested fixes
- [ ] T174 [P] Add context to all anyhow errors using .context() method
- [ ] T175 Implement recovery instructions in help documentation for common failures

### Performance Optimization

- [ ] T176 [P] Verify event-driven rendering only triggers on state changes
- [ ] T177 [P] Profile keycode search performance ensuring <100ms response time
- [ ] T178 Optimize terminal rendering to maintain 60fps (16ms/frame budget)
- [ ] T179 Verify memory footprint remains <100MB during typical editing session

### Code Quality

- [ ] T180 [P] Run cargo clippy and fix all warnings
- [ ] T181 [P] Run cargo fmt to ensure consistent formatting
- [ ] T182 Add inline documentation comments for public APIs
- [ ] T183 Verify all modules have proper error handling with no unwrap() calls in production paths

### Documentation

- [ ] T184 [P] Update README.md with installation instructions and basic usage
- [ ] T185 [P] Verify quickstart.md examples match implementation
- [ ] T186 Add troubleshooting section to documentation with common issues and solutions

### Final Validation

- [ ] T187 Run all unit tests and verify 100% pass rate
- [ ] T188 Run all integration tests with real QMK submodule
- [ ] T189 Run all contract tests with actual keyboard info.json files
- [ ] T190 Verify constitution compliance for all seven principles
- [ ] T191 Test full user workflow from quickstart.md end-to-end
- [ ] T192 Verify all 10 user stories can be completed independently

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Story 7 (Phase 3)**: Depends on Foundational - BLOCKS stories that need geometry/config
- **User Story 4 (Phase 4)**: Depends on Foundational - BLOCKS stories that need file I/O
- **User Story 1 (Phase 5)**: Depends on Foundational, US7, US4 - Core editing MVP
- **User Story 2 (Phase 6)**: Depends on US1 - Enhances editing with colors
- **User Story 3 (Phase 7)**: Depends on US2 - Manages categories for colors
- **User Story 6 (Phase 8)**: Depends on US7, US4 - Firmware output
- **User Story 5 (Phase 9)**: Depends on US4 - Template file I/O
- **User Story 8 (Phase 10)**: Depends on US1 - Help for main features
- **User Story 9 (Phase 11)**: Depends on US4 - Metadata editing
- **User Story 10 (Phase 12)**: Depends on US7, US1 - Layout switching
- **Polish (Phase 13)**: Depends on all desired user stories being complete

### User Story Dependencies

```
US7 (Config) ─────┬──────► US1 (Editing) ──► US2 (Colors) ──► US3 (Categories)
                  │                    │
US4 (Files) ──────┴──────────────┬────┴──────► US6 (Firmware)
                                 │
                                 ├────────────► US5 (Templates)
                                 ├────────────► US9 (Metadata)
                                 
US1 + US7 ────────────────────────────────────► US10 (Layouts)
US1 ───────────────────────────────────────────► US8 (Help)
```

### Critical Path for MVP

1. Setup (Phase 1)
2. Foundational (Phase 2) - CRITICAL BLOCKER
3. US7: Configuration (Phase 3) - Required for geometry loading
4. US4: File Persistence (Phase 4) - Required for saving work
5. US1: Visual Editing (Phase 5) - Core editing MVP
6. US2: Color System (Phase 6) - Visual organization MVP
7. US6: Firmware (Phase 8) - Output generation MVP

**STOP HERE for minimal viable product** - Remaining stories enhance but aren't blocking

### Parallel Opportunities

**Within Foundational Phase**:
- T005-T007 (Position, RgbColor, Category) can run in parallel
- T012 (color tests) can run in parallel with T013-T018 (geometry)
- T019-T022 (config structs) can run in parallel with T024-T027 (keycode db)

**Within User Story 1**:
- T069 (keyboard widget) and T080 (keycode picker state) can run in parallel
- T075-T076 (navigation) can run in parallel with T086-T087 (status bar)

**Within User Story 6**:
- T114 (validator) and T120-T123 (build thread) can run in parallel
- T127-T129 (build log widget) can run in parallel with T120-T123

**Across User Stories** (after dependencies met):
- US3, US5, US8, US9, US10 can all be developed in parallel by different team members
- All [P] tasks within each story can be parallelized

---

## Parallel Example: User Story 1 Implementation

```bash
# These tasks can start simultaneously after dependencies are met:

# Terminal UI foundation
Task T065: "Implement AppState struct in src/tui/mod.rs"
Task T066: "Implement terminal initialization in src/tui/mod.rs"

# Keyboard widget (parallel with AppState)
Task T069: "Implement keyboard widget struct in src/tui/keyboard.rs"

# Navigation (parallel with rendering)
Task T075: "Implement arrow key navigation in src/tui/mod.rs"
Task T076: "Implement VIM-style navigation in src/tui/mod.rs"

# Keycode picker state (parallel with main UI)
Task T080: "Implement KeycodePickerState struct in src/tui/keycode_picker.rs"

# Status bar (parallel with everything)
Task T086: "Implement status bar widget in src/tui/status_bar.rs"
Task T087: "Implement contextual help messages in src/tui/status_bar.rs"
```

---

## Implementation Strategy

### MVP First (Critical Path Only)

1. Complete Phase 1: Setup → Foundation scaffolded
2. Complete Phase 2: Foundational → Core models ready
3. Complete Phase 3: US7 Configuration → Can load keyboards
4. Complete Phase 4: US4 File Persistence → Can save/load
5. Complete Phase 5: US1 Visual Editing → Can edit layouts
6. Complete Phase 6: US2 Color System → Visual organization
7. Complete Phase 8: US6 Firmware → Can generate output
8. **STOP and VALIDATE**: Test end-to-end workflow
9. Deploy/demo MVP

**Estimated MVP**: ~190 tasks (T001-T131 + subset of foundational)

### Incremental Delivery Beyond MVP

1. Add Phase 7: US3 Categories → Enhanced organization
2. Add Phase 9: US5 Templates → Workflow acceleration
3. Add Phase 10: US8 Help → Discoverability
4. Add Phase 11: US9 Metadata → Better file management
5. Add Phase 12: US10 Multi-Layout → Multiple keyboards
6. Complete Phase 13: Polish → Production ready

Each increment adds value without breaking previous features.

### Parallel Team Strategy

With 3 developers after foundational phase:

- **Developer A**: US7 → US1 → US2 (Critical path MVP)
- **Developer B**: US4 → US6 (File I/O and firmware)
- **Developer C**: US3 → US5 → US8 (Enhancement features)

Integrate at checkpoints for coherent system.

---

## Task Completion Validation

### Per-Task Validation

- [ ] Task code compiles without warnings (cargo clippy)
- [ ] Task code is formatted (cargo fmt)
- [ ] Task includes inline tests/examples where appropriate
- [ ] Task updates dirty flag if modifying state
- [ ] Task follows constitution principles (see plan.md)

### Per-Story Validation

- [ ] Story can be tested independently per "Independent Test" criteria
- [ ] Story checkpoint reached with all tasks completed
- [ ] Story features accessible through documented shortcuts
- [ ] Story integrates cleanly with previously completed stories
- [ ] Story maintains <16ms render time, <100ms response time

### Full System Validation

- [ ] All user stories P1-P3 completed and tested
- [ ] All 10 success criteria from spec.md verified
- [ ] Quickstart.md workflow executes successfully
- [ ] Constitution compliance verified for all VII principles
- [ ] Memory footprint <100MB, firmware generation <2s

---

## Notes

- **[P] = Parallel**: Tasks marked [P] can run simultaneously (different files, no blocking dependencies)
- **[Story] = Traceability**: Maps task to specific user story for independent delivery
- **Tests inline**: Write tests during implementation, not as separate tasks
- **Checkpoint validation**: Stop at each phase checkpoint to verify story completeness
- **Constitution compliance**: Every task must follow principles from plan.md
- **Atomic commits**: Commit after each task or logical group of [P] tasks
- **Error handling**: All tasks must include proper error handling with context
- **File paths**: All tasks specify exact file paths for clarity

---

## Success Metrics Mapping

Tasks map to success criteria from spec.md:

- **SC-001** (10s keycode assignment): US1 tasks T075-T085
- **SC-002** (95% color distinction): US2 tasks T089-T103
- **SC-003** (responsive UI): US1 T067-T068, Performance T176-T178
- **SC-004** (3min setup): US7 tasks T036-T043
- **SC-005** (2s validation): US6 task T114
- **SC-006** (60s compilation): US6 tasks T120-T126
- **SC-007** (20s category create): US3 tasks T107-T108
- **SC-008** (1s save, readable): US4 tasks T054-T059
- **SC-009** (500ms template load): US5 tasks T137-T139
- **SC-010** (100ms keycode search): Foundational T027
- **SC-011** (resize without crash): US1 task T079
- **SC-012** (5min feature discovery): US8 tasks T144-T155
- **SC-013** (90% first-time success): US6 validation T114-T119
- **SC-014** (meaningful diffs): US4 tasks T054-T059
- **SC-015** (<100MB memory): Performance task T179

---

**Total Tasks**: 192
- Setup: 4 tasks
- Foundational: 24 tasks (CRITICAL PATH)
- US7 (Config): 15 tasks
- US4 (Files): 21 tasks
- US1 (Editing): 24 tasks (MVP CORE)
- US2 (Colors): 15 tasks
- US3 (Categories): 10 tasks
- US6 (Firmware): 18 tasks (MVP OUTPUT)
- US5 (Templates): 12 tasks
- US8 (Help): 12 tasks
- US9 (Metadata): 8 tasks
- US10 (Layouts): 9 tasks
- Polish: 20 tasks

**MVP Subset**: ~131 tasks (Phases 1-2, US7, US4, US1, US2, US6)
**Full Feature Set**: 192 tasks
