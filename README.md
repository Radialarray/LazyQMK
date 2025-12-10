<div align="center">

<pre>
   __                  ____  __  ________
  / /   ____ _____  __/ __ \/ |/ / //_/
 / /   / __ `/_ / // / / / / /|_/ / ,<   
/ /___/ /_/ / / /_/ /_/ / / /  / / /| |  
/_____/\__,_/ /___/\___/_/_/  /_/_/ |_|  
</pre>

### The Interactive Terminal Workspace for QMK Firmware

[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square)](https://www.rust-lang.org)
[![Latest Release](https://img.shields.io/github/v/release/Radialarray/LazyQMK?style=flat-square)](https://github.com/Radialarray/LazyQMK/releases)

[Features](#-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Documentation](#-documentation)

</div>

---

**LazyQMK – Keyboard Layout Editor** is a modern terminal-based keyboard layout editor for QMK firmware. Built in **Rust** with **Ratatui**, it bridges the gap between the raw power of QMK and the ease of visual configuration. Design keymaps, manage layers, organize with colors and categories, and compile firmware—all without leaving your terminal.

Inspired by tools like `lazygit` and `lazydocker`, LazyQMK makes firmware configuration effortless for keyboard enthusiasts who love the CLI.

## 🎯 Why LazyQMK?

| 🎨 Visual & Interactive | 🛡️ Type-Safe | 📝 Human-Readable |
| :--- | :--- | :--- |
| Rich TUI with visual keyboard layout, intuitive navigation, and live feedback. | Validates keycodes and layout before compilation. Catches errors instantly. | Plain Markdown files with YAML frontmatter. Perfect for version control. |

## ✨ Features

### Core Capabilities
- **Visual Layout Editor** - See your keyboard geometry as you edit with accurate physical positioning
- **Multi-Layer Support** - Create and manage unlimited QMK layers with easy tab-based navigation
- **Smart Color System** - Four-level priority system (key → key category → layer category → layer default)
- **Category Organization** - Group keys by function (navigation, symbols, modifiers, etc.)
- **Searchable Keycode Picker** - Fuzzy search through 600+ QMK keycodes with instant filtering
- **Language-Specific Keycodes** - Support for 10+ keyboard layouts (QWERTY, QWERTZ, AZERTY, Colemak, etc.)

### Firmware Integration
- **Direct QMK Integration** - Uses official QMK firmware with full keyboard database access
- **JSON5 Parser** - Handles complex QMK configs with C++ style comments
- **Background Compilation** - Build firmware without blocking the UI, with live progress updates
- **Smart Config Discovery** - Automatically merges parent `info.json` + variant `keyboard.json` files
- **Universal Keyboard Support** - Works with split keyboards, ortholinear, ergonomic, and standard layouts

### Developer-Friendly
- **Human-Readable Markdown** - Layouts stored as `.md` files with YAML frontmatter
- **Version Control Ready** - Plain text format perfect for git
- **Template System** - Save and reuse common layouts across keyboards
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
1. **QMK Firmware Path** - Path to your local QMK repository
2. **Keyboard Selection** - Choose from QMK's extensive keyboard database
3. **Layout Variant** - Select your physical layout (if multiple options)

Configuration saved to:
- **Linux**: `~/.config/LazyQMK/config.toml`
- **macOS**: `~/Library/Application Support/LazyQMK/config.toml`
- **Windows**: `%APPDATA%\LazyQMK\config.toml`

### Creating a Layout

```bash
# Create new layout file
touch my_layout.md
lazyqmk my_layout.md

