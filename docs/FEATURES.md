# LazyQMK - Features Documentation

> **Last Updated:** 2025-12-12

A comprehensive terminal-based keyboard layout editor for mechanical keyboards with QMK firmware support.

---

## Status

**Development Phase**: Active Development

---

## Core Features

### Visual Editing

**Interactive Keyboard Visualization**
- Real-time keyboard rendering with accurate physical key positioning
- Visual cursor navigation using arrow keys or VIM-style navigation (hjkl)
- Color-coded keys based on function/category
- Yellow highlight for selected key
- Split keyboard support (Corne, Ergodox, Ferris Sweep, etc.)
- Multiple layout sizes (36/40/42/46 keys and more)

**Key Assignment**
- Searchable keycode picker with fuzzy matching
- 600+ QMK keycodes organized by category
- Real-time keycode validation against QMK database
- Quick clear function (x or Delete → KC_TRNS)
- Category-based organization (Basic, Navigation, Symbols, Function, Media, Modifiers)

**Multi-Layer Support**
- Edit multiple keyboard layers (QMK supports up to 32)
- Tab-based layer navigation (Tab/Shift+Tab)
- Layer naming for organization
- Visual layer tabs showing all layers
- Dirty flag tracking (asterisk in title when unsaved)

### Color Organization

**Four-Level Color Priority System**
1. **Individual key color override** (highest) - Symbol: 'i'
2. **Key category color** - Symbol: 'k'  
3. **Layer category color** - Symbol: 'L'
4. **Layer default color** (lowest) - Symbol: 'd'

Each key displays a color source indicator in its top-right corner.

