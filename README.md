# Keyboard Configurator

A terminal-based keyboard layout editor for mechanical keyboards with standard QMK firmware support. Edit keyboard layouts visually in your terminal, manage multiple layers, organize keys with colors and categories, and generate QMK firmware - all with human-readable Markdown files.

## Features

- **Visual Editing**: Navigate and edit your keyboard layout using arrow keys or VIM-style navigation (hjkl)
- **Multi-Layer Support**: Edit multiple keyboard layers with easy layer switching (Tab/Shift+Tab)
- **Color Organization**: Four-level color priority system (individual key > key category > layer category > layer default)
- **Category System**: Group keys by logical function (navigation, symbols, modifiers, etc.)
- **Human-Readable Format**: Layouts stored as Markdown files with YAML frontmatter for version control
- **QMK Integration**: Parse QMK keyboard definitions and generate firmware code
- **Background Compilation**: Build QMK firmware in the background with live progress updates
- **Template System**: Save and reuse layouts across keyboards
- **Searchable Keycode Picker**: Fuzzy search through 600+ QMK keycodes with category filtering

## Status

🚧 **In Development** - Phases 1-12 Complete, Phase 13 (Polish) In Progress

- ✅ Core editing features complete
- ✅ Color and category management complete
- ✅ Firmware generation and building complete  
- ✅ Template system complete
- ✅ Help system and configuration dialogs complete
- 🔄 Final polish and testing in progress

Test Results: 109/110 tests passing (99% pass rate)

## Requirements

- **Rust**: Version 1.75 or higher
- **QMK Firmware**: Local clone of QMK firmware repository
- **Terminal**: ANSI escape sequence and Unicode support (iTerm2, Alacritty, Windows Terminal, GNOME Terminal, etc.)
- **Minimum Terminal Size**: 80x24 characters recommended

## Installation

### Pre-built Binary (macOS)

Download the latest release from GitHub:

```bash
# Download the binary
curl -LO https://github.com/Radialarray/Keyboard-Configurator/releases/latest/download/keyboard-configurator

# Make it executable
chmod +x keyboard-configurator

# Move to your PATH (optional)
mv keyboard-configurator /usr/local/bin/
```

Or visit the [Releases page](https://github.com/Radialarray/Keyboard-Configurator/releases) to download manually.

### From Source

```bash
# Clone the repository
git clone https://github.com/Radialarray/Keyboard-Configurator.git
cd keyboard-configurator

# Initialize QMK submodule
git submodule update --init --recursive qmk_firmware

# Build in release mode
cargo build --release

# Binary will be at target/release/keyboard-configurator
```

### Quick Build

```bash
cargo build --release
```

## Quick Start

### First Run

On first run, the onboarding wizard will guide you through configuration:

```bash
./target/release/keyboard-configurator
```

The wizard will prompt you for:
1. QMK firmware path (path to your QMK firmware directory)
2. Keyboard selection (from available keyboards in QMK)
3. Layout variant (if your keyboard has multiple layouts)

Configuration is saved to `~/.config/layout_tools/config.toml` (Unix) or `%APPDATA%\layout_tools\config.toml` (Windows).

### Loading an Existing Layout

```bash
keyboard-configurator path/to/layout.md
```

### Creating a New Layout

```bash
# Create an empty layout file first, then edit it
touch my_layout.md
keyboard-configurator my_layout.md
```

## Usage

### Navigation

- **Arrow Keys** or **hjkl** (VIM-style): Move cursor between keys
- **Tab**: Switch to next layer
- **Shift+Tab**: Switch to previous layer
- **?**: Open help overlay with all shortcuts

### Editing Keys

- **Enter**: Open keycode picker for selected key
- **x** or **Delete**: Clear key (set to KC_TRNS)
- **c**: Set individual key color
- **Shift+K**: Assign key to category

### Layer Operations

- **Shift+C**: Set layer default color
- **Shift+L**: Assign layer to category

### File Operations

- **Ctrl+S**: Save layout
- **Ctrl+Q**: Quit (prompts if unsaved changes)

### Firmware

- **Ctrl+G**: Generate firmware files (keymap.c and config.h)
- **Ctrl+B**: Build firmware in background
- **Ctrl+L**: View build log

### Configuration

- **Ctrl+P**: Change QMK firmware path
- **Ctrl+K**: Select different keyboard
- **Ctrl+Y**: Switch layout variant
- **Ctrl+T**: Open category manager
- **Shift+E**: Edit layout metadata

### Templates

- **t**: Browse and load templates
- **Shift+T**: Save current layout as template

## Project Structure

```
src/
├── models/        # Data structures (Layout, Layer, KeyDefinition)
├── parser/        # File parsing (Markdown, QMK info.json)
├── tui/           # Terminal UI components
├── keycode_db/    # QMK keycode database
├── firmware/      # Firmware generation and building
└── main.rs        # Entry point
```

## Documentation

- [Architecture Guide](TUI_ARCHITECTURE_GUIDE.md) - Technical architecture and design patterns
- [Quickstart](QUICKSTART.md) - Getting started guide
- [Implementation Plan](specs/001-tui-complete-features/plan.md) - Feature implementation details
- [Task Breakdown](specs/001-tui-complete-features/tasks.md) - Development task list

## License

MIT
