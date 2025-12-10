# LazyQMK – Keyboard Layout Editor - Quick Start Guide

A terminal-based keyboard layout editor for QMK firmware.

## Installation

```bash
# Clone the repository
git clone https://github.com/Radialarray/LazyQMK.git
cd lazyqmk

# Initialize QMK submodule
git submodule update --init --recursive qmk_firmware

# Build in release mode
cargo build --release

# Binary will be at target/release/lazyqmk
```

## First Run - Onboarding Wizard

On first run, the onboarding wizard will guide you through configuration:

```bash
./target/release/lazyqmk
```

The wizard will prompt you for:
1. **QMK firmware path** - Path to your QMK firmware directory
2. **Keyboard selection** - Choose from keyboards available in QMK
3. **Layout variant** - Select layout if your keyboard has multiple options

Configuration is saved to:
- Linux: `~/.config/LazyQMK/config.toml`
- macOS: `~/Library/Application Support/LazyQMK/config.toml`
- Windows: `%APPDATA%\LazyQMK\config.toml`

## Basic Usage

### Creating a New Layout

```bash
# Create an empty layout file
touch my_layout.md

# Open in editor
lazyqmk my_layout.md
```

### Loading an Existing Layout

```bash
lazyqmk path/to/existing_layout.md
```

## Core Workflows

### 1. Editing Keys

1. **Navigate** - Use arrow keys or `hjkl` (VIM-style) to move between keys
2. **Edit keycode** - Press `Enter` to open keycode picker
3. **Search** - Type to fuzzy search through 600+ QMK keycodes
4. **Assign** - Press `Enter` to assign selected keycode
5. **Clear key** - Press `x` or `Delete` to clear (sets to KC_TRNS)

### 2. Working with Layers

1. **Switch layers** - Press `Tab` to move to next layer, `Shift+Tab` for previous
2. **Edit keys** - Navigate and assign keycodes as normal
3. **Set layer color** - Press `c` to set the default color for current layer
4. **Layer category** - Press `Shift+L` to assign layer to a category

### 3. Color Organization

The color system has four priority levels (highest to lowest):
1. Individual key color override
2. Key's assigned category color
3. Layer's assigned category color
4. Layer default color

**To set individual key color:**
1. Navigate to key
2. Press `Shift+C`
3. Use arrow keys to adjust RGB sliders
4. Press `Enter` to apply

**To assign key to category:**
1. Navigate to key
2. Press `Shift+K`
3. Select category from list
4. Press `Enter` to assign

### 4. Category Management

Categories help organize keys by function (navigation, symbols, modifiers, etc.):

1. Press `Ctrl+T` to open Category Manager
2. **Create**: Press `n` to create new category
3. **Edit**: Select category and press `e` to edit name/color
4. **Delete**: Select category and press `d` to delete
5. **Navigate**: Use arrow keys to move between categories
6. **Close**: Press `Esc` or `q`

### 5. Firmware Generation

**Generate firmware files:**
1. Press `Ctrl+G` to generate `keymap.c` and `config.h`
2. Files are generated based on your configuration

**Build firmware:**
1. Press `Ctrl+B` to start background build
2. Press `Shift+B` to view build log with live progress
3. Build runs in background - you can continue editing

### 6. Templates

**Save current layout as template:**
1. Press `Shift+T`
2. Enter template name, description, author, and tags
3. Press `Enter` to save

**Load template:**
1. Press `t` to open template browser
2. Navigate with arrow keys
3. Press `Enter` to load selected template
4. Template is applied to current layout

### 7. Saving and Exiting

- **Save** - Press `Ctrl+S` to save layout to Markdown file
- **Quit** - Press `Ctrl+Q` to quit (prompts if unsaved changes)
- **Auto-save prompt** - Application warns before quitting with unsaved changes

## Keyboard Shortcuts

**Note:** Press `?` in the app for the complete, up-to-date shortcut reference with context-sensitive help.

**Essential shortcuts to get started:**
- Arrow keys / `hjkl` - Navigate between keys
- `Enter` - Open keycode picker  
- `Tab` / `Shift+Tab` - Switch layers
- `Ctrl+S` - Save layout
- `Ctrl+B` - Build firmware
- `?` - Help overlay

## File Format

Layouts are stored as human-readable Markdown files with YAML frontmatter:

```markdown
---
name: "My Layout"
keyboard: "crkbd/rev1"
layout_variant: "LAYOUT_split_3x6_3"
author: "Your Name"
version: "1.0"
---

# Layer 0 - Base

| KC_TAB | KC_Q | KC_W | KC_E | KC_R | KC_T |
| KC_LSFT | KC_A | KC_S | KC_D | KC_F | KC_G |
...
```

## Tips

1. **Use categories** - Organize related keys (navigation, numbers, symbols) for visual clarity
2. **Color layers** - Give each layer a distinct default color for quick identification
3. **Templates** - Save common layouts (QWERTY, Colemak) as templates for reuse
4. **Search keycodes** - Use fuzzy search in keycode picker (type "ctrl" to find all Ctrl keys)
5. **Check help** - Press `?` anytime to see all available shortcuts

## Next Steps

- Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for technical architecture
- Read [docs/FEATURES.md](docs/FEATURES.md) for comprehensive feature documentation
- See [specs/archived/](specs/archived/) for historical development specifications
- Check [README.md](README.md) for installation and development info

## Troubleshooting

**Terminal displays incorrectly:**
- Ensure your terminal supports ANSI escape sequences and Unicode
- Try a modern terminal (iTerm2, Alacritty, Windows Terminal)
- Minimum size: 80x24 characters recommended

**Configuration not persisting:**
- Check write permissions in config directory
- Linux: `~/.config/LazyQMK/`
- macOS: `~/Library/Application Support/LazyQMK/`
- Windows: `%APPDATA%\LazyQMK\`

**QMK build fails:**
- Verify QMK firmware path in configuration
- Ensure QMK toolchain is installed
- Check build log with `Shift+B` for specific errors

**Cannot find keyboard:**
- Ensure QMK firmware path points to valid QMK repository
- Verify keyboard exists in QMK: `qmk list-keyboards`
- Try re-running setup wizard (`Ctrl+W`) to reconfigure keyboard selection
