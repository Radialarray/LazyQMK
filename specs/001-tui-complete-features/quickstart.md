# Quickstart: Keyboard Configurator TUI

**Feature**: Complete TUI Keyboard Layout Editor  
**Branch**: `001-tui-complete-features`  
**Status**: Implementation Plan Complete
**Project**: Keyboard Configurator

## Overview

This guide helps developers get started implementing the TUI keyboard layout editor based on the architecture defined in TUI_ARCHITECTURE_GUIDE.md.

## Prerequisites

- Rust 1.75 or higher
- QMK firmware repository cloned locally
- Terminal with Unicode and true color support (recommended)
- Development tools: cargo, rust-analyzer (optional but recommended)

## Development Setup

### 1. Clone and Setup

```bash
# Repository should already exist
cd keyboard-configurator

# Ensure QMK submodule is initialized
git submodule update --init --recursive vial-qmk-keebart

# Build the project
cargo build
```

### 2. Project Structure Walkthrough

```text
src/
├── main.rs                  # Entry point - CLI arg parsing, main loop initialization
├── config.rs                # TOML config loading/saving (see contracts/config-toml-schema.md)
├── models/                  # Pure data structures (see data-model.md)
│   ├── layout.rs            # Layout, LayoutMetadata
│   ├── layer.rs             # Layer, KeyDefinition, Position
│   ├── category.rs          # Category
│   ├── rgb.rs               # RgbColor with hex conversion
│   ├── keyboard_geometry.rs # KeyboardGeometry, KeyGeometry  
│   └── visual_layout_mapping.rs # VisualLayoutMapping (coordinate transforms)
├── parser/                  # File parsing (independent of UI)
│   ├── layout.rs            # Markdown → Layout (see contracts/layout-markdown-schema.md)
│   ├── table.rs             # Table parsing state machine
│   ├── keyboard_json.rs     # QMK info.json parser (see contracts/qmk-info-json-schema.md)
│   └── template_gen.rs      # Layout → Markdown
├── tui/                     # UI components (stateless widgets)
│   ├── mod.rs               # AppState, main loop, event routing
│   ├── keyboard.rs          # Keyboard widget (renders keys with colors)
│   ├── keycode_picker.rs    # Searchable keycode picker
│   ├── color_picker.rs      # RGB color picker with sliders
│   └── [other dialogs]      # Category manager, template browser, help, etc.
├── keycode_db/              # Embedded QMK keycode database
│   └── keycodes.json        # 600+ keycodes with categories
└── firmware/                # Firmware generation and building
    ├── generator.rs         # Generate keymap.c and vial.json
    └── builder.rs           # Background build with channels
```

### 3. Core Concepts

#### Three-Coordinate System

The application manages three coordinate spaces (see research.md "Three-Coordinate Mapping"):

1. **Visual** - User's view in Markdown tables (row, col)
2. **Matrix** - Electrical wiring (row, col)
3. **LED** - Sequential LED index

**Key Insight**: `VisualLayoutMapping` handles all bidirectional conversions between these spaces.

#### Centralized State Management

All application state lives in `AppState` (see research.md "Centralized State Management"):

- Single source of truth
- UI components read immutably
- Event handlers update explicitly
- No hidden state in widgets

#### Immediate-Mode Rendering

Every frame rebuilds UI from state (see research.md "Immediate-Mode Rendering"):

```text
Event → Update AppState → Render from State → Display
```

## Implementation Order

### Phase 0: Foundation (MVP Core)

**Goal**: Basic layout loading and display

1. **Models** (data-model.md):
   - Implement `Position`, `RgbColor`, `KeyDefinition`, `Layer`, `Layout`
   - Add serde derives for serialization
   - Write unit tests for color priority resolution

2. **Parser** (contracts/layout-markdown-schema.md):
   - Implement Markdown table parser (table.rs)
   - Implement YAML frontmatter parsing (layout.rs)
   - Handle color/category syntax extraction
   - Write parser tests with sample Markdown files

3. **Basic TUI** (research.md "UI Component Patterns"):
   - Setup Ratatui terminal initialization
   - Implement keyboard widget (keyboard.rs)
   - Render keys with positions and labels
   - No interaction yet (static display)

**Checkpoint**: Can load a Markdown file and display keyboard layout in terminal

### Phase 1: User Interaction (MVP Navigation)

**Goal**: Navigate and edit keys

4. **Event Handling** (research.md "Event-Driven Rendering"):
   - Implement main event loop with 100ms poll
   - Handle arrow keys for navigation
   - Highlight selected key
   - Update AppState on key press

5. **Keycode Picker** (tui/keycode_picker.rs):
   - Load keycode database from keycodes.json
   - Implement fuzzy search
   - Category filtering
   - Update key on selection

