# LazyQMK Quick Start Guide

## Installation

> [!IMPORTANT]
> **Custom QMK Fork Required**: LazyQMK requires a custom QMK firmware fork for full functionality. Using the official QMK firmware will result in limited features.

### Step 1: Install LazyQMK Binary

Choose your platform:

**macOS/Linux (Homebrew - Recommended):**
```bash
brew install Radialarray/lazyqmk/lazyqmk
```

**Linux, macOS, or Windows (Manual):**
Download from the [releases page](https://github.com/Radialarray/LazyQMK/releases/latest) for your platform.

### Step 2: Clone Custom QMK Firmware Fork

```bash
git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git ~/qmk_firmware
```

### Step 3: Verify Your Setup

After installation, verify that all required tools are present:

```bash
lazyqmk doctor
```

This command checks:
- ✓ QMK CLI installation
- ✓ Required build toolchains (ARM GCC, avr-gcc, etc.)
- ✓ QMK firmware directory
- ✓ Git configuration

**Successful output looks like:**
```
✓ QMK CLI                  v1.1.5
✓ ARM GCC                  arm-none-eabi-gcc (11.3.1)
✓ AVR GCC                  avr-gcc (5.4.0)
✓ QMK Firmware Path        /Users/user/qmk_firmware
✓ Git Configuration        Configured
```

**With issues:**
```
✗ QMK CLI                  Not installed
? ARM GCC                  Not found in PATH
✓ AVR GCC                  avr-gcc (5.4.0)
```

To see detailed diagnostics:
```bash
lazyqmk doctor --verbose
```

**If issues are found:**
1. Install missing tools
2. Ensure PATH environment is configured correctly
3. Run `lazyqmk doctor` again to verify

### Step 4: Launch LazyQMK

```bash
lazyqmk
```

The onboarding wizard will ask for:
- **QMK Firmware Path**: `/Users/user/qmk_firmware` (or your custom fork path)
- **Keyboard**: Your keyboard name (e.g., `crkbd/rev1` for Corne)
- **Layout Variant**: Your physical layout (e.g., `LAYOUT_split_3x6_3`)

Done! You're ready to start editing your keyboard layout.

---

## Basic Workflow

Once configured, your typical workflow looks like this:

1. **Navigate** - Use arrow keys (`↑↓←→`) or VIM-style (`hjkl`)
2. **Edit Key** - Press `Enter` to open the searchable keycode picker
3. **Search Keycode** - Type to fuzzy search (e.g., "ctrl" finds all Ctrl-related keys)
4. **Assign Keycode** - Press `Enter` to apply
5. **Switch Layers** - Use `Tab` / `Shift+Tab` between layers
6. **Manage Layers** - Press `Shift+L` for layer manager
7. **Save Layout** - Press `Ctrl+S` to save
8. **Build Firmware** - Press `Ctrl+B` to compile

### Your First Edit

1. Launch LazyQMK: `lazyqmk`
2. Navigate to any key using arrow keys
3. Press `Enter` to open the keycode picker
4. Type "esc" and press `Enter` to assign `KC_ESC`
5. Press `Ctrl+S` to save
6. Press `Ctrl+B` to build firmware

---

## Keyboard Shortcuts

**Essential:**
- `↑↓←→` or `hjkl` - Navigate keyboard
- `Enter` - Open keycode picker
- `Tab` / `Shift+Tab` - Switch layers
- `Ctrl+S` - Save layout
- `Ctrl+B` - Build firmware
- `?` - Show help overlay

See `?` in the app for the complete shortcut reference.

---

## Web Interface

Want a modern browser-based editor? LazyQMK also includes a web interface:

```bash
lazyqmk web
```

Then open http://localhost:3001 in your browser.

---

## Troubleshooting

### Setup Issues

If LazyQMK won't start or the wizard loops:
```bash
# Check QMK setup
lazyqmk doctor --verbose

# Verify QMK firmware path exists
ls ~/qmk_firmware
```

### Build Errors

If firmware compilation fails:
```bash
# Diagnose environment
lazyqmk doctor --verbose

# Install missing QMK CLI
python3 -m pip install --user qmk
```

### Need Help?

1. Run `lazyqmk doctor` to diagnose setup issues
2. Check existing [GitHub Issues](https://github.com/Radialarray/LazyQMK/issues)
3. Open a new issue with your `lazyqmk doctor` output

---

## Next Steps

- **Create Custom Layouts** - Edit directly in LazyQMK or edit `.md` files manually
- **Organize with Colors** - Assign colors to keys, categories, and layers
- **Version Control** - Store layouts in git for easy sharing
- **Advanced Features** - Explore tap dance, mod-tap, and other QMK features

For complete documentation, see [README.md](README.md).
