# Lesson 0002: Install and Doctor (Phase 1 + 2)

> Get `lazyqmk` installed and the environment verified. End: `lazyqmk doctor` shows all `✓`.

## Goal

- `lazyqmk` binary on PATH
- Custom QMK fork cloned to `~/qmk_firmware`
- `lazyqmk doctor` returns all green

## Discovery Gate (silent)

```bash
# Is lazyqmk installed?
which lazyqmk && lazyqmk --version || echo "MISSING"

# Is QMK fork present?
ls -d ~/qmk_firmware 2>/dev/null && ls ~/qmk_firmware/keyboards | head -5 || echo "MISSING"

# Current doctor state (if lazyqmk installed)
lazyqmk doctor --json 2>/dev/null | head -50 || echo "lazyqmk not available"
```

## User Questions (if anything missing)

| Missing | Ask |
|---|---|
| `lazyqmk` not installed | "What OS are you on? macOS / Linux / Windows?" |
| `~/qmk_firmware` missing | (no question; install below) |
| QMK CLI missing | (no question; install via pip) |

## Phase 1: Install Steps

### macOS / Linux (Homebrew — recommended)

```bash
brew install Radialarray/lazyqmk/lazyqmk

# Verify
which lazyqmk && lazyqmk --version
```

### Other platforms

Download from <https://github.com/Radialarray/LazyQMK/releases/latest> and place in PATH.

### Install QMK CLI (used by `lazyqmk doctor`)

```bash
python3 -m pip install --user qmk
qmk --version
```

### Clone custom QMK fork (REQUIRED)

**Important:** the official `qmk/qmk_firmware` repo is missing LazyQMK's custom features (ripple overlay, idle effect state machine, PaletteFX integration). Using it produces a firmware that compiles but the advanced features don't work.

```bash
git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git ~/qmk_firmware

# Initialize QMK submodules if needed
cd ~/qmk_firmware
git submodule update --init --recursive
qmk setup  # if QMK CLI is configured
cd -
```

### Install ARM GCC (for RP2040 builds)

macOS: `brew install --cask gcc-arm-embedded`
Linux (Debian): `sudo apt install gcc-arm-none-eabi binutils-arm-none-eabi`
Windows: Use QMK MSYS2

### Install AVR GCC (for AVR builds — older keyboards)

macOS: included with Xcode CLI tools or `brew install avr-gcc`
Linux: `sudo apt install gcc-avr avr-libc`

## Phase 2: Doctor Steps

### Configure lazyqmk with the QMK path

```bash
# Edit config or use CLI
lazyqmk config set --qmk-path ~/qmk_firmware
```

Verify `~/Library/Application Support/LazyQMK/config.toml`:

```toml
[paths]
qmk_firmware = "/Users/<you>/qmk_firmware"

[build]
output_dir = "/Users/<you>/Library/Application Support/LazyQMK/builds"

[ui]
show_help_on_startup = true
theme_mode = "Auto"
keyboard_scale = 1.0
```

### Run doctor

```bash
lazyqmk doctor --json | jq '{status, passed, failed, unknown, dependencies: [.dependencies[] | {name, status, version}]}'
```

Healthy output looks like:

```json
{
  "status": "ready",
  "passed": 4,
  "failed": 0,
  "unknown": 0,
  "dependencies": [
    { "name": "QMK CLI",      "status": "available", "version": "1.1.8" },
    { "name": "ARM GCC",      "status": "available", "version": "8.5.0" },
    { "name": "AVR GCC",      "status": "available", "version": "8.5.0" },
    { "name": "QMK Firmware", "status": "available", "version": "Valid QMK directory at /Users/you/qmk_firmware" }
  ]
}
```

Top-level `status: "ready"` means all dependencies are `available`. There are exactly 4 dependencies checked: QMK CLI, ARM GCC, AVR GCC, QMK Firmware Path.

### If anything shows `missing` or `unknown`

```bash
lazyqmk doctor --verbose
```

Follow suggestions. Common fixes:

| Issue | Fix |
|---|---|
| QMK CLI missing | `python3 -m pip install --user qmk` |
| ARM GCC missing | Install via OS package manager or QMK MSYS (Windows) |
| AVR GCC missing | Same as above |
| QMK Firmware missing | `lazyqmk config set --qmk-path ~/qmk_firmware` |

## Validation

```bash
# Final check: all green
bash scripts/doctor.sh
# Or manually:
lazyqmk doctor --json | jq -r '.status'
# Should return: "ready"
```

## Workspace Update

If a workspace exists for the keyboard the user is setting up, append to NOTES.md:

```markdown
## Environment
- lazyqmk: <version>
- QMK CLI: <version>
- ARM GCC: <version>
- AVR GCC: <version>
- QMK fork: ~/qmk_firmware (commit <hash>)
```

If no workspace exists yet, proceed to **Mission Gate** (see SKILL.md) before doing anything else.

## Next Lesson

`lessons/0003-pick-keyboard.md` (Phase 3) — once doctor is green.