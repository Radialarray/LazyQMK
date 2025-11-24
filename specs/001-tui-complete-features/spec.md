# Feature Specification: Complete TUI Keyboard Layout Editor

**Feature Branch**: `001-tui-complete-features`  
**Created**: 2024-11-24  
**Status**: Draft  
**Input**: Complete TUI keyboard layout editor with all features from QUICKSTART and architecture guide

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Visual Keyboard Layout Editing (Priority: P1)

Users can visually edit keyboard layouts in a terminal interface with real-time feedback, navigate between keys using keyboard shortcuts, and assign keycodes to physical key positions.

**Why this priority**: This is the core MVP functionality. Without visual editing and keycode assignment, the application has no value. All other features depend on being able to view and modify keyboard layouts.

**Independent Test**: Can be fully tested by loading a keyboard layout, navigating to a key using arrow keys, pressing Enter to open the keycode picker, selecting a keycode, and verifying the key displays the new assignment with appropriate color coding.

**Acceptance Scenarios**:

1. **Given** a loaded keyboard layout, **When** user presses arrow keys or hjkl, **Then** cursor moves to adjacent keys and highlights the selected key in yellow
2. **Given** a key is selected, **When** user presses Enter, **Then** keycode picker opens with searchable list of 600+ QMK keycodes organized by category
3. **Given** keycode picker is open, **When** user types search text, **Then** list filters in real-time with fuzzy matching on keycode names and descriptions
4. **Given** a keycode is selected in picker, **When** user presses Enter, **Then** selected key updates with new keycode, dirty flag is set, and picker closes
5. **Given** a key has been modified, **When** user presses 'x' or Delete, **Then** key is cleared to KC_TRNS (transparent)
6. **Given** multiple layers exist, **When** user presses Tab or Shift+Tab, **Then** active layer switches and displays keys for that layer

---

### User Story 2 - Color Management System (Priority: P1)

Users can organize keys visually using a four-level color priority system, assigning colors to individual keys, key categories, layer categories, or layer defaults to aid muscle memory and visual recognition.

**Why this priority**: Color coding is essential for making complex layouts learnable and usable. This is part of the MVP because without it, users cannot effectively distinguish between different key functions, especially on layers with many keys.

**Independent Test**: Can be tested by creating a category with a color, assigning it to a key, verifying the key displays that color with a 'k' indicator, then setting an individual color override and verifying it takes precedence with an 'i' indicator.

**Acceptance Scenarios**:

1. **Given** a key is selected, **When** user presses Shift+C, **Then** color picker opens with RGB sliders showing current color
2. **Given** color picker is open, **When** user adjusts RGB values with arrow keys, **Then** preview updates in real-time and hex code displays current value
3. **Given** color is applied to key, **When** key is rendered, **Then** key shows color override with 'i' indicator in top-right corner
4. **Given** a layer is active, **When** user presses 'c', **Then** color picker opens for layer default color
5. **Given** categories are defined, **When** user presses Shift+K on a key, **Then** category picker opens showing available categories with color previews
6. **Given** key has category color and individual override, **When** key is rendered, **Then** individual override takes precedence over category color

---

### User Story 3 - Category Management (Priority: P2)

Users can create, edit, and delete categories to organize keys by logical function, assign colors to categories, and bulk-apply categories to keys or entire layers.

**Why this priority**: Categories enable advanced organization and are needed to fully utilize the color management system. This enhances the MVP but isn't required for basic editing functionality.

**Independent Test**: Can be tested by pressing Ctrl+T to open category manager, creating a new category named "Navigation" with a green color, assigning it to multiple keys, and verifying all assigned keys display the category color with 'k' indicator.

**Acceptance Scenarios**:

