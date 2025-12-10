<div align="center">

<pre>
   __                  ____  __  ________
  / /   ____ _____  __/_/ / /  |/  / //_/
 / /   / __ `/_  / / / / / / /|_/ / ,<   
/ /___/ /_/ / / /_/ / / / / /  / / /| |  
/_____/\__,_/ /___/_/ /_/ /_/  /_/_/ |_|  
</pre>

### The Interactive Terminal Workspace for QMK Firmware

[![Build Status](https://img.shields.io/github/actions/workflow/status/user/keyboard-configurator/ci.yml?branch=main&style=flat-square)](https://github.com/user/keyboard-configurator/actions)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE)
[![Rust Version](https://img.shields.io/badge/rust-1.75%2B-orange?style=flat-square)](https://www.rust-lang.org)
[![v0.4.0](https://img.shields.io/badge/version-0.4.0-green?style=flat-square)](https://github.com/user/keyboard-configurator/releases)

[Features](#features) • [Installation](#installation) • [Documentation](docs/README.md) • [Contributing](#contributing)

</div>

---

<!-- 
    HERO SECTION: 
    Replace the image source below with a GIF or SVG of your TUI in action.
    Tools like 'asciinema' or 'terminalizer' are great for this.
    For now, use a static screenshot if you don't have a recording.
-->

![LazyQMK Demo](docs/public/demo_placeholder.png)

**LazyQMK** is a modern TUI (Terminal User Interface) that bridges the gap between the raw power of QMK and the ease of visual configuration. Built in **Rust**, it provides a blazing fast, type-safe environment to design keymaps, manage layers, and compile firmware without ever leaving your terminal.

## Why use this?

We built this for keyboard enthusiasts who love the CLI but hate the friction of editing C files manually. Inspired by tools like `lazygit` and `lazydocker`, we want to make firmware configuration effortless.

| ⚡️ Blazing Fast | 🎨 Visual & Interactive | 🛡️ Type-Safe |
| :--- | :--- | :--- |
| Powered by Rust and QMK, compilation is instant. No more waiting for slow web configurators. | A rich TUI built with `Ratatui`. Visualize your physical layout, drag-and-drop keys (via keyboard), and manage layers intuitively. | Validates your configuration *before* compilation. Catch conflicting keycodes and matrix errors instantly. |

## Features

- **Visual Layout Editor**: See your keyboard geometry as you edit.
- **Layer Management**: Create, reorder, and toggle between unlimited QMK layers.
- **Live Validation**: Real-time error checking prevents invalid firmware states.
- **Direct QMK Integration**: Uses the official `qmk_firmware` submodule for 100% compatibility.
- **Vim-like Navigation**: Navigate the interface with `h`/`j`/`k`/`l` and standard modal shortcuts.

## Installation

### Prerequisites
- **Rust**: `cargo` (v1.75 or later)
- **QMK Dependencies**: `avr-gcc`, `arm-none-eabi-gcc` (standard QMK build tools)

### From Source
```bash
git clone --recursive https://github.com/user/keyboard-configurator.git lazyqmk
cd lazyqmk
cargo install --path .
```

## Quick Start

1. **Launch the TUI**:
   ```bash
   lazyqmk
   ```

2. **Select your Keyboard**: Use the fuzzy finder to search the QMK database (e.g., `dz60`).

3. **Edit Keymap**: 
   - Press `Enter` to modify a key.
   - Use `Tab` to switch between the visual map and the keycode picker.
   - Press `s` to save your configuration.

4. **Compile**: Press `b` to trigger a compilation. The firmware file will be generated in `./output`.

## Documentation

We maintain comprehensive documentation for users and contributors:

- [**Architecture Guide**](docs/ARCHITECTURE.md): Deep dive into the Rust + Ratatui internal structure.
- [**Feature List**](docs/FEATURES.md): A complete breakdown of supported QMK features.
- [**Component Docs**](docs/components/): Detailed usage for specific modules (e.g., Settings Manager).

## Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for details on how to submit pull requests, report issues, or request features.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
