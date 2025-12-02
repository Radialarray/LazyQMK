# Keyboard Layout Editor TUI - Complete Architecture Guide

## Executive Summary

This document provides a comprehensive guide to building a Terminal User Interface (TUI) application for editing mechanical keyboard layouts. The application supports visual keyboard rendering, real-time editing, color management, category systems, template management, and firmware generation/compilation. This guide focuses on architecture, design patterns, and technical implementation strategies without code examples.

---

## Table of Contents

1. [Overview & Features](#overview--features)
2. [Tech Stack](#tech-stack)
3. [Core Architecture](#core-architecture)
4. [Coordinate System Design](#coordinate-system-design)
5. [Data Models](#data-models)
6. [State Management](#state-management)
7. [User Interface Components](#user-interface-components)
8. [Input Handling](#input-handling)
9. [Rendering System](#rendering-system)
10. [File I/O & Persistence](#file-io--persistence)
11. [Parser Architecture](#parser-architecture)
12. [Build System Integration](#build-system-integration)
13. [Configuration Management](#configuration-management)
14. [Template System](#template-system)
15. [Category & Color System](#category--color-system)
16. [Performance Considerations](#performance-considerations)
17. [Testing Strategy](#testing-strategy)
18. [Future Extensions](#future-extensions)

---

## 1. Overview & Features

### Application Purpose
A terminal-based keyboard layout editor that allows users to:
- Visually edit mechanical keyboard layouts
- Assign keycodes to physical key positions
- Manage colors and visual organization
- Generate and compile firmware
- Save/load layouts as human-readable markdown
- Use templates for common layouts

### Core Features

#### Visual Editing
- Real-time keyboard visualization with accurate physical positioning
- Visual cursor navigation between keys
- Color-coded keys based on function/category
- Multiple layer support with tab-based navigation
- Support for split keyboards (e.g., Corne, Ergodox)
- Support for various layouts (36/40/42/46 keys)

#### Key Assignment
- Searchable keycode picker with fuzzy matching
- Category-based keycode organization (Basic, Navigation, Symbols, etc.)
- Support for 600+ QMK keycodes
- Quick clear/reset functions
- Per-key or bulk assignment

#### Color Management
- RGB color picker with channel-based adjustment
- Hex color code input
- Four-level color priority system:
  1. Individual key color override (highest)
  2. Key category color
  3. Layer category color
  4. Layer default color (lowest)
- Visual indicators showing color source

#### Organization System
- User-defined categories for grouping keys by function
- Category assignment to individual keys or entire layers
- Category manager for CRUD operations
- Color coding for visual recognition

#### Template System
- Save layouts as reusable templates
- Template browser with metadata (name, description, author, tags)
- Templates stored in user config directory
- Searchable template library

#### File Management
- Markdown-based file format (human-readable)
- Support for 12-column (standard) and 14-column (with EX keys) layouts
- Metadata embedded in markdown (YAML frontmatter or comments)
- Auto-save on major operations
- Dirty flag tracking for unsaved changes

#### Firmware Integration
- Generate QMK firmware C code from layouts
- Generate Vial JSON configuration
- Background compilation with progress tracking
- Build log with scrolling and copy functionality
- Multiple output formats (UF2, HEX, BIN)
- Configurable keyboard/keymap targets

#### Configuration
- Keyboard picker (scan QMK directory)
- Layout selector (switch between key counts)
- QMK firmware path configuration
- Output format selection
- Persistent configuration in TOML format

#### Onboarding
- First-run setup wizard
- Step-by-step configuration guide
- Path validation and keyboard detection

#### Help System
- Comprehensive help overlay with all shortcuts
- Scrollable documentation
- Context-sensitive help hints
- Status bar with current mode indicators

---

## 2. Tech Stack

### Primary Framework
**Ratatui 0.26** - Rust TUI framework
- Immediate mode rendering
- Widget-based architecture
- Buffer-based drawing system
- Cross-platform terminal support

### Terminal Backend
**Crossterm 0.27** - Terminal manipulation library
- Event handling (keyboard, mouse, resize)
- Raw mode terminal control
- Alternate screen buffer
- Cross-platform (Windows, Unix, macOS)

### Data Handling
**Serde 1.0** - Serialization framework
- JSON parsing for QMK metadata
- YAML for frontmatter
- TOML for configuration files
- Custom derive macros for models

### File Formats
**serde_json 1.0** - JSON parsing
**serde_yaml 0.9** - YAML parsing
**toml 0.8** - TOML configuration

### Parsing
**regex 1.0** - Pattern matching for markdown parsing
**pulldown-cmark 0.9** - Markdown processing (future use)

### System Integration
**dirs 5.0** - Cross-platform directory paths
**arboard 3.0** - Clipboard integration
**chrono 0.4** - Timestamp handling

### Error Handling
**anyhow 1.0** - Flexible error handling
- Context-aware error messages
- Error propagation with question mark operator
- User-friendly error reporting

### CLI Framework
**clap 4.0** - Command-line argument parsing
- Subcommands for different operations
- Environment variable integration
- Auto-generated help messages

---

## 2.1 Theming System

### OS-Integrated Theme Detection

The application automatically detects the operating system's dark/light mode preference and applies the appropriate color scheme. This ensures visual consistency with the user's terminal environment.

**Detection Strategy:**
- Uses the `dark-light` crate (v2.0) for cross-platform theme detection
- Queries OS settings at startup and on each render loop iteration
- Dynamically responds to theme changes while running (no restart required)

**Implementation:**

**Theme Module (`src/tui/theme.rs`):**
- `Theme::detect()` - Main entry point, queries OS and returns appropriate theme
- `Theme::dark()` - Dark mode colors (light text on dark backgrounds)
- `Theme::light()` - Light mode colors (dark text on light backgrounds)
- `ThemeVariant` enum - `Dark` or `Light` variant marker

**Theme Properties:**
- `background` - Main application background color
- `text` - Primary text color  
- `text_muted` - Secondary/dimmed text color
- `accent` - Highlight color for selections, focus
- `border` - Border color for frames and boxes
- `success`, `error`, `warning` - Status colors

**Color Values:**

| Property | Dark Mode | Light Mode |
|----------|-----------|------------|
| background | Black | White |
| text | White | Black |
| text_muted | Gray | DarkGray |
| accent | Yellow | Blue |
| border | Gray | DarkGray |

**Usage Pattern:**

All UI components receive a `Theme` reference and apply colors consistently:
1. Outer `Block` widgets use `bg(theme.background)` for backgrounds
2. Inner content blocks also apply `bg(theme.background)` to prevent color bleeding
3. Text spans use `theme.text` or `theme.text_muted` for foreground colors
4. Borders and accents use `theme.border` and `theme.accent`

**Dynamic Detection Loop:**

The theme is re-detected on each iteration of the main event loop and sub-loops (wizard, picker):
- Main TUI loop (`src/tui/mod.rs` - `run_tui()`)
- Onboarding wizard loop (`src/main.rs` - `run_onboarding_wizard()`)
- Layout picker loop (`src/main.rs` - `run_layout_picker()`)

This allows the app to respond immediately when the user changes their OS theme preference.

**Why Not Manual Toggle:**

Earlier versions included an F12 toggle for theme switching, but this was removed because:
1. Two independent theme systems (OS + app) caused color conflicts
2. Terminal background colors are controlled by the OS/terminal, not the app
3. Automatic detection provides a seamless, consistent experience
4. Removes configuration burden from the user

---

## 3. Core Architecture

### Architectural Pattern
**Model-View-Controller (MVC) with State Management**

The application follows a clear separation of concerns:

1. **Models** - Data structures representing layouts, keys, colors, categories
2. **Views** - UI components (widgets) that render state
3. **Controller** - Event loop that processes input and updates state

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
       ├──► Poll Event │
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

### Module Organization

```
layout_tools/
├── src/
│   ├── main.rs              # Entry point, CLI parsing
│   ├── config.rs            # Configuration management
│   ├── models/              # Data structures
│   │   ├── layout.rs        # Layout, Layer, KeyDefinition
│   │   ├── category.rs      # Category system
│   │   ├── rgb.rs           # Color handling
│   │   ├── keyboard_geometry.rs  # Physical key positions
│   │   ├── matrix_mapping.rs     # Electrical matrix mapping
│   │   └── visual_layout_mapping.rs  # Coordinate transformations
│   ├── parser/              # File parsing and generation
│   │   ├── layout.rs        # Main layout parser
│   │   ├── layer.rs         # Layer parsing
│   │   ├── table.rs         # Markdown table parsing
│   │   ├── keyboard_json.rs # QMK metadata parsing
│   │   ├── template_gen.rs  # Template generation
│   │   └── vial_config.rs   # Vial JSON parsing
│   ├── tui/                 # UI components
│   │   ├── mod.rs           # Main TUI loop and state
│   │   ├── keyboard.rs      # Keyboard widget
│   │   ├── keycode_picker.rs
│   │   ├── color_picker.rs
│   │   ├── category_picker.rs
│   │   ├── category_manager.rs
│   │   ├── template_browser.rs
│   │   ├── help_overlay.rs
│   │   ├── build_log.rs
│   │   ├── onboarding_wizard.rs
│   │   └── [other dialogs]
│   └── keycode_db/          # Keycode database
└── Cargo.toml
```

### State Management Pattern

**Centralized State Object** (`AppState`)
- Single source of truth for entire application
- All UI components read from state
- All user actions modify state
- State is mutable and owned by main loop

**State Updates Flow:**
1. User input event received
2. Event handler determines action
3. Action modifies AppState
4. Dirty flag set if data changed
5. Next render cycle reads updated state

This pattern ensures:
- Predictable state updates
- Easy debugging (single state object)
- Clear data flow
- No synchronization issues

---

## 4. Coordinate System Design

### The Three-Coordinate Problem

Mechanical keyboards require managing three different coordinate systems:

#### 1. **Matrix Coordinates** (Electrical Wiring)
- How the keyboard is physically wired
- Row/column electrical connections
- Example: Corne has 8 rows × 7 columns
  - Rows 0-3: Left half
  - Rows 4-7: Right half (mirrored)
- Used by firmware for scanning keys

#### 2. **LED Index** (Sequential Order)
- Order in which RGB LEDs are wired
- Sequential numbering (0, 1, 2, ...)
- Often follows a zigzag pattern
- Used for RGB lighting control

#### 3. **Visual Position** (User's Mental Model)
- How keys appear in the markdown table
- Logical grid positions
- Split into left/right halves
- Used for user interface and editing

### Coordinate Mapping Architecture

**VisualLayoutMapping** class handles all transformations:

```
LED Index ←→ Matrix Position ←→ Visual Position
```

**Key Methods:**
- `led_to_matrix_pos()` - Convert LED index to matrix coordinates
- `matrix_to_visual_pos()` - Convert matrix to visual coordinates
- `visual_to_matrix_pos()` - Convert visual to matrix coordinates
- `led_to_visual_pos()` - Direct LED to visual (convenience)
- `visual_to_led_index()` - Direct visual to LED (convenience)

### Split Keyboard Mapping

For split keyboards like Corne:

**Matrix Layout:**
- Rows 0-3, Cols 0-6: Left half
- Rows 4-7, Cols 0-6: Right half

**Visual Layout:**
- Rows 0-3, Cols 0-6: Left half
- Rows 0-3, Cols 7-13: Right half (mapped from matrix rows 4-7)

**Special Handling:**
- Right side columns are often **reversed** (col 0 = rightmost physically)
- EX keys (extra keys) have special column positions
- Thumb keys span multiple visual columns but limited matrix positions

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

## 5. Data Models

### Layout Hierarchy

```
Layout (entire keymap)
├── metadata: LayoutMetadata
├── categories: Vec<Category>
└── layers: Vec<Layer>
    ├── number: u8
    ├── name: String
    ├── default_color: RgbColor
    ├── category_id: Option<String>
    └── keys: Vec<KeyDefinition>
        ├── position: Position
        ├── keycode: String
        ├── label: Option<String>
        ├── color_override: Option<RgbColor>
        ├── category_id: Option<String>
        └── combo_participant: bool
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
- Used for quick lookups during rendering
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

### Configuration Models

**Config**
- `paths`: PathConfig
  - `qmk_firmware`: Optional path to QMK directory
- `build`: BuildConfig
  - `keyboard`: Target keyboard
  - `layout`: Selected layout variant
  - `keymap`: Keymap name
  - `output_format`: UF2/HEX/BIN
  - `output_dir`: Build directory

**Persistence:**
- Stored in `~/.config/layout_tools/config.toml`
- Loaded at startup
- Updated via UI dialogs

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
The `layout_variant` field stores the user's selected QMK layout variant in the markdown
frontmatter. This enables:
- Proper geometry restoration when loading a saved layout
- Correct RGB matrix lookup for layer-aware coloring
- Automatic detection of keyboard variant (36/40/42/46 keys)

The variant is saved when the user switches layouts via Ctrl+Y, and is used during
layout loading to:
1. Extract the base keyboard name using `extract_base_keyboard()`
2. Determine the correct keyboard variant path based on key count
3. Load the appropriate `KeyboardGeometry` and `VisualLayoutMapping`
4. Adjust layer key positions to match the visual geometry

---

## 6. State Management

### AppState Structure

The central state object contains:

**Core Data:**
- `layout`: Current layout being edited
- `source_path`: File path
- `dirty`: Unsaved changes flag

**UI State:**
- `current_layer`: Active layer index
- `selected_position`: Cursor position
- `active_popup`: Current dialog/picker
- `status_message`: Bottom status text

**Component States:**
- `keycode_picker_state`: Search, selection, category
- `color_picker_state`: RGB values, active channel
- `category_picker_state`: Category list, selection
- `category_manager_state`: CRUD operations
- `template_browser_state`: Template list, metadata
- `help_overlay_state`: Scroll position
- `build_log_state`: Log lines, scroll position
- `onboarding_state`: Wizard step, inputs

**System State:**
- `keycode_db`: Database of valid keycodes
- `keyboard_geometry`: Physical key positions
- `matrix_mapping`: Electrical wiring
- `visual_layout_mapping`: Coordinate transforms
- `config`: User configuration

**Build State:**
- `build_state`: Idle/Validating/Compiling/Success/Failed
- `build_receiver`: Message channel from background thread
- `build_log_state`: Build output capture

### State Lifetime Management

**Initialization:**
1. Parse command-line arguments
2. Load configuration from disk
3. Parse markdown layout file
4. Load keyboard geometry from QMK
5. Build coordinate mappings
6. Initialize UI component states
7. Enter main loop

**Main Loop:**
```
loop {
    1. Poll for events (timeout: 100ms)
    2. Handle event → update state
    3. Check background channels (build progress)
    4. Render UI from current state
    5. If should_quit, break loop
}
```

**Termination:**
1. Check dirty flag
2. Prompt for save if needed (double Ctrl+Q)
3. Cleanup terminal (restore normal mode)
4. Exit process

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

## 7. User Interface Components

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

**Space Management:**
- Calculate key width from geometry
- Ensure minimum width (3 chars) and height (3 lines)
- Skip keys that don't fit in terminal
- Scale based on terminal size

### Layer Tabs Widget

**Layout:**
- Horizontal row of tabs at top
- Active tab highlighted
- Shows layer number and name
- Dirty indicator (*) if modified

**Interaction:**
- Tab/Shift+Tab to cycle
- Click to select (mouse support)

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

**Search Algorithm:**
- Case-insensitive substring matching
- Match against keycode and description
- Real-time filtering as user types

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

**Channel Selection:**
- Active channel highlighted
- Tab/Shift+Tab to switch
- Arrow keys adjust value
- Shift modifier for larger steps

### Category Picker Dialog

**Layout:**
- List of available categories
- Color preview for each
- "None" option to unassign
- Keyboard navigation

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

### Template Browser Dialog

**Layout:**
- List of saved templates
- Metadata preview (name, description, author)
- Tags display
- Enter to load, Esc to cancel

**Features:**
- Scan templates directory
- Parse metadata from markdown frontmatter
- Sort by name or date
- Preview key information

### Help Overlay

**Layout:**
- Centered modal overlay (60% width, 80% height)
- Scrollable content
- Organized by category
- Color-coded sections

**Content:**
- Navigation shortcuts
- Editing shortcuts
- File operations
- Build commands
- Picker controls
- System commands

**Scroll Support:**
- Arrow keys to scroll
- Home/End to jump
- Page Up/Down for faster scrolling

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

### Onboarding Wizard

**Multi-step workflow:**
1. Welcome screen
2. QMK path configuration
3. Keyboard selection
4. Layout selection
5. Completion summary

**Features:**
- Step indicators
- Input validation
- Path verification
- Automatic keyboard scanning
- Save configuration on completion

---

## 8. Input Handling

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

**Principles:**
- Ctrl for major operations (save, quit, build)
- Shift for variations (Shift+C = key color, C = layer color)
- Single letters for common actions (x = clear, c = color)
- Arrow keys for navigation
- Enter for confirm, Esc for cancel
- ? for help

**Shortcut Categories:**

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
- `Ctrl+E` - Edit metadata
- `Ctrl+V` - Validate
- `Ctrl+W` - Setup wizard

**Templates:**
- `t` - Load template
- `Shift+T` - Save as template

**Build:**
- `Ctrl+G` - Generate firmware
- `Ctrl+B` - Build firmware
- `Ctrl+F` - Flash firmware
- `Ctrl+L` - View build log

**Configuration:**
- `Ctrl+P` - Set QMK path
- `Ctrl+K` - Select keyboard
- `Ctrl+Y` - Select layout
- `Ctrl+U` - Configure output format
- `Ctrl+O` - Set output directory
- `Ctrl+M` - Set keymap name

**System:**
- `Ctrl+T` - Category manager
- `?` - Help overlay
- `Ctrl+Q` - Quit (twice if dirty)

### Input Validation

**Strategies:**
1. **Bounds Checking** - Ensure cursor stays within valid positions
2. **State Validation** - Only allow operations in valid states
3. **Path Validation** - Verify file/directory existence
4. **Keycode Validation** - Check against keycode database
5. **Color Validation** - Ensure RGB values in 0-255 range

### Text Input Handling

**For Dialogs with Text Fields:**
- Capture character input
- Handle backspace/delete
- Support basic editing (no advanced features like cursor movement)
- Tab to switch fields
- Enter to submit
- Escape to cancel

**Implementation Pattern:**
- Maintain string buffer in state
- Append characters on keypress
- Pop characters on backspace
- Display with cursor indicator

---

## 9. Rendering System

### Immediate Mode Rendering

**Concept:**
- Entire UI rebuilt every frame
- No retained UI state between frames
- State drives rendering, not vice versa

**Advantages:**
- Simple mental model
- No synchronization issues
- Easy to reason about
- State is single source of truth

**Rendering Pipeline:**

```
Frame Start
    │
    ├─ Clear Terminal Buffer
    │
    ├─ Render Main Layout
    │   ├─ Layer tabs
    │   ├─ Keyboard widget
    │   └─ Status bar
    │
    ├─ Render Active Popup (if any)
    │   ├─ Clear overlay area
    │   ├─ Draw popup background
    │   └─ Draw popup content
    │
    └─ Flush Buffer to Terminal
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

### Drawing Primitives

**Text:**
- Single-line text with styles
- Multi-line paragraphs
- Word wrapping (if needed)

**Styles:**
- Foreground color
- Background color
- Modifiers (bold, italic, underline, dim)

**Borders:**
- Block widget with border styles
- Unicode box-drawing characters
- Optional title in border

**Lists:**
- Vertical list of items
- Selection highlighting
- Scrolling support

**Tables:**
- Grid layout
- Column widths
- Row selection

### Color Handling

**Terminal Color Support:**
- True color (24-bit RGB) preferred
- Fallback to 256 colors if not supported
- Detect capabilities at runtime

**Color Conversion:**
- Store colors as RGB (0-255 per channel)
- Convert to terminal color format
- Cache conversions if needed

**Color Strategy:**
- Use colors meaningfully (function, not decoration)
- Ensure readability (contrast)
- Provide color-blind friendly alternatives
- Support monochrome terminals (fallback to bold/dim)

### Performance Optimization

**Strategies:**
1. **Lazy Evaluation** - Only compute what's visible
2. **Caching** - Store expensive computations
3. **Early Return** - Skip rendering if nothing changed
4. **Clipping** - Don't render out-of-bounds content
5. **Throttling** - Limit render rate (tied to event loop)

**Rendering Budget:**
- Target: 60fps (16ms per frame)
- Typical: Event-driven rendering (100ms poll timeout)
- Only render on events or state changes

### Terminal Compatibility

**Cross-Platform Considerations:**
- Test on multiple terminals (iTerm2, Terminal.app, Windows Terminal, alacritty)
- Handle different Unicode support levels
- Graceful degradation for limited features
- Alternate buffer to avoid scrollback pollution

**Size Constraints:**
- Minimum size requirement (80x24 recommended)
- Display warning if too small
- Scale UI elements proportionally
- Handle resize events gracefully

---

## 10. File I/O & Persistence

### Markdown File Format

**Choice Rationale:**
- Human-readable and editable
- Version control friendly (plain text)
- Standard format (widely supported)
- Easy to parse and generate
- Allows comments and documentation

**File Structure:**

```
# Layout Title

[Optional YAML frontmatter for metadata]

## Layer 0: Base
**Color**: #FF0000

| KC_TAB | KC_Q | ... | KC_BSPC |
|--------|------|-----|---------|
| KC_LCTL| KC_A | ... | KC_QUOT |
| KC_LSFT| KC_Z | ... | KC_ESC  |
|        |      | ... |         |

## Layer 1: Lower
**Color**: #00FF00

[similar table structure]

---

Categories:
- navigation (#00FF00)
- symbols (#FF0000)
```

**Key Assignment Syntax:**
- Plain keycode: `KC_A`
- With color override: `KC_A{#FF0000}`
- With category: `KC_A@navigation`
- Combined: `KC_A{#FF0000}@navigation`

**Layer Structure:**
- H2 heading with layer number and name
- Bold text for default color in hex
- Optional category assignment
- Markdown table with key grid

**Table Format:**
- First row: Key assignments
- Second row: Separator (dashes)
- Subsequent rows: More keys
- Last row: Thumb keys (if applicable)
- Support for 12-column (standard) or 14-column (with EX keys)

### Metadata Handling

**Metadata Location:**
- YAML frontmatter at file start, OR
- Markdown comments with structured data

**Example Frontmatter:**
```yaml
---
name: "My Custom Layout"
description: "Optimized for programming"
author: "username"
tags: ["programming", "vim", "productivity"]
created: "2024-01-15T10:30:00Z"
modified: "2024-01-20T15:45:00Z"
is_template: false
version: "1.0"
layout_variant: "LAYOUT_split_3x6_3_ex2"
---
```

**Fields:**
- Name, description, author
- Creation and modification timestamps
- Tags for searchability
- Template flag
- Schema version for future compatibility
- Layout variant for geometry restoration

### Configuration Files

**Location:**
- `~/.config/layout_tools/config.toml` (Unix/macOS)
- `%APPDATA%\layout_tools\config.toml` (Windows)

**TOML Format:**
```toml
[paths]
qmk_firmware = "/path/to/qmk_firmware"

[build]
keyboard = "keebart/corne_choc_pro/mini"
layout = "LAYOUT_split_3x6_3_ex2"
keymap = "keebart"
output_format = "uf2"
output_dir = ".build"
```

**Persistence Strategy:**
- Load at startup (with defaults if missing)
- Save on configuration changes
- Atomic writes (temp file + rename)

### Template Directory

**Location:**
- `~/.config/layout_tools/templates/` (Unix/macOS)
- `%APPDATA%\layout_tools\templates\` (Windows)

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
- User-configurable interval (optional feature)

### Error Handling

**File Errors:**
- File not found → Show error, offer to create
- Permission denied → Show error with instructions
- Corrupt file → Show specific parse error, line number
- Invalid keycode → List invalid codes, allow correction

**Recovery Strategy:**
- Keep backup before overwriting
- Transaction-style saves (temp + rename)
- Validation before writing
- Clear error messages with actionable steps

---

## 11. Parser Architecture

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

**Challenges:**
- Variable column count (12 vs 14)
- Empty cells (corner positions, thumb row)
- Color syntax in cells
- Category syntax in cells

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

### Color Parser

**Hex Color Syntax:** `#RRGGBB`

**Algorithm:**
1. Strip '#' prefix
2. Validate 6 hex digits
3. Parse each pair as hex byte
4. Build RgbColor(r, g, b)

**Error Handling:**
- Invalid format → descriptive error
- Out-of-range values → clamp to 0-255
- Missing '#' → add automatically (lenient)

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
    },
    "LAYOUT_split_3x6_3_ex2": { ... }
  }
}
```

**Parsing Steps:**
1. **Load JSON** - Parse file with serde_json
2. **Extract Layouts** - Get layout definitions
3. **Select Layout** - Choose based on config
4. **Build Geometry** - Create KeyGeometry for each key
   - Extract matrix position
   - Extract visual position (x, y)
   - Assign sequential LED index
5. **Build Matrix Mapping** - Create lookup tables
6. **Build Visual Mapping** - Create coordinate transforms

### Template Generator

**Purpose:**
- Generate blank markdown layouts
- Use keyboard geometry as source
- Create tables with correct dimensions

**Algorithm:**
1. **Load Geometry** - Get KeyboardGeometry for layout
2. **Determine Dimensions** - Count rows and columns
3. **Generate Tables** - Create markdown tables
   - Determine split point (left/right halves)
   - Handle thumb row separately
   - Insert empty cells for placeholders
4. **Generate Layers** - Repeat for N layers
5. **Add Metadata** - Insert frontmatter template
6. **Write Output** - Format as markdown string

### Firmware Generator

**Purpose:**
- Convert Layout to QMK C code
- Generate keymap.c file
- Generate vial.json file

**C Code Generation:**

1. **Header** - Includes and defines
2. **Keymap Array** - 2D array of keycodes
   ```c
   const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
       [0] = LAYOUT_split_3x6_3(
           KC_TAB, KC_Q, ...,
           ...
       ),
       ...
   };
   ```
3. **RGB Lighting** - LED color configuration (if applicable)
4. **Combos** - Key combination definitions (if used)

**Vial JSON Generation:**

1. **Layout Definition** - Physical key positions
2. **Key Mappings** - Assign keycodes to positions
3. **Layer Colors** - RGB values for each layer
4. **Metadata** - Keyboard info, layout name

**Coordinate Mapping:**
- Convert visual positions back to matrix positions
- Use VisualLayoutMapping in reverse
- Handle split keyboard peculiarities
- Validate all positions have keys

---

## 12. Build System Integration

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

**Challenge:**
- QMK compilation takes 30-60 seconds
- Don't block UI during build
- Show progress updates
- Allow cancellation

**Solution: Background Thread with Message Passing**

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

**Progress Reporting:**
- Parse QMK output for milestones
- Extract percentage if available
- Update UI with status messages
- Stream log output in real-time

### QMK Integration

**Invoking Make:**
```bash
cd $QMK_FIRMWARE_PATH
make crkbd:keebart
```

**Environment:**
- Set current directory to QMK root
- Use system PATH to find toolchain
- Capture output (stdout + stderr)
- Monitor exit code

**Output Parsing:**
- Detect compilation errors
- Extract error line numbers
- Show in build log
- Highlight errors in red

**Build Products:**
- Locate .uf2 or .hex file
- Report to user
- Display file size
- Copy to convenient location if requested

### Flashing Support

**Flashing Strategies:**
1. **UF2 Bootloader** - Drag-and-drop to mounted drive
2. **QMK Flash** - Use qmk flash command
3. **DFU-Util** - Direct USB flashing
4. **Manual** - Provide instructions

**Implementation:**
- Detect bootloader type (if possible)
- Provide appropriate instructions
- Auto-copy UF2 to mounted volume
- Show step-by-step guide

**Safety:**
- Confirm before flashing
- Validate firmware file exists
- Check file size (sanity check)
- Provide recovery instructions

---

## 13. Configuration Management

### Configuration Architecture

**Layered Configuration:**
1. **Defaults** - Hardcoded fallbacks
2. **System Config** - Global settings
3. **User Config** - Personal overrides
4. **Project Config** - Per-layout settings (future)
5. **CLI Arguments** - Command-line overrides

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
theme = "default"
show_help_on_startup = true
```

### Path Management

**QMK Firmware Path:**
- User-configurable
- Validated on save
- Used for keyboard scanning
- Used for compilation

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

### Keyboard Selection

**Keyboard Picker:**
- Scan QMK keyboards directory
- List all available keyboards
- Show manufacturer and name
- Filter by search term

**Layout Selection:**
- Read info.json for keyboard
- List available layout variants
- Show key count for each
- Allow runtime switching

**Persistence:**
- Save selected keyboard to config
- Save selected layout to config
- Automatically reload geometry
- Update UI without restart

### Build Configuration

**Output Format:**
- UF2 (RP2040 bootloader)
- HEX (AVR chips)
- BIN (ARM chips)

**Keymap Name:**
- Defaults to "default"
- User-customizable
- Used in QMK make command
- Affects output directory

**Output Directory:**
- Relative to QMK firmware root
- Defaults to keyboards/{keyboard}/keymaps/{keymap}
- Can override with absolute path

### First-Run Experience

**Onboarding Wizard:**
1. **Welcome** - Explain purpose
2. **QMK Path** - Prompt for location, validate
3. **Keyboard** - Scan and select
4. **Layout** - Choose variant
5. **Complete** - Save config, close wizard

**Validation:**
- Verify each step before proceeding
- Show helpful error messages
- Allow back navigation
- Skip wizard if config exists

---

## 14. Template System

### Template Purpose

**Use Cases:**
- Save commonly-used layouts
- Share layouts with others
- Quick-start for new keymaps
- Experimentation without affecting main layout

### Template Storage

**Location:**
- User config directory: `~/.config/layout_tools/templates/`
- System templates: `<install_path>/templates/` (future)

**File Format:**
- Standard markdown layout files
- Enhanced metadata (is_template = true)
- Descriptive tags for searching

### Template Metadata

**Required Fields:**
- Name (human-readable)
- Description (detailed explanation)
- Author (creator name)

**Optional Fields:**
- Tags (keywords for searching)
- Created date
- Compatible layouts (36/40/42/46 key)
- Recommended categories

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

**UI Design:**
- Scrollable list
- Selection highlighting
- Metadata panel
- Keyboard navigation

**Sorting Options:**
- Alphabetical by name
- By creation date
- By tag match (if searching)

### Creating Templates

**Save as Template Dialog:**
- Prompt for name
- Prompt for description
- Prompt for author
- Prompt for tags (comma-separated)
- Validate inputs
- Save to templates directory

**Process:**
1. Copy current layout
2. Clear temporary data (source path, etc.)
3. Set is_template = true
4. Add metadata
5. Save to templates directory
6. Show success message

### Loading Templates

**Process:**
1. User selects template from browser
2. Parse template file
3. Validate layout structure
4. Load into current session
5. Set dirty flag
6. Update source path to current file (don't overwrite template)
7. Prompt to save as new file

**Conflict Handling:**
- Warn if current layout has unsaved changes
- Require explicit confirmation
- Option to save before loading

---

## 15. Category & Color System

### Category System Design

**Purpose:**
- Organize keys by logical function
- Apply consistent colors to related keys
- Aid visual memory and learning
- Simplify bulk operations

**Category Definition:**
- Unique ID (kebab-case slug)
- Display name (human-readable)
- RGB color

**Example Categories:**
- navigation (arrow keys, home/end, etc.)
- symbols (punctuation, brackets, etc.)
- numbers (0-9, numpad)
- function (F1-F12)
- media (play, pause, volume)
- modifiers (shift, ctrl, alt, gui)

### Color Priority System

**Four Levels (Highest to Lowest):**

1. **Individual Key Color Override**
   - Manually set per key
   - Indicator: 'i'
   - Takes precedence over all else
   - Use for special keys or exceptions

2. **Key Category Color**
   - Color from assigned category
   - Indicator: 'k'
   - Affects single key
   - Use for functional grouping

3. **Layer Category Color**
   - Color from layer's assigned category
   - Indicator: 'L'
   - Affects all keys on layer
   - Use for layer-wide themes

4. **Layer Default Color**
   - Base color for layer
   - Indicator: 'd'
   - Fallback if nothing else set
   - Use for overall color scheme

**Resolution Algorithm:**
```
for each key:
    if key has color_override:
        return color_override
    else if key has category_id:
        if category exists:
            return category.color
    else if layer has category_id:
        if category exists:
            return category.color
    return layer.default_color
```

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

**Special Operations:**
- Clear key color (keys only, not layers)
- Reset to default
- Copy hex code to clipboard

### Category Manager

**CRUD Operations:**

**Create:**
- Prompt for name
- Generate ID from name (slugify)
- Prompt for color (open color picker)
- Add to category list

**Read/List:**
- Display all categories
- Show color preview
- Navigate with arrow keys

**Update:**
- Rename: Prompt for new name
- Change Color: Open color picker
- Update references in keys/layers

**Delete:**
- Confirm deletion
- Remove category from all key assignments
- Remove category from all layer assignments
- Delete from category list

**Validation:**
- Ensure unique IDs
- Prevent deletion of in-use categories (or reassign first)
- Validate color values

### Category Assignment

**To Single Key:**
- Select key with cursor
- Press Shift+K
- Choose from category picker
- Key inherits category color

**To Entire Layer:**
- Press Shift+L
- Choose from category picker
- All keys on layer inherit color (unless overridden)

**Unassignment:**
- Select "None" in category picker
- Removes category reference
- Falls back to next priority level

### Color Indicator System

**Visual Representation:**
- Small character in key's top-right corner
- 'i' = individual override
- 'k' = key category
- 'L' = layer category
- 'd' = default

**Purpose:**
- Quick identification of color source
- Debugging color conflicts
- Understanding color hierarchy
- Educational tool for users

---

## 16. Performance Considerations

### Rendering Performance

**Target:**
- 60fps ideal (16ms per frame)
- 30fps acceptable (33ms per frame)
- Event-driven: Only render on events

**Optimization Strategies:**

1. **Lazy Rendering**
   - Only render when state changes
   - Track dirty flag per component
   - Skip unchanged components

2. **Viewport Culling**
   - Only render visible keys
   - Skip keys outside terminal bounds
   - Check before expensive operations

3. **String Caching**
   - Cache formatted keycode strings
   - Reuse when keycode unchanged
   - Clear cache on update

4. **Layout Caching**
   - Cache terminal layout calculations
   - Invalidate on resize
   - Reuse between frames

### Memory Management

**Allocation Strategies:**

1. **Pre-Allocation**
   - Allocate fixed-size vectors where possible
   - Reuse buffers for rendering
   - Avoid dynamic allocation in hot paths

2. **Copy-on-Write**
   - Share immutable data
   - Clone only when modifying
   - Use Rc/Arc for large shared data

3. **Streaming**
   - Process large files in chunks
   - Don't load entire keyboard database at once
   - Stream build output line-by-line

### File I/O Optimization

**Read Operations:**
- Buffer reads (use BufReader)
- Parse incrementally where possible
- Validate as you parse (early exit on error)

**Write Operations:**
- Buffer writes (use BufWriter)
- Atomic writes (temp file + rename)
- Flush explicitly when needed

### Background Task Management

**Build Process:**
- Spawn thread, don't block main loop
- Use channels for communication
- Limit message frequency (throttle)
- Cancel gracefully if needed

**Keyboard Scanning:**
- Cache results
- Incremental updates
- Debounce rapid scans

### Database Optimization

**Keycode Database:**
- Load once at startup
- Index by category for fast filtering
- Use HashMap for O(1) lookups
- Binary search for sorted data

**Category Lookups:**
- HashMap for O(1) access by ID
- Index by name for fuzzy search
- Cache color values

### Terminal Rendering

**Buffer Management:**
- Single buffer per frame
- Clear before drawing
- Flush atomically
- Minimize terminal writes

**Unicode Handling:**
- Use ASCII fallbacks where possible
- Check terminal capabilities
- Cache grapheme clusters

---

## 17. Testing Strategy

### Unit Testing

**Model Tests:**
- Position equality
- Color parsing
- Coordinate transformations
- Matrix mapping

**Parser Tests:**
- Valid input produces correct output
- Invalid input produces clear errors
- Edge cases (empty tables, malformed markdown)
- Round-trip (parse → write → parse)

**Color Priority Tests:**
- Four-level precedence
- All combinations
- Fallback behavior

### Integration Testing

**File I/O:**
- Save and load layouts
- Template operations
- Config persistence
- Error recovery

**Parser Integration:**
- End-to-end layout parsing
- QMK metadata parsing
- Firmware generation
- Coordinate mapping pipeline

### UI Testing

**Manual Testing Required:**
- Visual appearance
- Keyboard interaction
- Terminal compatibility
- Resize handling

**Automated Snapshot Tests (Future):**
- Capture rendered output
- Compare against baselines
- Detect visual regressions

### Build System Testing

**QMK Integration:**
- Firmware generation correctness
- Compilation success
- Error handling
- Output file validation

**Cross-Platform:**
- Test on Windows, macOS, Linux
- Different terminal emulators
- Various QMK versions

### Test Data

**Sample Layouts:**
- Minimal (basic 36-key)
- Standard (42-key Corne)
- Extended (46-key with EX)
- Edge cases (empty layers, all transparent)

**Invalid Inputs:**
- Malformed markdown
- Invalid keycodes
- Missing layers
- Color syntax errors

---

## 18. Future Extensions

### Potential Enhancements

**Visual Improvements:**
- Mouse support for key selection
- Graphical key representations
- Key legends from labels
- Animated transitions

**Editing Features:**
- Copy/paste keys
- Undo/redo support
- Multi-key selection
- Bulk operations
- Layer duplication
- Key swapping

**Advanced Keycode Support:**
- Tap dance editor
- Combo editor
- Macro editor
- Unicode input
- Key overrides

**Layout Features:**
- Home row mods visualization
- Key frequency heatmap
- Layout analyzer
- Efficiency metrics

**Collaboration:**
- Import from online repos
- Export to QR code
- Share via cloud
- Layout comparison tool

**AI Integration:**
- Layout suggestions
- Ergonomic analysis
- Common pattern detection
- Auto-categorization

**Hardware Integration:**
- Live preview on physical keyboard
- Key test mode
- RGB preview
- Firmware flashing UI

**Advanced Templates:**
- Template variables
- Conditional sections
- Template inheritance
- Auto-generation from typing data

**Internationalization:**
- Multiple language support
- Locale-aware keycodes
- Translated UI

**Plugin System:**
- Custom keycode parsers
- Custom generators
- Custom color schemes
- External tool integration

---

## Conclusion

This architecture guide provides a comprehensive blueprint for building a sophisticated TUI application for mechanical keyboard layout editing. The key takeaways:

1. **Clear Architecture** - Separation of models, views, and controllers
2. **Robust Coordinate System** - Three-way mapping between LED, matrix, and visual
3. **User-Friendly Design** - Intuitive shortcuts, visual feedback, helpful messages
4. **Extensible Structure** - Easy to add features, categories, templates
5. **Performance-Conscious** - Event-driven rendering, lazy evaluation, caching
6. **Cross-Platform** - Terminal independence, platform-aware paths
7. **Well-Tested** - Unit tests, integration tests, manual verification

The system successfully handles the complex requirements of keyboard layout editing while maintaining a responsive, intuitive interface within the constraints of a terminal environment.

**Technologies Demonstrated:**
- Immediate-mode UI with Ratatui
- Event-driven architecture with Crossterm
- Background task management with channels
- File format design (markdown)
- Complex coordinate transformations
- State management patterns
- Build system integration

This architecture can serve as a reference for building other TUI applications with similar requirements: complex data models, multiple coordinate systems, background operations, and rich user interaction.
