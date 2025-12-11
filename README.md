
# LazyQMK
### Terminal Keyboard Layout Editor for QMK Firmware

**LazyQMK** is a terminal-based keyboard layout editor for QMK firmware. Design keymaps, manage layers, organize with colors and categories, and compile firmware—all without leaving your terminal.

---

![LazyQMK Screen](docs/LazyQMK.png)

## 💡 Motivation

I created LazyQMK because I wanted to edit my keyboard firmware for my **Keebart Corne Choc Pro** directly without diving into code every time I needed to tweak a keymap. At the same time, I wanted to support complex coloring of layers and individual keys for better visual organization.

This led me to add custom code to my QMK fork and implement visual layer-aware coloring in a terminal UI editor. Why a TUI? Because I love having small, focused utilities in the terminal—like `lazygit` and `neovim`. LazyQMK follows that philosophy: stay in the terminal, work efficiently, and keep it simple.

> [!IMPORTANT]
> **Project Status**: This is an experimental project testing how far AI-guided coding can go, so expect some rough edges! It's been mostly tested on my Corne Choc Pro, and I can't guarantee it'll work smoothly with other keyboards. The codebase may be unstable or break with other keyboards.
That said, if you're interested in helping make this more robust, broaden hardware support, or refine functionality, contributions and support from the community are highly appreciated. PRs and feedback are very welcome!. 


## ✨ Features

### Core Capabilities
- **Visual Layout Editor** - See your keyboard geometry as you edit with accurate physical positioning
- **Multi-Layer Support** - Create and manage unlimited QMK layers with easy tab-based navigation
- **Smart Color System** - Four-level priority system (key → key category → layer category → layer default)
- **Category Organization** - Group keys by function (navigation, symbols, modifiers, etc.)
- **Searchable Keycode Picker** - Fuzzy search through 600+ QMK keycodes with instant filtering
- **Language-Specific Keycodes** - Support for german keycodes

### Firmware Integration
- **Direct QMK Integration** - Uses custom QMK firmware fork with full keyboard database access (I added support for lighting in the custom firmware fork you can find here: [Custom QMK Firmware Fork](https://github.com/Radialarray/qmk_firmware)). It could potentially work with normal QMK firmware, but LEDs are not supported.

### Developer-Friendly
- **Human-Readable Markdown** - Layouts stored as `.md` files with YAML frontmatter
- **Version Control Ready** - Plain text format perfect for git or a dotfile manager like [chezmoi](https://github.com/twpayne/chezmoi)
- **Template System** - Save and share common layouts across keyboards
- **OS Theme Integration** - Automatic dark/light mode detection from system settings

## 📦 Installation

### Prerequisites
- **Rust** 1.75+ with `cargo`
- **QMK Firmware** - Local clone of the QMK repository
- **Terminal** - Modern terminal with Unicode support (iTerm2, Alacritty, Windows Terminal, etc.)

### From Latest Release

```bash
# Install via cargo
cargo install --git https://github.com/Radialarray/LazyQMK.git --tag v0.7.0

# Or download pre-built binary from releases page
# https://github.com/Radialarray/LazyQMK/releases
```

### From Source

```bash
# Clone with QMK submodule
git clone --recursive https://github.com/Radialarray/LazyQMK.git
cd LazyQMK

# Build in release mode
cargo build --release

# Binary at: target/release/lazyqmk
```

## 🚀 Quick Start

### First Run - Onboarding Wizard

On first launch, the onboarding wizard guides you through setup:

```bash
lazyqmk
```

You'll configure:
1. **QMK Firmware Path** - Path to your local QMK repository (needed to compile the created firmware)
2. **Keyboard Selection** - Choose from QMK's extensive keyboard database
3. **Layout Variant** - Select your physical layout (if multiple options)

Configuration saved to:
- **Linux**: `~/.config/LazyQMK/config.toml`
- **macOS**: `~/Library/Application Support/LazyQMK/config.toml`
- **Windows**: `%APPDATA%\LazyQMK\config.toml`

### Basic Workflow

1. **Navigate** - Arrow keys or `hjkl` (VIM-style)
2. **Edit Key** - Press `Enter` to open keycode picker
3. **Search Keycode** - Type to fuzzy search (e.g., "ctrl" finds all Ctrl keys)
4. **Assign** - Press `Enter` to apply keycode
5. **Switch Layers** - `Tab` / `Shift+Tab`
6. **Save** - `Ctrl+S`
7. **Build Firmware** - `Ctrl+B` (background compilation with live progress)

## ⌨️ Keyboard Shortcuts

**Note:** Press `?` in the app for the complete, up-to-date shortcut reference.

### Essential Shortcuts
- `↑↓←→` or `hjkl` - Navigate keyboard
- `Enter` - Open keycode picker
- `Tab` / `Shift+Tab` - Switch between layers
- `Ctrl+S` - Save layout
- `Ctrl+Q` - Quit application
- `Ctrl+B` - Build firmware
- `Ctrl+G` - Generate firmware files
- `?` - Show help overlay

## 📋 File Format

Layouts are stored as **human-readable Markdown** with YAML frontmatter:

```markdown
---
name: "My Corne Layout"
keyboard: "crkbd/rev1"
layout_variant: "LAYOUT_split_3x6_3"
author: "Your Name"
version: "1.0"
created: "2025-01-15T10:30:00Z"
modified: "2025-01-15T14:20:00Z"
tags: ["colemak", "programming"]
---

## Layer 0: Base

**Color**: #282828 | **Category**: base

| KC_TAB | KC_Q | KC_W | KC_F | KC_P | KC_B |
| KC_LCTL | KC_A{#FF5555} | KC_R | KC_S | KC_T | KC_G |
| KC_LSFT | KC_Z | KC_X | KC_C | KC_D | KC_V |
| KC_ESC | MO(1) | KC_SPC |

## Layer 1: Navigation

**Color**: #FF5555 | **Category**: navigation

| KC_TRNS | KC_HOME | KC_UP | KC_END | KC_PGUP | KC_TRNS |
| KC_TRNS | KC_LEFT@navigation | KC_DOWN | KC_RGHT | KC_PGDN | KC_TRNS |
...
```

**Syntax:**
- Plain keycode: `KC_A`
- With color override: `KC_A{#FF0000}`
- With category: `KC_A@navigation`
- Combined: `KC_A{#FF0000}@navigation`

## 🎨 Color Organization

**Four-Level Priority System** (highest to lowest):

1. **Individual Key Color** (symbol: `i`) - Per-key color overrides
2. **Key Category Color** (symbol: `k`) - Color from key's assigned category
3. **Layer Category Color** (symbol: `L`) - Color from layer's assigned category
4. **Layer Default Color** (symbol: `d`) - Fallback color for layer

Each key displays its color source indicator in the top-right corner.



## 📄 License

This project is licensed under the **MIT License** - see [LICENSE](LICENSE) for details.
