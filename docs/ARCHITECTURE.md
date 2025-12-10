# LazyQMK - Architecture Documentation

> **Last Updated:** 2025-12-06

Comprehensive technical architecture and design decisions for the LazyQMK project.

---

## Table of Contents

1. [Overview](#overview)
2. [Technology Stack](#technology-stack)
3. [Architectural Patterns](#architectural-patterns)
4. [Coordinate System Architecture](#coordinate-system-architecture)
5. [Data Models](#data-models)
6. [State Management](#state-management)
7. [User Interface Components](#user-interface-components)
8. [Input Handling](#input-handling)
9. [Rendering System](#rendering-system)
10. [Theming System](#theming-system)
11. [File Format & Persistence](#file-format--persistence)
12. [Parser Architecture](#parser-architecture)
13. [Firmware Integration](#firmware-integration)
14. [Color Management](#color-management)
15. [Template System](#template-system)
16. [Configuration Management](#configuration-management)
17. [Performance Considerations](#performance-considerations)
18. [Project Structure](#project-structure)
19. [Future Architectural Considerations](#future-architectural-considerations)

---

## Overview

LazyQMK is a terminal-based keyboard layout editor built using **Model-View-Controller (MVC)** architecture with centralized state management. It provides a visual interface for editing mechanical keyboard layouts and generating QMK firmware.

### Core Features

**Visual Editing**
- Real-time keyboard visualization with accurate physical positioning
- Visual cursor navigation between keys
- Color-coded keys based on function/category
- Multiple layer support with tab-based navigation
- Support for split keyboards (Corne, Ergodox) and various layouts (36/40/42/46 keys)

**Key Assignment**
- Searchable keycode picker with fuzzy matching
- Category-based organization (Basic, Navigation, Symbols, etc.)
- Support for 600+ QMK keycodes
- Quick clear/reset functions

**Color Management**
- RGB color picker with channel-based adjustment
- Four-level color priority system (individual → key category → layer category → layer default)
- Visual indicators showing color source

**Organization & Templates**
- User-defined categories for grouping keys by function
- Template browser for reusable layouts
- Category manager for CRUD operations

**Firmware Integration**
- Generate QMK firmware C code from layouts
- Background compilation with progress tracking
- Multiple output formats (UF2, HEX, BIN)

---

## Technology Stack

### Core Framework
- **Rust 1.75+** - Systems programming language (using 1.88.0)
- **Ratatui 0.29** - TUI framework with immediate mode rendering
- **Crossterm 0.29** - Cross-platform terminal manipulation library

### Data & Serialization
- **Serde 1.0** - Serialization/deserialization framework
- **serde_json 1.0** - JSON parsing (QMK info.json)
- **serde_yml 0.0.12** - YAML frontmatter parsing
- **toml 0.9** - Configuration file format

### System Integration
- **dirs 6.0** - Cross-platform directory paths
- **arboard 3.6** - Clipboard integration
- **chrono 0.4** - Timestamp handling
- **dark-light 2.0** - OS theme detection

### Error Handling & CLI
- **anyhow 1.0** - Flexible error handling with context
- **clap 4.5** - Command-line argument parsing

---

## Architectural Patterns

### Model-View-Controller (MVC)

The application follows clear separation of concerns:

**Models** (`src/models/`)
- Data structures representing domain concepts
- `Layout`, `Layer`, `KeyDefinition` - Core layout data
- `KeyboardGeometry`, `MatrixMapping`, `VisualLayoutMapping` - Coordinate systems
- `Category`, `RgbColor` - Organization and styling
- No business logic - pure data containers

**Views** (`src/tui/`)
- UI components rendering state to terminal
- Keyboard widget, color picker, keycode picker, dialogs
- Read-only access to state
- No direct state modification

**Controller** (`src/tui/handlers/`)
- Event loop processing user input
- Input handlers for different contexts (navigation, popups, actions)
- Updates `AppState` based on events
- Coordinates between models and views

### Component Trait Pattern (Spec 017 - COMPLETE)

**Standardized Component Interface**
```rust
pub trait Component {
    type Event;
    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event>;
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme);
}

pub trait ContextualComponent<Context> {
    type Event;
    fn handle_input(&mut self, key: KeyEvent, context: &Context) -> Option<Self::Event>;
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme, context: &Context);
}
```

**Benefits**
- Encapsulated state, rendering, and input handling
- Event-driven communication (no tight coupling)
- Easy to test and maintain
- Clear separation of concerns

**Migrated Components (14 total)**
1. ColorPicker - `impl Component`
2. KeycodePicker - `impl ContextualComponent<KeycodeDb>`
3. LayerPicker - `impl Component`
4. CategoryPicker - `impl Component`
5. ModifierPicker - `impl Component`
6. CategoryManager - `impl Component`
7. MetadataEditor - `impl Component`
8. TemplateBrowser - `impl Component`
9. LayoutPicker - `impl Component`
10. KeyboardPicker - `impl Component`
11. BuildLog - `impl ContextualComponent<BuildLogContext>`
12. HelpOverlay - `impl Component`
13. KeyEditor - `impl Component`
14. TemplateSaveDialog - `impl Component`

**Migration Complete**
- All 14 active components migrated to Component/ContextualComponent traits
- Components integrated into AppState via `ActiveComponent` enum
- Handlers refactored to use component events
- State consolidation achieved with ~50% reduction in duplicate fields

### Application Flow

```
┌─────────────┐
│   Startup   │ Load config, parse layout file, initialize state
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Main Loop  │ ◄─────┐
└──────┬──────┘       │
       │              │
       ├──► Poll Event (100ms timeout)
       ├──► Handle Input
       ├──► Update State
       ├──► Render UI
       │              │
       └──────────────┘
       │
       ▼
┌─────────────┐
│  Shutdown   │ Save if dirty, cleanup terminal
└─────────────┘
```

---

## Coordinate System Architecture

### The Three-Coordinate Problem

Mechanical keyboards require managing three different coordinate systems:

**1. Matrix Coordinates** (Electrical Wiring)
- How the keyboard is physically wired (rows/columns)
- Example: Corne has 8 rows × 7 columns (4 rows per half)
- Used by firmware for key scanning

**2. LED Index** (Sequential Order)
- Order in which RGB LEDs are wired
- Sequential numbering (0, 1, 2, ...)
- Often follows a zigzag pattern
- Used for RGB lighting control

**3. Visual Position** (User's Mental Model)
- How keys appear in the markdown table and UI
- Logical grid positions
- Split into left/right halves for split keyboards
- Used for editing interface

### VisualLayoutMapping

**Core Responsibility**: Bidirectional transformations between all three coordinate systems.

**Key Methods**
- `led_to_matrix_pos()` - LED index → Matrix (row, col)
- `matrix_to_visual_pos()` - Matrix (row, col) → Visual (row, col)
- `visual_to_matrix_pos()` - Visual (row, col) → Matrix (row, col)
- `led_to_visual_pos()` - LED index → Visual (row, col)
- `visual_to_led_index()` - Visual (row, col) → LED index

**Split Keyboard Handling**
- Left half: Rows 0-3, Cols 0-6 → Visual rows 0-3, cols 0-6
- Right half: Rows 4-7, Cols 0-6 → Visual rows 0-3, cols 7-13
- Right columns often reversed (col 0 = rightmost physically)
- Special handling for EX keys and thumb clusters

### Physical to Terminal Rendering

**KeyboardGeometry** class stores physical positions:
- `visual_x`, `visual_y`: Position in keyboard units (1u = key width)
- `width`, `height`: Key dimensions in keyboard units
- Rotation information for angled keys

**Terminal Coordinate Transformation:**
- Scale factor: 7 characters per keyboard unit (horizontal)
- Scale factor: 2.5 lines per keyboard unit (vertical)
- Converts floating-point keyboard units to integer terminal cells
- Handles key rendering with borders and labels

### Handling Multiple Layouts

Different layouts (36/40/42/46 keys) require:
- Dynamic geometry loading from QMK info.json
- Layout-specific matrix mappings
- Flexible table parsing (12 vs 14 columns)
- Runtime geometry switching without restart

**Strategy:**
- Parse QMK's `info.json` to get available layouts
- Build geometry on-the-fly for selected layout
- Generate visual mapping from geometry
- Support layout switching via Ctrl+Y

---

## Data Models

### Layout Hierarchy

```
Layout
├── metadata: LayoutMetadata
│   ├── name, description, author
│   ├── created, modified (timestamps)
│   ├── tags (Vec<String>)
│   ├── is_template (bool)
│   ├── version (String)
│   └── layout_variant (e.g., "LAYOUT_split_3x6_3_ex2")
├── categories: Vec<Category>
│   └── id, name, color
└── layers: Vec<Layer>
    ├── number, name, default_color, category_id
    └── keys: Vec<KeyDefinition>
        ├── position, keycode, label
        ├── color_override, category_id
        └── combo_participant
```

### Key Models

**Position**
- `row: u8` - Visual row (0-3 for Corne)
- `col: u8` - Visual column (0-13 for 46-key)
- Implements equality, hashing for lookups

**KeyDefinition**
- Position in visual grid
- QMK keycode string
- Optional label (currently unused)
- Optional color override
- Optional category assignment
- Flags (combo participation, etc.)

**Layer**
- Sequential number (0-based)
- Human-readable name
- Default RGB color
- Optional category assignment
- Vector of key definitions (fixed size per layout)

### Geometry Models

**KeyGeometry**
- Matrix position (row, col)
- LED index
- Physical position (x, y in keyboard units)
- Dimensions (width, height)
- Rotation information

**KeyboardGeometry**
- Keyboard name
- Matrix dimensions (rows, cols)
- Vector of key geometries
- Helper methods for terminal coordinate conversion

**MatrixMapping**
- Bidirectional HashMap: `(row, col) ↔ LED index`
- O(1) lookups during rendering
- Built from QMK layout definition

**VisualLayoutMapping**
- LED to Matrix mapping (Vec)
- Matrix to LED mapping (HashMap)
- Matrix to Visual mapping (HashMap)
- Visual to Matrix mapping (HashMap)
- Handles all coordinate transformations

### Category System

**Category**
- Unique ID (string, kebab-case)
- Display name
- RGB color
- Used for organizing keys by function

**Color Priority Levels:**
1. Individual key color override (symbol: 'i')
2. Key category color (symbol: 'k')
3. Layer category color (symbol: 'L')
4. Layer default color (symbol: 'd')

### Metadata Models

**LayoutMetadata**
- `name`: Layout name
- `description`: Long description
- `author`: Creator name
- `created`: Timestamp
- `modified`: Last modified timestamp
- `tags`: Searchable keywords
- `is_template`: Template flag
- `version`: Schema version
- `layout_variant`: Selected QMK layout variant (e.g., "LAYOUT_split_3x6_3_ex2")

**Layout Variant Persistence:**
The `layout_variant` field stores the user's selected QMK layout variant in the markdown frontmatter. This enables:
- Proper geometry restoration when loading a saved layout
- Correct RGB matrix lookup for layer-aware coloring
- Automatic detection of keyboard variant (36/40/42/46 keys)

---

## State Management

### AppState - Single Source of Truth

**Core Data**
- `layout: Layout` - Current layout being edited
- `source_path: PathBuf` - File location
- `dirty: bool` - Unsaved changes flag

**UI State**
- `current_layer: usize` - Active layer index
- `selected_position: Position` - Cursor position
- `active_component: Option<ActiveComponent>` - Current dialog/picker
- `status_message: String` - Status bar text

**System State**
- `keycode_db: KeycodeDb` - QMK keycode database
- `keyboard_geometry: KeyboardGeometry` - Physical key positions
- `matrix_mapping: MatrixMapping` - Electrical wiring
- `visual_layout_mapping: VisualLayoutMapping` - Coordinate transforms
- `config: Config` - User configuration
- `theme: Theme` - Current color theme

**Build State**
- `build_state: BuildState` - Idle/Validating/Compiling/Success/Failed
- `build_receiver: mpsc::Receiver<BuildMessage>` - Background thread communication
- `build_log: Vec<String>` - Build output capture

### State Flow

```
User Input Event
    ↓
Input Handler
    ↓
Update AppState
    ↓
Set Dirty Flag (if data changed)
    ↓
Next Render Cycle
    ↓
UI Reflects Updated State
```

**Guarantees**
- Predictable state updates (single mutable owner)
- Easy debugging (inspect single state object)
- No synchronization issues (single-threaded UI)
- Clear data flow

### Popup State Management

**Popup Stack Pattern:**
- `active_popup`: Current popup enum
- `previous_popup`: For nested popups (e.g., color picker from category manager)

**Popup Lifecycle:**
1. User triggers popup (Enter, Ctrl+T, etc.)
2. Set `active_popup` to appropriate variant
3. Initialize popup-specific state
4. Render popup overlay
5. Handle popup-specific input
6. On close: Clear `active_popup`, restore previous

**Popup Types:**
- Modal dialogs (blocking main UI)
- Pickers (selection interfaces)
- Overlays (help, build log)
- Wizards (multi-step workflows)

### Dirty Flag Management

**When to Set Dirty:**
- Key assignment changed
- Layer added/deleted/renamed
- Color changed (layer or key)
- Category assigned
- Metadata edited
- Template loaded

**When to Clear Dirty:**
- File saved (Ctrl+S)
- Auto-save after major operations

**Usage:**
- Display asterisk in title bar when dirty
- Require double Ctrl+Q to quit with unsaved changes
- Auto-save before firmware generation

---

## User Interface Components

### Main Window Layout

```
┌─────────────────────────────────────────┐
│ Layer Tabs: [Base] [Lower] [Raise] ... │
├─────────────────────────────────────────┤
│                                         │
│          Keyboard Widget                │
│    (Visual key rendering with colors)   │
│                                         │
├─────────────────────────────────────────┤
│ Status Bar: [Mode] [Position] [Info]   │
└─────────────────────────────────────────┘
```

### Keyboard Widget

**Responsibilities:**
- Render all keys at correct positions
- Show key labels (keycodes)
- Apply color coding
- Highlight selected key
- Display color source indicators

**Rendering Strategy:**
1. Iterate through KeyboardGeometry keys
2. For each key:
   - Convert keyboard units to terminal coords
   - Find corresponding KeyDefinition by visual position
   - Determine key color (priority system)
   - Draw bordered box with keycode label
   - Add color source indicator in top-right corner

**Visual Elements:**
- Box borders using Unicode characters (┌─┐│└┘)
- Centered keycode text (abbreviated)
- Color indicator: 'i', 'k', 'L', or 'd'
- Yellow highlight for selected key
- Gray for transparent keys (KC_TRNS)

### Status Bar Widget

**Layout:**
- Bottom row of terminal
- Multiple sections: mode, position, info, help

**Sections:**
1. **Mode** - Current operation (Normal, Editing, Building)
2. **Position** - Selected key (Row X, Col Y)
3. **Debug Info** - Matrix position, LED index (if debug mode)
4. **Help** - Shortcut reminder ("Press ? for help")

**Color Coding:**
- Green: Normal operation
- Yellow: Input mode
- Cyan: Information
- Red: Errors
- Magenta: Building

### Keycode Picker Dialog

**Layout:**
```
┌─────────────────────────────────────────┐
│ Search: ___________                     │
├─────────────────────────────────────────┤
│ [Basic] [Nav] [Symbols] [Fn] [Media]...│
├─────────────────────────────────────────┤
│ > KC_A        Letter A                  │
│   KC_B        Letter B                  │
│   KC_C        Letter C                  │
│   ...                                   │
└─────────────────────────────────────────┘
```

**Features:**
- Fuzzy search across all keycodes
- Category tabs for organization
- Scrollable list
- Description tooltips
- Keyboard navigation (↑↓, Enter)

### Color Picker Dialog

**Layout:**
```
┌─────────────────────────────────────────┐
│ RGB Color Picker                        │
├─────────────────────────────────────────┤
│ [R]: ████████████▒▒▒▒▒▒▒▒ 192           │
│  G : ████████▒▒▒▒▒▒▒▒▒▒▒▒ 128           │
│  B : ████████████████████ 255           │
├─────────────────────────────────────────┤
│ Preview: ██████ #C080FF                 │
├─────────────────────────────────────────┤
│ ↑↓: ±1  Shift+↑↓: ±10  Tab: Channel    │
│ h: Enter hex  Del: Clear  Enter: Apply │
└─────────────────────────────────────────┘
```

**Features:**
- Three RGB channel sliders
- Visual preview with color swatch
- Hex code display and input
- Keyboard controls for precise adjustment
- Option to clear color (keys only)

### Category Manager Dialog

**Layout:**
```
┌─────────────────────────────────────────┐
│ Category Manager                        │
├─────────────────────────────────────────┤
│ > Navigation  [#00FF00]                 │
│   Symbols     [#FF0000]                 │
│   Function    [#0000FF]                 │
│   Media       [#FFFF00]                 │
├─────────────────────────────────────────┤
│ n: New  r: Rename  c: Color  d: Delete │
└─────────────────────────────────────────┘
```

**Operations:**
- Create new category
- Rename existing
- Change color (opens color picker)
- Delete category
- Navigate with arrow keys

### Build Log Dialog

**Layout:**
```
┌─────────────────────────────────────────┐
│ Build Log [Ctrl+C to copy]             │
├─────────────────────────────────────────┤
│ [INFO] Starting firmware build...       │
│ [OK]   Generating QMK firmware...       │
│ [INFO] Compiling...                     │
│ [OK]   Build successful!                │
│        Output: firmware.uf2 (234 KB)    │
└─────────────────────────────────────────┘
```

**Features:**
- Real-time log streaming
- Color-coded log levels
- Auto-scroll to bottom
- Ctrl+C to copy all to clipboard
- Scrollable history

### Help Overlay

**Features:**
- Centered modal overlay (60% width, 80% height)
- Scrollable content with arrow keys, Home/End, Page Up/Down
- Organized by category (Navigation, Editing, File Ops, Build, etc.)
- Color-coded sections

---

## Input Handling

### Event Loop Architecture

**Event Polling:**
- Use crossterm's `poll()` with 100ms timeout
- Non-blocking to allow background tasks
- Check for keyboard events, terminal resize

**Event Types:**
1. **Key Press** - Single key with optional modifiers
2. **Resize** - Terminal size changed
3. **Mouse** - Click/scroll events (limited support)

### Input Routing Pattern

```
Event Received
    │
    ├─ Active Popup?
    │   ├─ Yes → Route to popup handler
    │   └─ No → Route to main handler
    │
    ├─ Popup Handler
    │   ├─ Process popup-specific input
    │   ├─ Update popup state
    │   └─ Close popup if needed
    │
    └─ Main Handler
        ├─ Navigation (↑↓←→)
        ├─ Layer switching (Tab)
        ├─ Editing commands (Enter, x, c)
        ├─ System commands (Ctrl+S, Ctrl+Q)
        └─ Open popup if requested
```

### Keyboard Shortcut Design

**Note:** For the authoritative, up-to-date shortcut reference, press `?` in the application. The shortcuts below represent the conceptual design.

**Principles:**
- Ctrl for major operations (save, quit, build)
- Shift for variations (Shift+C = key color, C = layer color)
- Single letters for common actions (x = clear, c = color)
- Arrow keys for navigation
- Enter for confirm, Esc for cancel
- ? for help

**Navigation:**
- `↑↓←→` or `hjkl` - Move cursor
- `Tab` / `Shift+Tab` - Cycle layers

**Editing:**
- `Enter` - Open keycode picker
- `x` / `Delete` - Clear key
- `c` - Set layer color
- `Shift+C` - Set key color
- `Shift+K` - Assign key category
- `Shift+L` - Assign layer category

**File Operations:**
- `Ctrl+S` - Save
- `Shift+E` - Edit metadata
- `Ctrl+V` - Validate

**Templates:**
- `t` - Load template
- `Shift+T` - Save as template

**Build:**
- `Ctrl+G` - Generate firmware
- `Ctrl+B` - Build firmware
- `Ctrl+L` - View build log

**System:**
- `Ctrl+T` - Category manager
- `?` - Help overlay
- `Ctrl+Q` - Quit (twice if dirty)

---

## Rendering System

### Immediate Mode Rendering

**Concept**
- Entire UI rebuilt every frame from current state
- No retained UI state between frames
- State drives rendering, not vice versa

**Advantages**
- Simple mental model
- No synchronization issues
- Easy to reason about
- State is single source of truth

**Rendering Pipeline**
```
Frame Start
    ↓
Clear Terminal Buffer
    ↓
Render Main Layout
    ├─ Layer tabs
    ├─ Keyboard widget
    └─ Status bar
    ↓
Render Active Component (if any)
    ├─ Clear overlay area
    ├─ Draw component background
    └─ Draw component content
    ↓
Flush Buffer to Terminal
```

### Ratatui Widget System

**Widget Trait:**
- All UI components implement Widget trait
- `render()` method takes buffer and area
- Draws into buffer at specified rectangle

**Layout System:**
- Use ratatui's Layout for splitting screen
- Constraints define size requirements
- Direction: Horizontal or Vertical

**Example Layout Strategy:**
```
Vertical split:
├─ Layer tabs (Fixed 3 lines)
├─ Keyboard area (Proportional, min 20 lines)
└─ Status bar (Fixed 1 line)
```

### Color Handling

**Terminal Color Support:**
- True color (24-bit RGB) preferred
- Fallback to 256 colors if not supported
- Detect capabilities at runtime

**Color Strategy:**
- Use colors meaningfully (function, not decoration)
- Ensure readability (contrast)
- Support monochrome terminals (fallback to bold/dim)

### Performance Optimization

**Event-Driven Updates**
- Only render on events or state changes
- 100ms poll timeout (don't spin CPU)
- Target: 60fps (16ms/frame), typical: event-driven

**Lazy Evaluation**
- Only compute visible elements
- Viewport culling (skip keys outside terminal bounds)
- Early returns for unchanged components

**Caching**
- Cache formatted keycode strings
- Reuse terminal layout calculations (invalidate on resize)
- Pre-allocate fixed-size vectors

---

## Theming System

### OS-Integrated Theme Detection

**Automatic Theme Selection**
- Detects OS dark/light mode preference at startup
- Re-detects on each render loop iteration
- Responds immediately to OS theme changes (no restart)

**Theme Module** (`src/tui/theme.rs`)
- `Theme::detect()` - Query OS and return appropriate theme
- `Theme::dark()` - Light text on dark backgrounds
- `Theme::light()` - Dark text on light backgrounds
- `ThemeVariant` enum - `Dark` or `Light` marker

**Theme Properties**
- `background`, `text`, `text_muted` - Basic colors
- `accent` - Highlights, selections, focus
- `border` - Frames and boxes
- `success`, `error`, `warning` - Status colors

**Color Values**

| Property | Dark Mode | Light Mode |
|----------|-----------|------------|
| background | Black | White |
| text | White | Black |
| text_muted | Gray | DarkGray |
| accent | Yellow | Blue |
| border | Gray | DarkGray |

**Usage Pattern**
1. All UI components receive `Theme` reference
2. Outer blocks use `bg(theme.background)`
3. Inner content blocks also use `bg(theme.background)` (prevent color bleeding)
4. Text spans use `theme.text` or `theme.text_muted`
5. Borders and accents use `theme.border` and `theme.accent`

**Dynamic Detection Loop**

The theme is re-detected on each iteration of the main event loop:
- Main TUI loop (`src/tui/mod.rs` - `run_tui()`)
- Onboarding wizard loop (`src/main.rs` - `run_onboarding_wizard()`)
- Layout picker loop (`src/main.rs` - `run_layout_picker()`)

This allows the app to respond immediately when the user changes their OS theme preference.

---

## File Format & Persistence

### Markdown Layout Format

**Choice Rationale**
- Human-readable and editable
- Version control friendly (plain text, meaningful diffs)
- Standard format (widely supported)
- Easy to parse and generate

**Structure**
```markdown
---
name: "My Layout"
description: "Custom layout for programming"
author: "username"
tags: ["programming", "vim"]
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
is_template: false
version: "1.0"
layout_variant: "LAYOUT_split_3x6_3_ex2"
---

## Layer 0: Base
**Color**: #FF0000
**Category**: typing

| KC_TAB | KC_Q | KC_W{#00FF00} | KC_E@navigation | ... |
|--------|------|---------------|-----------------|-----|
| KC_LCTL| KC_A | KC_S | KC_D | ... |
...

## Layer 1: Lower
...
```

**Key Syntax**
- Plain: `KC_A`
- With color: `KC_A{#FF0000}`
- With category: `KC_A@navigation`
- Combined: `KC_A{#FF0000}@navigation`

**Table Format:**
- First row: Key assignments
- Second row: Separator (dashes)
- Subsequent rows: More keys
- Last row: Thumb keys (if applicable)
- Support for 12-column (standard) or 14-column (with EX keys)

### Configuration Storage

**Location**
- Linux: `~/.config/LazyQMK/config.toml`
- macOS: `~/Library/Application Support/LazyQMK/config.toml`
- Windows: `%APPDATA%\LazyQMK\config.toml`

**Format (TOML)**
```toml
[paths]
qmk_firmware = "/path/to/qmk_firmware"

[build]
output_dir = ".build"

[ui]
theme = "auto"
show_help_on_startup = true
keyboard_scale = 1.0
```

**Note:** Keyboard, layout variant, keymap name, output format, and firmware-specific settings are stored in each layout file's metadata, not in the global config.

### Template Directory

**Location:**
- Linux: `~/.config/LazyQMK/templates/`
- macOS: `~/Library/Application Support/LazyQMK/templates/`
- Windows: `%APPDATA%\LazyQMK\templates\`

**Structure:**
```
templates/
├── qwerty-basic.md
├── colemak-programmer.md
├── gaming-wasd.md
└── [user-created templates]
```

**Template Format:**
- Same as regular layout files
- Additional metadata (is_template = true)
- Well-documented for reuse

### File Operations

**Save Operation:**
1. Check if file path exists
2. Generate markdown from Layout object
3. Write to temporary file
4. Atomic rename to target path
5. Clear dirty flag
6. Update status message

**Load Operation:**
1. Read file contents
2. Parse markdown structure
3. Extract metadata
4. Parse layers and keys
5. Validate keycodes
6. Build Layout object
7. Set source path
8. Clear dirty flag

**Auto-Save Triggers:**
- Before firmware generation
- Before major operations

---

## Parser Architecture

### Parsing Strategy

**Top-Down Recursive Parsing:**
- Start with entire file
- Identify major sections (metadata, layers)
- Parse each section recursively
- Build object hierarchy

**Line-by-Line State Machine:**
- Track current parsing context (in layer, in table, etc.)
- State transitions based on markers (H2 for layer start, etc.)
- Accumulate data until section complete

### Layout Parser

**Phases:**

1. **Metadata Extraction**
   - Detect YAML frontmatter (if present)
   - Parse with YAML parser
   - Extract fields into LayoutMetadata
   - Skip frontmatter lines for layer parsing

2. **Layer Identification**
   - Scan for H2 headings with "Layer N: Name" pattern
   - Extract layer number and name
   - Parse layer properties (color, category)

3. **Table Parsing**
   - Identify markdown table (pipe-delimited rows)
   - Skip separator row (contains dashes)
   - Parse each row into key definitions
   - Handle empty cells (transparent keys)
   - Extract color overrides and categories

4. **Category Parsing**
   - Look for "Categories:" section (optional)
   - Parse list of category definitions
   - Extract name and color

5. **Assembly**
   - Build KeyDefinition objects
   - Build Layer objects
   - Build Category objects
   - Assemble into Layout object

### Table Parser

**Algorithm:**

1. **Split by Pipes**
   - Split line by '|' character
   - Trim whitespace
   - Remove first/last empty elements

2. **Validate Column Count**
   - Check for 12 or 14 columns
   - Error if mismatch

3. **Parse Each Cell**
   - Check for color override: `{#RRGGBB}`
   - Check for category: `@category-id`
   - Extract base keycode
   - Handle empty → KC_TRNS

4. **Position Calculation**
   - Map table column to visual position
   - Handle split keyboard offset
   - Handle thumb row special cases
   - Create Position object

5. **Build KeyDefinition**
   - Set position, keycode, color, category
   - Validate keycode against database
   - Return KeyDefinition object

### QMK Metadata Parser

**Purpose:**
- Load keyboard geometry from QMK's info.json
- Support multiple layouts per keyboard
- Extract matrix mapping
- Build coordinate transformations

**JSON Structure:**
```json
{
  "keyboard_name": "...",
  "manufacturer": "...",
  "layouts": {
    "LAYOUT_split_3x6_3": {
      "layout": [
        {"matrix": [0, 0], "x": 0, "y": 0},
        {"matrix": [0, 1], "x": 1, "y": 0},
        ...
      ]
    }
  }
}
```

**Parsing Steps:**
1. **Load JSON** - Parse file with serde_json
2. **Extract Layouts** - Get layout definitions
3. **Select Layout** - Choose based on config
4. **Build Geometry** - Create KeyGeometry for each key
5. **Build Matrix Mapping** - Create lookup tables
6. **Build Visual Mapping** - Create coordinate transforms

---

## Firmware Integration

### QMK Integration

**Metadata Parsing**
- Parse `info.json` from QMK keyboard directory
- Extract layout definitions (multiple variants supported)
- Build `KeyboardGeometry` from layout data
- Create coordinate mappings

**Code Generation**
- Generate `keymap.c` with PROGMEM arrays
- Generate `config.h` with custom settings
- Layer-aware RGB matrix configuration

**Background Compilation**
- Spawn background thread for QMK make
- Use `mpsc` channels for progress updates
- No shared mutable state (thread-safe)
- Capture stdout/stderr for build log
- Report errors with line numbers

### Build Pipeline

**Stages:**

1. **Validation**
   - Check layout for errors
   - Verify all keycodes are valid
   - Ensure no duplicate layer names
   - Validate matrix coverage

2. **Generation**
   - Generate keymap.c
   - Generate vial.json
   - Generate config files (if needed)
   - Write to output directory

3. **Compilation**
   - Invoke QMK make system
   - Capture stdout/stderr
   - Parse build output
   - Report progress

4. **Post-Build**
   - Locate firmware files
   - Report file sizes
   - Copy to convenient location (optional)
   - Prepare for flashing

### Background Build Process

**Architecture:**
```
Main Thread                 Build Thread
    │                            │
    ├─ Spawn Thread ────────────►│
    │                            │
    │                       ┌────┴────┐
    │                       │ Generate│
    │                       │ Compile │
    │                       │ Report  │
    │                       └────┬────┘
    │                            │
    │◄──── Progress Message ─────┤
    │◄──── Log Output ───────────┤
    │◄──── Complete/Error ───────┤
    │                            │
    └─ Update UI                 ✓
```

**Message Types:**
- **Progress** - Update status text
- **LogOutput** - Add to build log
- **Complete** - Success or error result

**Thread Safety:**
- Use channels (mpsc) for message passing
- No shared mutable state
- Clone necessary data before spawning

---

## Color Management

### Four-Level Priority System

```
1. Individual Key Color Override (highest)
      ↓
2. Key Category Color
      ↓
3. Layer Category Color
      ↓
4. Layer Default Color (lowest)
```

**Resolution Algorithm**
```rust
for each key:
    if key.color_override.is_some():
        return key.color_override (indicator: 'i')
    else if key.category_id.is_some():
        if category exists:
            return category.color (indicator: 'k')
    else if layer.category_id.is_some():
        if category exists:
            return category.color (indicator: 'L')
    return layer.default_color (indicator: 'd')
```

**Visual Indicators**
- Each key displays color source in top-right corner
- 'i' = individual override
- 'k' = key category
- 'L' = layer category
- 'd' = default

### Color Picker Implementation

**RGB Channel Model:**
- Three independent channels: Red, Green, Blue
- Each channel: 0-255 range
- Active channel highlighted
- Tab to switch channels

**Adjustment Controls:**
- Arrow keys: ±1
- Shift + Arrow keys: ±10
- Direct input: Type value
- Hex input: Type hex code (#RRGGBB)

**Visual Feedback:**
- Horizontal slider bar (filled/empty)
- Numeric value display
- Live color preview swatch
- Hex code display

---

## Template System

### Template Purpose

**Use Cases:**
- Save commonly-used layouts
- Share layouts with others
- Quick-start for new keymaps
- Experimentation without affecting main layout

### Template Metadata

**Required Fields:**
- Name (human-readable)
- Description (detailed explanation)
- Author (creator name)

**Optional Fields:**
- Tags (keywords for searching)
- Created date
- Compatible layouts (36/40/42/46 key)

**Example:**
```yaml
---
name: "QWERTY Programming"
description: "Standard QWERTY with programming symbols on layer 1"
author: "username"
tags: ["qwerty", "programming", "symbols"]
is_template: true
compatible_layouts: ["LAYOUT_split_3x6_3", "LAYOUT_split_3x6_3_ex2"]
---
```

### Template Browser

**Features:**
- List all templates in directory
- Display metadata for each
- Search/filter by name or tags
- Preview key information
- Load selected template

### Creating Templates

**Save as Template Dialog:**
- Prompt for name, description, author, tags
- Validate inputs
- Save to templates directory

**Process:**
1. Copy current layout
2. Clear temporary data (source path, etc.)
3. Set is_template = true
4. Add metadata
5. Save to templates directory
6. Show success message

---

## Configuration Management

### Configuration Architecture

**Layered Configuration:**
1. **Defaults** - Hardcoded fallbacks
2. **User Config** - Personal overrides (config.toml)
3. **CLI Arguments** - Command-line overrides

**Configuration File Structure:**

```toml
[paths]
qmk_firmware = "/path/to/qmk_firmware"

[build]
keyboard = "crkbd"
layout = "LAYOUT_split_3x6_3"
keymap = "default"
output_format = "uf2"
output_dir = ".build"

[ui]
theme = "auto"
show_help_on_startup = true
```

### Path Management

**QMK Firmware Path:**
- User-configurable
- Validated on save
- Used for keyboard scanning and compilation

**Discovery Strategy:**
1. Check config file
2. Check environment variable (QMK_HOME)
3. Check common locations (~/qmk_firmware, etc.)
4. Prompt user if not found

**Validation:**
- Check directory exists
- Check for Makefile
- Check for keyboards subdirectory
- Warn if structure looks wrong

### First-Run Experience

**Onboarding Wizard:**
1. **Welcome** - Explain purpose
2. **QMK Path** - Prompt for location, validate
3. **Keyboard** - Scan and select
4. **Layout** - Choose variant
5. **Complete** - Save config, close wizard

---

## Performance Considerations

### Rendering Performance
- Target: 60fps (16ms/frame)
- Typical: Event-driven (100ms poll timeout)
- Only render on events or state changes

### Memory Management
- Pre-allocate fixed-size vectors
- Reuse buffers for rendering
- Stream large files in chunks
- Share immutable data with Rc/Arc

### File I/O
- Buffered reads/writes (BufReader, BufWriter)
- Atomic writes (temp file + rename)
- Incremental parsing where possible

### Background Tasks
- Spawn threads for long operations (compilation)
- Message passing via channels (no shared state)
- Throttle message frequency

### Database Optimization

**Keycode Database:**
- Load once at startup
- Index by category for fast filtering
- Use HashMap for O(1) lookups

---

## Project Structure

```
src/
├── app/
│   ├── launch.rs          # Application launch logic
│   ├── layout_picker.rs   # Layout selection UI
│   └── onboarding.rs      # First-run wizard
├── models/
│   ├── layout.rs          # Layout, Layer, KeyDefinition
│   ├── category.rs        # Category system
│   ├── rgb.rs             # Color handling
│   ├── keyboard_geometry.rs        # Physical key positions
│   ├── matrix_mapping.rs           # Electrical wiring
│   └── visual_layout_mapping.rs    # Coordinate transforms
├── parser/
│   ├── layout.rs          # Main layout parser
│   ├── keyboard_json.rs   # QMK metadata parsing
│   ├── template_gen.rs    # Template generation
│   └── [others]           # Vial, templates, etc.
├── tui/
│   ├── mod.rs             # Main TUI loop, AppState
│   ├── theme.rs           # OS theme detection
│   ├── handlers/          # Input handling (refactored structure)
│   │   ├── action_handlers/ # Action-specific handlers (12 modules)
│   │   ├── actions.rs     # Main dispatcher
│   │   ├── popups.rs      # Popup routing
│   │   ├── category.rs    # Category operations
│   │   ├── layer.rs       # Layer operations
│   │   ├── settings.rs    # Settings dialog
│   │   ├── templates.rs   # Template operations
│   │   └── main.rs        # Handler coordination
│   ├── keyboard.rs        # Keyboard widget
│   ├── keycode_picker.rs  # Keycode selection dialog
│   ├── color_picker.rs    # Color selection dialog
│   ├── category_manager.rs # Category CRUD
│   ├── build_log.rs       # Build output viewer
│   ├── help_overlay.rs    # Help documentation
│   └── [22+ other components]
├── keycode_db/
│   ├── categories.json    # Keycode database
│   └── categories/        # 20 category files
├── firmware/
│   ├── generator.rs       # Generate keymap.c
│   ├── builder.rs         # Background compilation
│   └── validator.rs       # Layout validation
├── services/
│   ├── geometry.rs        # Geometry loading
│   └── layouts.rs         # Layout services
├── config.rs              # Configuration management
├── constants.rs           # App constants
└── main.rs                # Entry point
```

---

## Future Architectural Considerations

### Message-Based Architecture (Spec 017 Phase C.2 - Optional)
- Replace direct state mutation with message passing
- `AppState` receives messages, updates itself
- Components emit messages, don't modify state directly
- Benefits: better testing, clearer data flow, easier debugging

### Plugin System
- Custom keycode parsers
- Custom code generators
- Custom color schemes
- External tool integration

### Multi-Keyboard Profiles
- Switch between different keyboards
- Shared templates across keyboards
- Per-keyboard settings

### Advanced Features
- Copy/paste keys
- Undo/redo support
- Multi-key selection
- Tap dance editor
- Combo editor
- Macro editor
- Layout analyzer with efficiency metrics

---

## References

- [FEATURES.md](FEATURES.md) - Complete feature documentation
- [QUICKSTART.md](../QUICKSTART.md) - User guide
- [README.md](../README.md) - Project overview
- [specs/archived/017-tui-architecture-refactor/](../specs/archived/017-tui-architecture-refactor/) - Component refactoring history