1. **Given** category manager is open (Ctrl+T), **When** user presses 'n', **Then** prompt appears for category name
2. **Given** category name is entered, **When** user confirms, **Then** color picker opens to set category color
3. **Given** category color is set, **When** category is created, **Then** it appears in category list with color preview
4. **Given** a category is selected in manager, **When** user presses 'r', **Then** rename prompt appears with current name
5. **Given** a category is selected in manager, **When** user presses 'c', **Then** color picker opens to change category color
6. **Given** a category is selected in manager, **When** user presses 'd', **Then** confirmation prompt appears, and on confirm, category is deleted and all references are removed
7. **Given** category manager is open, **When** user presses Shift+L, **Then** layer category assignment picker opens

---

### User Story 4 - File Persistence and Management (Priority: P1)

Users can save keyboard layouts as human-readable Markdown files, load existing layouts, and see a dirty flag indicator when there are unsaved changes.

**Why this priority**: Without the ability to save and load work, the application is unusable. This is critical MVP functionality that enables all other workflows.

**Independent Test**: Can be tested by making changes to a layout, verifying asterisk appears in title bar, pressing Ctrl+S, verifying file is written with Markdown table structure, closing and reopening the file, and confirming all changes are preserved.

**Acceptance Scenarios**:

1. **Given** layout has been modified, **When** dirty flag is set, **Then** asterisk (*) appears in title bar next to filename
2. **Given** user presses Ctrl+S, **When** save operation completes, **Then** layout is written as Markdown with YAML frontmatter, tables for each layer, and dirty flag clears
3. **Given** a Markdown layout file exists, **When** application opens file, **Then** metadata is parsed from frontmatter, layers are reconstructed from tables, and keycodes are validated
4. **Given** unsaved changes exist, **When** user presses Ctrl+Q, **Then** confirmation prompt appears warning of unsaved changes
5. **Given** confirmation prompt is shown, **When** user presses Ctrl+Q again, **Then** application quits without saving
6. **Given** layout has color overrides and categories, **When** file is saved, **Then** Markdown includes color syntax {#RRGGBB} and category syntax @category-id in table cells

---

### User Story 5 - Template System (Priority: P2)

Users can save commonly-used layouts as reusable templates with metadata, browse available templates with search functionality, and load templates as starting points for new layouts.

**Why this priority**: Templates accelerate workflow for users creating multiple similar layouts. This is valuable but not essential for basic editing, making it a P2 enhancement.

**Independent Test**: Can be tested by pressing Shift+T to save current layout as template, entering name/description/tags, verifying template appears in ~/.config/layout_tools/templates/, pressing 't' to browse templates, selecting the template, and confirming it loads into current session.

**Acceptance Scenarios**:

1. **Given** user presses Shift+T, **When** template save dialog opens, **Then** prompts appear for name, description, author, and tags
2. **Given** template metadata is entered, **When** user confirms, **Then** template is saved to templates directory with is_template=true in frontmatter
3. **Given** user presses 't', **When** template browser opens, **Then** list of templates appears with name, description, and tags visible
4. **Given** template browser is open, **When** user types search text, **Then** templates filter by name and tag matching
5. **Given** a template is selected, **When** user presses Enter, **Then** template loads into current session, source path updates to current file, and dirty flag is set
6. **Given** current layout has unsaved changes, **When** user attempts to load template, **Then** confirmation prompt warns of data loss

---

### User Story 6 - Firmware Generation and Building (Priority: P1)

Users can generate QMK firmware C code and Vial JSON configuration from their layouts, compile firmware in the background with progress tracking, and view detailed build logs.

**Why this priority**: Firmware generation is the end goal of creating layouts. Without this, users cannot actually use their keyboard layouts on hardware. This is essential MVP functionality.

**Independent Test**: Can be tested by pressing Ctrl+G to generate firmware, verifying keymap.c and vial.json are created in correct directories, pressing Ctrl+B to build firmware, watching progress updates in status bar, and viewing complete build log with Ctrl+L.

**Acceptance Scenarios**:

1. **Given** user presses Ctrl+G, **When** generation starts, **Then** validation runs checking for invalid keycodes and matrix coverage
2. **Given** validation passes, **When** generation proceeds, **Then** keymap.c is created with PROGMEM keymap array and vial.json is created with layout definition
3. **Given** generation fails validation, **When** error occurs, **Then** clear error message indicates which keys have invalid keycodes or missing matrix positions
4. **Given** user presses Ctrl+B, **When** build starts, **Then** background thread spawns, status changes to "Building", and main UI remains responsive
5. **Given** build is running, **When** progress updates arrive, **Then** status bar shows current phase (Generating/Compiling/Linking) and build log updates in real-time
6. **Given** build completes successfully, **When** firmware file is created, **Then** success message shows file path and size, and build log includes complete output
7. **Given** build fails, **When** compilation errors occur, **Then** error messages are captured in build log with line numbers and highlighted in red

---

### User Story 7 - Configuration and Keyboard Selection (Priority: P1)

Users can configure QMK firmware path, select target keyboard from available options, choose layout variant, and set output format and directory through interactive dialogs.

**Why this priority**: Configuration is required before any firmware generation can work. This is prerequisite MVP functionality that gates the firmware generation feature.

**Independent Test**: Can be tested by running first-time setup wizard, entering QMK firmware path, verifying path validation, selecting keyboard from scanned list, choosing layout variant, and confirming configuration is saved to ~/.config/layout_tools/config.toml.

**Acceptance Scenarios**:

1. **Given** application starts with no config, **When** onboarding wizard appears, **Then** step-by-step prompts guide through QMK path, keyboard selection, and layout selection
2. **Given** user presses Ctrl+P, **When** path configuration dialog opens, **Then** user can enter QMK firmware directory path with validation
3. **Given** QMK path is entered, **When** validation runs, **Then** checks for Makefile, keyboards directory, and displays error if structure is invalid
4. **Given** user presses Ctrl+K, **When** keyboard picker opens, **Then** scanned list of keyboards from QMK directory appears with manufacturer and name
5. **Given** keyboard picker is open, **When** user types search text, **Then** keyboard list filters in real-time
6. **Given** keyboard is selected, **When** user presses Ctrl+Y, **Then** layout picker shows available variants from info.json with key counts
7. **Given** configuration changes, **When** user confirms, **Then** config is saved to TOML file and geometry is reloaded without restart

---

### User Story 8 - Help System and Documentation (Priority: P2)

Users can access comprehensive help documentation by pressing '?', view all keyboard shortcuts organized by category, scroll through help content, and see context-sensitive status messages.

**Why this priority**: Help improves discoverability and reduces learning curve, but users can learn through trial and error. This enhances usability but isn't blocking for MVP.

**Independent Test**: Can be tested by pressing '?' to open help overlay, verifying all shortcuts are documented and organized (Navigation, Editing, File Operations, Build, Configuration, System), scrolling with arrow keys, and closing with Escape.

**Acceptance Scenarios**:

1. **Given** user presses '?', **When** help overlay opens, **Then** centered modal appears covering 60% width and 80% height with scrollable content
2. **Given** help overlay is open, **When** content exceeds viewport, **Then** scrollbar indicator appears and arrow keys scroll content
3. **Given** help is scrollable, **When** user presses Home or End, **Then** help jumps to top or bottom respectively
4. **Given** help overlay is open, **When** user presses Escape or '?', **Then** help closes and returns to main interface
5. **Given** user performs action, **When** status bar updates, **Then** contextual information appears about current mode, position, and available actions
6. **Given** error occurs, **When** error message displays, **Then** status bar shows error in red with actionable guidance for resolution

---

### User Story 9 - Metadata Management (Priority: P3)

Users can edit layout metadata including name, description, author, creation date, tags, and version information through a dedicated metadata editor dialog.

**Why this priority**: Metadata improves organization and searchability but doesn't affect core editing functionality. This is a nice-to-have feature for better file management.

**Independent Test**: Can be tested by pressing Ctrl+E to open metadata editor, modifying name and description fields, adding tags as comma-separated values, confirming changes, and verifying metadata appears in YAML frontmatter when file is saved.

**Acceptance Scenarios**:

1. **Given** user presses Ctrl+E, **When** metadata editor opens, **Then** form displays current values for name, description, author, and tags
2. **Given** metadata editor is open, **When** user modifies fields, **Then** Tab switches between fields and Backspace deletes characters
3. **Given** metadata editor is open, **When** user enters tags, **Then** comma-separated format is supported for multiple tags
4. **Given** user confirms metadata changes, **When** Enter is pressed, **Then** metadata is updated, modified timestamp refreshes, and dirty flag is set
5. **Given** metadata editor is open, **When** user presses Escape, **Then** changes are discarded and editor closes

---

### User Story 10 - Multi-Layout Support (Priority: P2)

Users can switch between different keyboard layout variants (36/40/42/46 keys) at runtime, load appropriate geometry from QMK info.json, and have the UI dynamically adjust to the selected layout.

**Why this priority**: Multi-layout support enables the application to work with various keyboard models, expanding its utility. This is important but not blocking for single-keyboard workflows.

**Independent Test**: Can be tested by pressing Ctrl+Y with a keyboard that has multiple layouts defined, selecting a different layout variant, and verifying keyboard rendering updates to show correct key count and positions without restart.

**Acceptance Scenarios**:

1. **Given** keyboard has multiple layouts in info.json, **When** layout picker opens (Ctrl+Y), **Then** available variants appear with key counts (e.g., "LAYOUT_split_3x6_3 (42 keys)")
2. **Given** user selects different layout, **When** layout is applied, **Then** keyboard geometry rebuilds, visual mapping updates, and rendering shows new key positions
3. **Given** layout changes, **When** table parsing occurs, **Then** column count adjusts (12 vs 14 columns) based on layout requirements
4. **Given** layout has EX keys, **When** 14-column format is used, **Then** extra keys render in correct positions with special column handling
5. **Given** split keyboard layout is selected, **When** geometry loads, **Then** left/right halves render with correct visual offset and right side column reversal

---

### Edge Cases

- What happens when user selects an invalid keycode that isn't in the QMK database?
- How does system handle corrupted Markdown files with malformed tables or invalid syntax?
- What happens when QMK firmware path is invalid or keyboards directory is missing?
- How does system handle extremely large keyboard layouts that exceed terminal dimensions?
- What happens when user attempts to flash firmware without putting keyboard in bootloader mode?
- How does system handle coordinate transformation errors for keys with unusual physical positions?
- What happens when template directory is inaccessible or read-only?
- How does system handle nested submodule failures during QMK submodule initialization?
- What happens when build process is interrupted or killed mid-compilation?
- How does system handle layouts with missing or incomplete matrix mapping definitions?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST render keyboard layouts in terminal with accurate physical key positions scaled to terminal coordinates
- **FR-002**: System MUST support navigation between keys using arrow keys (↑↓←→) and VIM-style shortcuts (hjkl)
- **FR-003**: System MUST provide keycode picker with fuzzy search across 600+ QMK keycodes organized by category (Basic, Navigation, Symbols, Function, Media, etc.)
- **FR-004**: System MUST implement four-level color priority system: individual key override > key category > layer category > layer default
- **FR-005**: System MUST display color source indicators ('i', 'k', 'L', 'd') in top-right corner of each key
- **FR-006**: System MUST persist layouts as Markdown files with YAML frontmatter, structured tables, and human-readable syntax for color overrides {#RRGGBB} and categories @category-id
- **FR-007**: System MUST track dirty flag and display asterisk in title bar when unsaved changes exist
- **FR-008**: System MUST support multiple layers with Tab/Shift+Tab navigation and layer-specific colors
- **FR-009**: System MUST provide category manager for CRUD operations (create, rename, change color, delete) on categories
- **FR-010**: System MUST validate keycodes against QMK database and display errors for invalid codes
- **FR-011**: System MUST generate QMK firmware keymap.c with PROGMEM arrays and vial.json configuration from layouts
- **FR-012**: System MUST compile firmware in background thread without blocking UI, streaming progress updates via message channel
- **FR-013**: System MUST capture and display build logs with color-coded log levels (INFO/OK/ERROR) and scrollable history
- **FR-014**: System MUST provide configuration system with TOML persistence for QMK path, keyboard, layout, output format, and directories
- **FR-015**: System MUST scan QMK keyboards directory and present filterable list of available keyboards
- **FR-016**: System MUST parse QMK info.json to extract keyboard geometry, available layouts, and matrix mapping
- **FR-017**: System MUST support keyboard switching at runtime with geometry reload and coordinate remapping
- **FR-018**: System MUST implement three-coordinate mapping system: visual positions ↔ matrix positions ↔ LED indices
- **FR-019**: System MUST handle split keyboard layouts with left/right half separation and column reversal for right side
- **FR-020**: System MUST provide RGB color picker with three channel sliders, hex input, and live preview
- **FR-021**: System MUST support template saving with metadata (name, description, author, tags, is_template flag)
- **FR-022**: System MUST provide template browser with search/filter by name and tags
- **FR-023**: System MUST display comprehensive help overlay accessible via '?' key with organized shortcut documentation
- **FR-024**: System MUST provide first-run onboarding wizard with QMK path validation and keyboard selection
- **FR-025**: System MUST implement event-driven rendering with 100ms poll timeout to maintain UI responsiveness
- **FR-026**: System MUST support atomic file writes using temp file + rename pattern for save operations
- **FR-027**: System MUST handle terminal resize events and reflow layout accordingly
- **FR-028**: System MUST validate matrix coverage ensuring all physical keys have defined positions
- **FR-029**: System MUST support clearing keys to KC_TRNS (transparent) via 'x' or Delete key
- **FR-030**: System MUST display status bar with current mode, cursor position, and contextual help

### Key Entities *(include if feature involves data)*

- **Layout**: Represents complete keyboard mapping with metadata (name, description, author, created, modified, tags, version), collection of layers, and collection of categories
- **Layer**: Numbered layer (0-N) with name, default RGB color, optional category assignment, and collection of key definitions for all positions
- **KeyDefinition**: Individual key assignment with visual position (row, col), QMK keycode string, optional label, optional color override (RGB), optional category assignment, and flags (combo participation)
- **Category**: User-defined grouping with unique ID (kebab-case), display name, and RGB color for organizing keys by logical function
- **KeyboardGeometry**: Physical keyboard definition with keyboard name, matrix dimensions (rows × cols), and collection of KeyGeometry entries defining each key's physical position
- **KeyGeometry**: Individual key physical properties with matrix position (row, col), LED index (sequential), visual position (x, y in keyboard units), dimensions (width, height), and rotation information
- **VisualLayoutMapping**: Coordinate transformation system with bidirectional mappings: LED index ↔ matrix position, matrix position ↔ visual position, enabling conversions between all three coordinate systems
- **RgbColor**: Color value with red, green, blue channels (0-255 each) and optional hex string representation
- **Config**: Application configuration with paths (qmk_firmware directory), build settings (keyboard, layout, keymap, output_format, output_dir), and UI preferences
- **LayoutMetadata**: File metadata with name, description, author, creation timestamp, modification timestamp, searchable tags, is_template flag, and schema version
- **Template**: Reusable layout stored in user config directory with enhanced metadata for sharing and discovery
- **BuildState**: Firmware compilation state tracking status (Idle/Validating/Compiling/Success/Failed), message channel receiver, and build log accumulator
- **AppState**: Centralized application state with current layout, active layer index, selected position, active popup/dialog, component states (pickers, managers, browsers), and system state (keycode database, geometry, mappings, config)

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can load an existing keyboard layout, navigate to any key, and assign a new keycode in under 10 seconds
- **SC-002**: Users can visually distinguish between different key functions using the color system with 95% accuracy after 5 minutes of use
- **SC-003**: Application remains responsive (no visible lag) during all UI interactions including keyboard navigation, popup opening, and search operations
- **SC-004**: Users can complete the first-run setup wizard and configure their QMK environment in under 3 minutes
- **SC-005**: Firmware generation from layout completes validation and file creation in under 2 seconds for standard 42-key layout
- **SC-006**: Background firmware compilation completes for typical keyboard in 30-60 seconds with real-time progress updates every 2 seconds
- **SC-007**: Users can create and apply a category to multiple keys in under 20 seconds using category manager
- **SC-008**: Save operations complete in under 1 second and produce human-readable Markdown files that can be edited manually
- **SC-009**: Template browser loads and displays all available templates with metadata in under 500ms
- **SC-010**: Keycode picker search returns filtered results within 100ms of each keystroke for fuzzy matching across 600+ keycodes
- **SC-011**: Application handles terminal resize without crashes and reflowed layout appears within 200ms
- **SC-012**: Users can discover all major features within 5 minutes using built-in help system and status bar guidance
- **SC-013**: 90% of users successfully generate and compile firmware for their keyboard on first attempt
- **SC-014**: Layout files saved by application can be version controlled with meaningful diffs that show exactly which keys changed
- **SC-015**: Application consumes less than 100MB memory during typical editing session with 6 layers and 42 keys per layer

## Assumptions *(optional)*

- Users have QMK firmware repository cloned and available on local filesystem
- Users are familiar with basic terminal navigation and keyboard shortcuts
- Target keyboards are supported by QMK firmware and have valid info.json definitions
- Users have appropriate toolchain installed for QMK compilation (arm-none-eabi-gcc, etc.)
- Terminal supports true color (24-bit RGB) or falls back gracefully to 256 colors
- Minimum terminal size of 80×24 characters is available
- Users have read/write permissions to ~/.config directory for storing configuration and templates
- QMK firmware submodule is properly initialized with all nested submodules
- Users understand mechanical keyboard terminology (layers, keycodes, firmware, flashing)
- Keyboard matrix mapping is correctly defined in QMK info.json files

## Dependencies *(optional)*

- **External**: QMK firmware repository (vial-qmk-keebart submodule) for keyboard definitions and compilation toolchain
- **External**: Rust toolchain (1.75+) with cargo for building application
- **External**: Terminal with ANSI escape sequence support and Unicode rendering
- **Library**: Ratatui 0.26 for terminal UI framework and immediate mode rendering
- **Library**: Crossterm 0.27 for cross-platform terminal manipulation and event handling
- **Library**: Serde 1.0 for JSON/YAML/TOML parsing and serialization
- **Library**: regex 1.0 for pattern matching in Markdown parsing
- **Data**: QMK keycode database embedded in application for validation and search
- **Configuration**: TOML configuration file at ~/.config/layout_tools/config.toml
- **Template Storage**: User template directory at ~/.config/layout_tools/templates/

## Out of Scope *(optional)*

- Graphical user interface (GUI) version of the application
- Web-based interface or cloud synchronization of layouts
- Direct keyboard flashing (users must manually flash firmware files)
- Real-time preview of RGB lighting effects on physical keyboard
- Macro recording or custom keycode definition beyond QMK standards
- Multi-user collaboration or concurrent editing of layouts
- Automatic firmware updates or QMK version management
- Support for non-QMK keyboard firmware (TMK, ZMK, etc.)
- Mobile or embedded versions of the application
- Integration with online keyboard communities or layout sharing platforms
- Advanced debugging tools for firmware issues
- Custom keyboard geometry editor (relies on QMK info.json)
- Automated testing of generated firmware
- Performance profiling or optimization tools for keymap code