**RGB Color Picker**
- Three independent RGB channel sliders (0-255 each)
- Hex code display and input (#RRGGBB)
- Live color preview swatch
- Fine adjustment: Arrow keys (±1), Shift+Arrow keys (±10)
- Direct hex input support

**Category System**
- User-defined categories for grouping keys by function
- Full CRUD operations via Category Manager (Ctrl+T)
- Per-category color assignment
- Assign categories to individual keys (Shift+K) or entire layers (Shift+L)
- Common presets: navigation, symbols, numbers, function, media, modifiers

### File Format & Persistence

**Human-Readable Markdown**
- Layouts stored as `.md` files with YAML frontmatter
- Version control friendly (plain text, diffable)
- Supports 12-column (standard) and 14-column (with EX keys) layouts
- Comments supported for documentation

**Metadata (YAML Frontmatter)**
- Name, description, author
- Creation and modification timestamps
- Tags for searchability
- Template flag
- Schema version
- Layout variant (e.g., `LAYOUT_split_3x6_3_ex2`)

**Key Syntax in Markdown Tables**
- Plain: `KC_A`
- With color: `KC_A{#FF0000}`
- With category: `KC_A@navigation`
- Combined: `KC_A{#FF0000}@navigation`

**File Operations**
- Auto-save on major operations
- Dirty flag tracking (asterisk in title when unsaved)
- Save warnings on quit (double Ctrl+Q required if unsaved)
- Atomic writes (temp file + rename) for safety

### Template System

**Template Management**
- Save current layout as reusable template (Shift+T)
- Template browser with metadata preview (t key)
- Stored in `~/.config/LazyQMK/templates/` (Linux), `~/Library/Application Support/LazyQMK/templates/` (macOS), or `%APPDATA%\LazyQMK\templates\` (Windows)
- Searchable by name, description, or tags

**Template Loading**
- Templates copied to current file (originals preserved)
- Full metadata preserved
- Compatibility tracking for different keyboard layouts

### Firmware Integration

**QMK Integration**
- Parse QMK keyboard definitions from `info.json`
- Support multiple layout variants per keyboard
- Automatic geometry loading based on QMK metadata
- Matrix mapping (electrical wiring)
- LED index mapping (for RGB lighting)
- Support for split and non-split keyboards

**Code Generation**
- Generate `keymap.c` from layout
- Generate `config.h` with settings
- Layer-aware RGB matrix configuration

**Background Compilation**
- Non-blocking firmware builds (Ctrl+B)
- Live progress updates during compilation
- Build log viewer with scrolling (Shift+B)
- Copy build log to clipboard (Ctrl+C in log view)
- Multiple output formats: UF2 (RP2040), HEX (AVR), BIN (ARM)

**Idle Effect Screensaver**
- Configurable RGB screensaver that activates after keyboard inactivity
- Three-state system: Normal → Idle Effect Animation → LEDs Off
- Customizable idle timeout (default: 1 minute)
- Customizable effect duration (default: 5 minutes before LEDs turn off)
- 9 selectable RGB effects: Breathing (default), Rainbow Beacon, Rainbow Pinwheels, Solid Color, Alphas Mods, Gradient Up/Down, Gradient Left/Right, Band, Band Pinwheel
- Per-layout settings stored in markdown files
- Automatically restores previous RGB mode on keypress
- Conflicts with RGB_MATRIX_TIMEOUT (suppressed when idle effect enabled)
- Configurable via Settings Manager (Shift+S)

**Tap Dance**
- Configure keys with different actions based on tap count and hold
- Two-way tap dance: single tap → keycode, double tap → keycode
- Three-way tap dance: single tap → keycode, double tap → keycode, hold → keycode
- Managed via Tap Dance Editor (Shift+D)
- Create new tap dance actions with step-by-step wizard
- Select from existing tap dances and apply to keys
- Delete unused tap dance definitions
- Actions stored in layout frontmatter (YAML) for version control
- Generates QMK `tap_dance_actions` array automatically in keymap.c
- Supports both `ACTION_TAP_DANCE_DOUBLE` (2-way) and `ACTION_TAP_DANCE_FN_ADVANCED` (3-way)
- Validation warnings for orphaned tap dances (defined but unused)
- Limitations: Uses QMK built-in patterns only (no custom C callbacks)

### Configuration & Setup

**First-Run Onboarding Wizard**
- Step-by-step initial setup
- QMK firmware path configuration with validation
- Keyboard detection from QMK repository
- Layout variant selection

**Configuration Storage**
- TOML format:
  - Linux: `~/.config/LazyQMK/config.toml`
  - macOS: `~/Library/Application Support/LazyQMK/config.toml`
  - Windows: `%APPDATA%\LazyQMK\config.toml`
- Persistent across sessions
- Settings are managed through the Settings Manager (Shift+S) and Setup Wizard (Ctrl+W). See in-app help (?) for all configuration shortcuts.

### User Interface

**OS-Integrated Theming**
- Automatic dark/light mode detection using OS settings
- Dynamic theme switching (responds to OS changes without restart)
- Consistent colors across dark and light modes
- Theme detection using `dark-light` crate v2.0

**Terminal Compatibility**
- Cross-platform: macOS, Linux, Windows
- Supported terminals: iTerm2, Terminal.app, Alacritty, Windows Terminal, GNOME Terminal, etc.
- ANSI escape sequences for colors
- Unicode box-drawing characters
- Minimum recommended size: 80x24 characters
- Responsive layout scaling

**Help System**
- Comprehensive help overlay (? key)
- Scrollable documentation
- Organized by category (Navigation, Editing, File Operations, Firmware, Configuration)
- Context-sensitive status bar

**Status Bar**
- Current mode indicator (Normal, Editing, Building)
- Selected key position (Row, Col)
- Debug info (matrix position, LED index - if enabled)
- Help reminder

---

## Advanced Features

### Coordinate System Architecture

**Three Coordinate Systems**
1. **Matrix Coordinates** - Electrical wiring (row/column for firmware)
2. **LED Index** - Sequential RGB LED order
3. **Visual Position** - How keys appear in UI and markdown

**VisualLayoutMapping**
- Bidirectional transformations between all three systems
- Methods: `led_to_matrix_pos()`, `matrix_to_visual_pos()`, `visual_to_matrix_pos()`, etc.
- Special handling for split keyboards (left/right halves)
- Handles reversed columns and EX keys

### Validation & Error Handling

- Keycode validation against QMK database
- Layout validation before firmware generation
- Matrix coverage checking
- Descriptive error messages with line numbers (for file parsing)
- Recovery suggestions for common errors

### Performance

**Rendering**
- Immediate mode rendering (entire UI rebuilt each frame)
- Event-driven updates (only render on events/state changes)
- Target: 60fps (16ms/frame), typical: 100ms poll timeout
- Lazy evaluation (only compute visible elements)
- Optimized with clipping and early returns

**Background Threading**
- Firmware compilation in background thread
- Message passing (mpsc channels) for progress updates
- No shared mutable state (thread-safe)

---

## Architecture

### Tech Stack

**Core**
- Rust 1.75+ (using Rust 1.88.0)
- Ratatui 0.29 - TUI framework with immediate mode rendering
- Crossterm 0.29 - Cross-platform terminal manipulation

**Data Handling**
- Serde 1.0 - Serialization framework
- serde_json 1.0 - QMK metadata parsing
- serde_yml 0.0.12 - Layout frontmatter parsing
- toml 0.9 - Configuration files

**Utilities**
- anyhow 1.0 - Error handling
- dirs 6.0 - Cross-platform paths
- arboard 3.6 - Clipboard integration
- chrono 0.4 - Timestamps
- dark-light 2.0 - OS theme detection

### Design Patterns

**Model-View-Controller (MVC)**
- Models: Data structures (Layout, Layer, KeyDefinition, etc.)
- Views: UI components (widgets)
- Controller: Event loop processing input and updating state

**Component Trait Pattern (COMPLETE)**
- 14/14 active components migrated to `Component` or `ContextualComponent` traits
- Standardized interface: `handle_input()`, `render()`
- Event-driven communication between components
- Clean separation: components handle UI, handlers handle business logic

**AppState Integration (COMPLETE)**
- All components integrated via `ActiveComponent` enum
- Handler refactoring complete with new `action_handlers/` structure
- State consolidation achieved (~50% reduction in duplicate fields)
- Event-driven architecture fully implemented

### Data Models

**Layout Hierarchy**
```
Layout
├── metadata: LayoutMetadata (name, description, author, tags, timestamps, layout_variant)
├── categories: Vec<Category> (id, name, color)
└── layers: Vec<Layer>
    ├── number, name, default_color, category_id
    └── keys: Vec<KeyDefinition>
        ├── position, keycode, label
        ├── color_override, category_id
        └── combo_participant
```

**Geometry Models**
- KeyGeometry: matrix position, LED index, physical x/y, dimensions, rotation
- KeyboardGeometry: keyboard name, matrix dimensions, key geometries
- MatrixMapping: bidirectional HashMap for (row, col) ↔ LED index
- VisualLayoutMapping: all coordinate transformations

---

## Keyboard Shortcuts

**Note:** Press `?` in the app for the complete, up-to-date shortcut reference.

Essential shortcuts include navigation (arrow keys/hjkl), keycode picker (Enter), layer switching (Tab), saving (Ctrl+S), and firmware building (Ctrl+B/G). The in-app help provides context-sensitive guidance for all features.

---

## Use Cases

**Keyboard Enthusiasts**
- Design custom layouts
- Experiment with layer configurations
- Visualize and iterate on designs
- Share layouts (markdown format)

**Programmers**
- Optimize for programming languages
- Create symbol layers
- Customize function keys
- Version control layouts

**Gamers**
- Gaming-specific layouts
- Macro layers
- WASD optimization
- Quick-switch profiles

**Productivity Users**
- Workflow-specific layers
- Application-specific configurations
- Color-coded visual memory
- Template common layouts

---

## Supported Keyboards

- Any keyboard in QMK firmware repository
- Tested with: Corne (crkbd), Ferris Sweep, split keyboards
- Support for split and non-split layouts
- Support for 36, 40, 42, 46 key layouts (and more)

---

## Project Structure

```
src/
├── app/           # Application entry point and launch logic
├── models/        # Data structures (Layout, Layer, KeyDefinition, etc.)
├── parser/        # File parsing (Markdown, QMK info.json)
├── tui/           # Terminal UI components
│   ├── handlers/  # Input handlers (actions, category, layer, popups, etc.)
│   ├── [components] # UI components (keyboard, color_picker, etc.)
├── keycode_db/    # QMK keycode database (JSON categories)
├── firmware/      # Firmware generation and building
├── services/      # Business logic (geometry, layouts)
└── main.rs        # Entry point
```

---

## Recent Major Milestones

### TUI Architecture Refactor (COMPLETE)
- **Component trait pattern** implemented across 14 active components
- **~50% AppState reduction** by eliminating duplicate state fields
- **Event-driven architecture** with clear separation of concerns
- **Performance maintained** at 60fps UI target
- **Handler refactoring** with new `action_handlers/` directory structure
- **ActiveComponent enum** providing type-safe component management

---

## Future Extensions

*Potential areas for future development (not yet implemented):*
- Combo key configuration UI
- Tap-dance configuration UI
- Macro recording and playback
- Layout analysis and heatmaps
- Multi-keyboard profile management
- Cloud sync for layouts/templates
- ZMK firmware support