# Or load existing layout
lazyqmk path/to/layout.md
```

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

## 🏗️ Architecture

### Tech Stack

- **Rust 1.88.0** - Systems programming language
- **Ratatui 0.29** - Terminal UI framework with immediate mode rendering
- **Crossterm 0.29** - Cross-platform terminal manipulation
- **Serde 1.0** - Serialization/deserialization
- **JSON5 1.3** - QMK config parsing (supports C++ comments)
- **Clap 4.5** - CLI argument parsing

### Design Patterns

- **MVC Architecture** - Clean separation of models, views, and controllers
- **Component Trait Pattern** - All 14 active components use standardized `Component` trait
- **Event-Driven** - Components communicate via events, handlers update state
- **Immediate Mode Rendering** - UI rebuilt every frame from centralized `AppState`

### Project Structure

```
src/
├── app/          # Application entry point and launch logic
├── models/       # Data structures (Layout, Layer, KeyDefinition)
├── parser/       # File parsing (Markdown, QMK info.json, JSON5)
├── tui/          # Terminal UI components
│   ├── handlers/ # Input handlers (actions, categories, layers)
│   └── [components] # UI widgets (keyboard, pickers, editors)
├── keycode_db/   # QMK keycode database (600+ keycodes)
├── firmware/     # Code generation and compilation
├── services/     # Business logic (geometry, layouts)
└── main.rs       # Entry point
```

## 📚 Documentation

### User Guides
- **[Quick Start Guide](QUICKSTART.md)** - Getting started, workflows, shortcuts
- **[Features Overview](docs/FEATURES.md)** - Comprehensive feature documentation

### Technical Documentation
- **[Architecture Guide](docs/ARCHITECTURE.md)** - Deep dive into technical design
- **[Shortcut System](docs/SHORTCUT_SYSTEM_ANALYSIS.md)** - Keyboard shortcut design

### Component Guides
- **[Settings Manager](docs/components/SETTINGS_MANAGER.md)** - Configuration management

### Specifications
- **[Archived Specs](specs/archived/)** - Historical development specifications

## 🤝 Contributing

We welcome contributions! This project is actively maintained and follows best practices:

- **Conventional Commits** - Structured commit messages
- **Comprehensive Testing** - All changes must pass tests
- **Documentation** - User and technical docs kept up-to-date
- **Code Quality** - Clippy lints enforced

See [AGENTS.md](AGENTS.md) for development guidelines and workflow.

## 🗺️ Roadmap

**Completed (v0.7.0)**
- ✅ Visual keyboard editor with accurate geometry
- ✅ Multi-layer management
- ✅ Color and category system
- ✅ Template system
- ✅ Background firmware compilation
- ✅ QMK JSON5 parser with robust config discovery
- ✅ Language-specific keycode support
- ✅ OS theme integration
- ✅ Complete dependency updates (ratatui 0.29, crossterm 0.29, etc.)

**Future Enhancements**
- Combo key configuration UI
- Tap-dance configuration UI
- Macro recording and playback
- Layout analysis and heatmaps
- Multi-keyboard profile management
- ZMK firmware support

## 🎯 Supported Keyboards

LazyQMK works with **any keyboard in the QMK firmware repository**, including:

- **Split Keyboards**: Corne (crkbd), Ferris Sweep, Lily58, Kyria, Sofle, Ergodox
- **Ortholinear**: Planck, Preonic, Let's Split
- **Standard**: DZ60, Tofu60, KBD67, etc.
- **Sizes**: 36-key, 40%, 60%, 65%, 75%, TKL, and full-size

Successfully tested with complex structures like `splitkb/aurora/lily58/rev1`.

## 📈 Project Status

**Current Version**: v0.7.0  
**Status**: Active Development  
**Test Coverage**: 287/287 passing  
**Last Updated**: 2025-12-10

### Recent Updates (v0.7.0)
- 🎉 Major dependency updates (ratatui 0.29, crossterm 0.29, clap 4.5)
- 🔧 Robust QMK keyboard parser with JSON5 support
- 🐛 Fixed 43 deprecation warnings + 11 clippy warnings
- 📦 Migrated from deprecated `serde_yaml` to `serde_yml`
- ✨ Improved config discovery for complex QMK structures

## 📄 License

This project is licensed under the **MIT License** - see [LICENSE](LICENSE) for details.

---

<div align="center">

**Built with ❤️ by keyboard enthusiasts, for keyboard enthusiasts**

[Report Bug](https://github.com/Radialarray/LazyQMK/issues) • [Request Feature](https://github.com/Radialarray/LazyQMK/issues) • [View Releases](https://github.com/Radialarray/LazyQMK/releases)

</div>