**Checkpoint**: Can navigate keyboard and assign keycodes

### Phase 2: Color System (MVP Visual Organization)

**Goal**: Four-level color priority

6. **Color Picker** (tui/color_picker.rs):
   - RGB sliders with keyboard navigation
   - Hex input parsing
   - Live preview
   - Apply to keys/layers

7. **Category System** (models/category.rs + tui/category_manager.rs):
   - Category CRUD operations
   - Category picker dialog
   - Color resolution algorithm (priority levels)

**Checkpoint**: Can organize keys with colors and categories

### Phase 3: Persistence (MVP Save/Load)

**Goal**: Human-readable file format

8. **Save Implementation** (parser/template_gen.rs):
   - Generate Markdown from Layout
   - Serialize color/category syntax
   - Atomic file writes (temp + rename)
   - Dirty flag management

9. **Configuration** (config.rs + contracts/config-toml-schema.md):
   - TOML parsing with serde
   - Platform-specific config directories
   - Validation of paths

**Checkpoint**: Can save/load layouts as Markdown files

### Phase 4: QMK Integration (MVP Firmware)

**Goal**: Generate and build firmware

10. **Geometry Loading** (parser/keyboard_json.rs + contracts/qmk-info-json-schema.md):
    - Parse QMK info.json
    - Build KeyboardGeometry
    - Build VisualLayoutMapping with three-coordinate system
    - Write coordinate transformation tests

11. **Firmware Generation** (firmware/generator.rs):
    - Generate keymap.c with PROGMEM arrays
    - Generate vial.json
    - Validate keycodes and matrix coverage
    - Pre-generation validation

12. **Background Building** (firmware/builder.rs + research.md "Background Thread"):
    - Spawn build thread
    - Message channel for progress updates
    - Stream build log
    - Success/error handling

**Checkpoint**: Can generate and compile QMK firmware

### Phase 5: Enhanced UX (P2 Features)

**Goal**: Improve discoverability and workflow

13. **Help System** (tui/help_overlay.rs):
    - Comprehensive shortcut documentation
    - Scrollable overlay
    - Context-sensitive status messages

14. **Template System** (tui/template_browser.rs):
    - Save layouts as templates
    - Browse with metadata
    - Search by tags
    - Load into session

15. **Configuration Dialogs** (tui/*.rs):
    - Onboarding wizard for first-run
    - Keyboard picker with QMK scanning
    - Layout picker from info.json
    - Path/format configuration

**Checkpoint**: Full-featured editor with templates and help

## Development Workflow

### Running the Application

```bash
# Development mode (with debug output)
cargo run -- path/to/layout.md

# Release mode (optimized)
cargo run --release -- path/to/layout.md

# Run the configurator
keyboard-configurator path/to/layout.md

# Run tests
cargo test

# Run specific test module
cargo test parser::tests

# Check without building
cargo check
```

### Testing Strategy

1. **Unit Tests** (tests/unit/):
   - Parser round-trip (Markdown → Layout → Markdown)
   - Coordinate transformations (visual ↔ matrix ↔ LED)
   - Color priority resolution
   - Keycode validation

2. **Integration Tests** (tests/integration/):
   - Full save/load cycle
   - Firmware generation pipeline
   - Configuration persistence

3. **Contract Tests** (tests/contract/):
   - Parse real QMK info.json files from submodule
   - Validate against actual keyboard definitions

### Debugging Tips

**Terminal Issues**:
```bash
# Check terminal capabilities
echo $TERM

# Force true color
export COLORTERM=truecolor

# Reset terminal if corrupted
reset
```

**Render Issues**:
- Add debug logging in keyboard.rs render method
- Use cargo's RUST_LOG for detailed output:
  ```bash
  RUST_LOG=debug cargo run
  ```

**Coordinate Mapping Issues**:
- Write tests that print visual → matrix → LED transformations
- Verify against QMK info.json physical positions
- Check split keyboard column reversal logic

## Key Files to Start With

### 1. main.rs

```rust
use clap::Parser;

#[derive(Parser)]
struct Cli {
    /// Path to layout markdown file
    #[arg(required = true)]
    layout_path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load config
    let config = config::load_or_default()?;
    
    // Parse layout
    let layout = parser::load_layout(&cli.layout_path)?;
    
    // Load keyboard geometry
    let geometry = parser::load_keyboard_geometry(&config)?;
    let mapping = models::VisualLayoutMapping::build(&geometry);
    
    // Initialize TUI
    let mut terminal = tui::setup_terminal()?;
    let mut app_state = AppState::new(layout, geometry, mapping, config);
    
    // Main loop
    loop {
        terminal.draw(|f| tui::render(f, &app_state))?;
        
        if let Some(event) = tui::poll_event()? {
            if tui::handle_event(&mut app_state, event)? {
                break; // User quit
            }
        }
    }
    
    tui::restore_terminal(terminal)?;
    Ok(())
}
```

### 2. models/layout.rs

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Layout {
    pub metadata: LayoutMetadata,
    pub layers: Vec<Layer>,
    pub categories: Vec<Category>,
}

impl Layout {
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
    
    pub fn resolve_key_color(&self, layer_idx: usize, key: &KeyDefinition) -> RgbColor {
        // Four-level priority (see data-model.md)
        if let Some(color) = key.color_override {
            return color; // 1. Individual override (highest)
        }
        if let Some(cat_id) = &key.category_id {
            if let Some(cat) = self.categories.iter().find(|c| &c.id == cat_id) {
                return cat.color; // 2. Key category
            }
        }
        let layer = &self.layers[layer_idx];
        if let Some(cat_id) = &layer.category_id {
            if let Some(cat) = self.categories.iter().find(|c| &c.id == cat_id) {
                return cat.color; // 3. Layer category
            }
        }
        layer.default_color // 4. Layer default (fallback)
    }
}
```

### 3. tui/mod.rs (AppState)

```rust
pub struct AppState {
    // Core data
    pub layout: Layout,
    pub source_path: Option<PathBuf>,
    pub dirty: bool,
    
    // UI state
    pub current_layer: usize,
    pub selected_position: Position,
    pub active_popup: Option<PopupType>,
    
    // System resources
    pub keycode_db: KeycodeDatabase,
    pub geometry: KeyboardGeometry,
    pub mapping: VisualLayoutMapping,
    pub config: Config,
    
    // Component states
    pub keycode_picker_state: KeycodePickerState,
    pub color_picker_state: ColorPickerState,
    // ... other component states
}

pub fn handle_event(state: &mut AppState, event: Event) -> Result<bool> {
    match event {
        Event::Key(key_event) => {
            if let Some(popup) = &state.active_popup {
                handle_popup_input(state, popup, key_event)
            } else {
                handle_main_input(state, key_event)
            }
        }
        Event::Resize(_, _) => {
            // Terminal resized, recalculate layout
            Ok(false)
        }
    }
}
```

## Common Patterns

### Pattern 1: Popup Management

```rust
// Open popup
state.active_popup = Some(PopupType::KeycodePicker);
state.keycode_picker_state = KeycodePickerState::default();

// Close popup and apply
let selected_keycode = state.keycode_picker_state.selected_keycode();
state.layout.layers[state.current_layer]
    .get_key_mut(state.selected_position)
    .keycode = selected_keycode;
state.active_popup = None;
state.dirty = true;
```

### Pattern 2: Coordinate Transformation

```rust
// Visual → Matrix (for saving)
let matrix_pos = state.mapping.visual_to_matrix_pos(visual_row, visual_col)?;

// Matrix → LED (for firmware generation)
let led_idx = state.mapping.matrix_to_led.get(&matrix_pos)?;

// LED → Visual (for rendering)
let matrix_pos = state.mapping.led_to_matrix[led_idx];
let visual_pos = state.mapping.matrix_to_visual[&matrix_pos];
```

### Pattern 3: Atomic Save

```rust
fn save_layout(layout: &Layout, path: &Path) -> Result<()> {
    let markdown = template_gen::generate_markdown(layout)?;
    
    let temp_path = path.with_extension("md.tmp");
    std::fs::write(&temp_path, markdown)?;
    std::fs::rename(temp_path, path)?;
    
    Ok(())
}
```

## Reference Documentation

- **Architecture**: `TUI_ARCHITECTURE_GUIDE.md` - Comprehensive technical architecture
- **Features**: `QUICKSTART.md` (root) - Feature list and user guide
- **Data Model**: `specs/001-tui-complete-features/data-model.md` - Entity relationships
- **Research**: `specs/001-tui-complete-features/research.md` - Pattern decisions
- **Contracts**: `specs/001-tui-complete-features/contracts/` - File format schemas
- **Spec**: `specs/001-tui-complete-features/spec.md` - User stories and requirements

## Next Steps

1. **Start Implementation**: Begin with Phase 0 (Foundation)
2. **Write Tests First**: For each module, write tests before implementation
3. **Incremental Progress**: Complete one phase before moving to next
4. **Constitution Compliance**: Verify adherence to principles at each checkpoint

## Getting Help

- QMK Documentation: https://docs.qmk.fm/
- Ratatui Documentation: https://ratatui.rs/
- Crossterm Documentation: https://docs.rs/crossterm/
- Constitution: `.specify/memory/constitution.md` - Project principles
