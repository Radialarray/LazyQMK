# Lesson 0009: Build UF2 (Phase 9)

> Compile generated QMK source to firmware binary (`.uf2`, `.hex`, or `.bin`). End: firmware file ready to flash.

## Goal

- `qmk compile` succeeds
- Firmware file (`*.uf2` or whatever `output_format`) located
- File copied to a discoverable location

## Discovery Gate (silent)

```bash
# QMK CLI available?
which qmk && qmk --version

# QMK fork ready? (use the configured path, not the default)
QMK_PATH=$(lazyqmk config show --json | jq -r '.paths.qmk_firmware // empty')
[ -z "$QMK_PATH" ] && QMK_PATH="$HOME/qmk_firmware"
ls "$QMK_PATH/keyboards/<metadata.keyboard>/info.json"

# Generated files exist (rules.mk is OPTIONAL)
ls -la <out_dir>/keymap.c <out_dir>/config.h
[ -f <out_dir>/rules.mk ] && echo "rules.mk: present" || echo "rules.mk: absent (no combos/tap dance)"
[ -f <out_dir>/keymap.json ] && echo "keymap.json: present" || echo "keymap.json: absent (PaletteFX not enabled)"

# Output format from metadata (display only — actual extension depends on bootloader)
jq -r '.metadata.output_format' ~/.../my.json
```

## User Questions

None — compile is automatic (no user input needed unless errors).

## Phase 9: Steps

### Step 1: Verify QMK CLI

```bash
qmk --version
```

If missing: `python3 -m pip install --user qmk`.

### Step 2: Verify QMK fork

```bash
ls ~/qmk_firmware/keyboards/<vendor>/<kb>/info.json
# Should exist
```

If missing, the lazyqmk config path is wrong:

```bash
lazyqmk config set --qmk-path ~/qmk_firmware
```

### Step 3: Run doctor (final check)

```bash
bash scripts/doctor.sh
```

Should be all green. If not, fix before compiling.

### Step 4: Deploy generated keymap into QMK tree

`lazyqmk generate` writes to `--out-dir` only; it does NOT install files into the QMK tree. You must copy them manually before `qmk compile` can use them.

Use the **full** `metadata.keyboard` path (which may include a variant like `keebart/corne_choc_pro/standard`), matching what `FirmwareGenerator::get_keymap_directory()` does internally:

```bash
QMK_PATH=$(lazyqmk config show --json | jq -r '.paths.qmk_firmware // empty')
[ -z "$QMK_PATH" ] && QMK_PATH="$HOME/qmk_firmware"
KEYBOARD=$(jq -r '.metadata.keyboard' ~/.../my.json)
KEYMAP_NAME=$(jq -r '.metadata.keymap_name' ~/.../my.json)

KEYMAP_DIR="$QMK_PATH/keyboards/$KEYBOARD/keymaps/$KEYMAP_NAME"
mkdir -p "$KEYMAP_DIR"

# Wipe stale files first (prevents stale rules.mk / keymap.json after disabling features)
rm -f "$KEYMAP_DIR/rules.mk" "$KEYMAP_DIR/keymap.json"

# Copy generated files into QMK tree
cp <out_dir>/keymap.c "$KEYMAP_DIR/"
cp <out_dir>/config.h "$KEYMAP_DIR/"
[ -f <out_dir>/rules.mk ] && cp <out_dir>/rules.mk "$KEYMAP_DIR/"
[ -f <out_dir>/keymap.json ] && cp <out_dir>/keymap.json "$KEYMAP_DIR/"

ls -la "$KEYMAP_DIR"
```

If you skip this step, `qmk compile` will either build a stale existing keymap or fail with "keymap not found."

### Step 5: Compile (cd to QMK path first)

```bash
KEYBOARD="<vendor>/<keyboard>/<variant>"
# e.g., "keebart/corne_choc_pro/standard"

KEYMAP_NAME=$(jq -r '.metadata.keymap_name' ~/.../my.json)
# e.g., "corne_choc_pro"

# IMPORTANT: cd into the QMK path so qmk compile uses the configured fork,
# not wherever QMK CLI's user.qmk_home points (which may be elsewhere).
QMK_PATH=$(lazyqmk config show --json | jq -r '.paths.qmk_firmware // empty')
[ -z "$QMK_PATH" ] && QMK_PATH="$HOME/qmk_firmware"
cd "$QMK_PATH"

qmk compile -kb "$KEYBOARD" -km "$KEYMAP_NAME"
```

`qmk compile` will:

1. Read keymap files from `~/qmk_firmware/keyboards/<kb>/keymaps/<keymap_name>/`
2. Compile via `make` (Linux/macOS) or QMK MSYS (Windows)
3. Produce a `.uf2`, `.hex`, or `.bin` file

### Step 6: Locate the firmware file

LazyQMK's builder looks for a flat file at `<qmk-path>/.build/<keyboard_with_slashes_as_underscores>_<keymap>.<ext>` (matching `find_firmware_file()` in `src/firmware/builder/build.rs`):

```bash
QMK_PATH=$(lazyqmk config show --json | jq -r '.paths.qmk_firmware // empty')
[ -z "$QMK_PATH" ] && QMK_PATH="$HOME/qmk_firmware"
KEYBOARD=$(jq -r '.metadata.keyboard' ~/.../my.json)
KEYMAP_NAME=$(jq -r '.metadata.keymap_name' ~/.../my.json)

KEYBOARD_SLUG=$(echo "$KEYBOARD" | tr '/' '_')

# Try common extensions in order (uf2, hex, bin) — same priority as find_firmware_file()
FIRMWARE_PATH=""
for ext in uf2 hex bin; do
  CANDIDATE="$QMK_PATH/.build/${KEYBOARD_SLUG}_${KEYMAP_NAME}.${ext}"
  if [ -f "$CANDIDATE" ]; then
    FIRMWARE_PATH="$CANDIDATE"
    break
  fi
done

if [ -z "$FIRMWARE_PATH" ]; then
  echo "FAIL: no firmware found under $QMK_PATH/.build/${KEYBOARD_SLUG}_${KEYMAP_NAME}.{uf2,hex,bin}"
  ls "$QMK_PATH/.build/" 2>/dev/null | head
  exit 1
fi

ls -la "$FIRMWARE_PATH"
echo "Firmware: $FIRMWARE_PATH"
```

Note: `metadata.output_format` is stored for display only; the actual extension depends on the keyboard's QMK bootloader config (RP2040 → `.uf2`, AVR → `.hex`, ARM → `.bin`).

## Validation

```bash
# Firmware file exists and is non-empty
test -s "$FIRMWARE_PATH"

# Firmware file is valid (basic checks)
file "$FIRMWARE_PATH"
# For UF2: "data" or "Microsoft UF2 firmware image"
# For HEX: "ASCII text" with Intel HEX format
# For BIN: "data"

# UF2 magic bytes (if UF2)
xxd "$FIRMWARE_PATH" | head -1
# Should start with: 55 46 32 0a 57 51 5d 1e (UF2 magic)
```

## Firmware Copy to Builds Directory

LazyQMK does NOT automatically copy built firmware to `~/Library/Application Support/LazyQMK/builds/`. Do it manually:

```bash
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
KEYBOARD_SLUG=$(echo "$KEYBOARD" | tr '/' '_')
KEYMAP=$(jq -r '.metadata.keymap_name' ~/.../my.json)

BUILD_DIR=~/Library/Application\ Support/LazyQMK/builds/${KEYBOARD_SLUG}_${KEYMAP}_${TIMESTAMP}
mkdir -p "$BUILD_DIR"

# Copy firmware + all source artifacts
cp "$FIRMWARE_PATH" "$BUILD_DIR/"
cp <out_dir>/keymap.c "$BUILD_DIR/"
cp <out_dir>/config.h "$BUILD_DIR/"
[ -f <out_dir>/rules.mk ] && cp <out_dir>/rules.mk "$BUILD_DIR/"
[ -f <out_dir>/keymap.json ] && cp <out_dir>/keymap.json "$BUILD_DIR/"

ls -la "$BUILD_DIR"
```

## Build Errors and Fixes

Common QMK compile errors:

| Error | Cause | Fix |
|---|---|---|
| `ERROR: keymap ... not found` | Generated keymap.c not in right place | Verify `qmk compile` placed it correctly; check `~/qmk_firmware/keyboards/<kb>/keymaps/<keymap_name>/` |
| `undefined reference to tap_dance_actions` | `TAP_DANCE_ENABLE = yes` missing | Check generated `rules.mk`; verify `tap_dances[]` non-empty in layout |
| `palettefx/reactive: not found` | Official QMK fork | Switch to Radialarray fork |
| `rgb_matrix_indicators_advanced_user: undefined` | Official QMK fork (ripple missing) | Switch to Radialarray fork |
| `LQMK_IDLE_* undefined` | Official QMK fork (idle effect missing) | Switch to Radialarray fork |
| `LAYOUT_* macro not found` | Wrong variant name | Verify `metadata.layout_variant` matches `info.json` `layouts[].name` |

If you see "undefined reference" or "missing function" errors, **the most common cause is using the official QMK fork**. Re-clone from `Radialarray/qmk_firmware`:

```bash
cd ~/qmk_firmware
git remote -v  # should show Radialarray, not qmk
# If wrong:
cd ~/
mv qmk_firmware qmk_firmware.official.bak
git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git
```

After re-cloning:

```bash
lazyqmk config set --qmk-path ~/qmk_firmware
bash scripts/doctor.sh
cd ~/qmk_firmware
qmk compile -kb "$KEYBOARD" -km "$KEYMAP_NAME"
```

## Workspace Update

Add to NOTES.md:

```markdown
## Built firmware
- YYYY-MM-DD HH:MM:SS: Built <kb>_<keymap>_<ts>
  - Output: $BUILD_DIR/<keymap_name>.uf2
  - Size: <bytes>
  - Compile: qmk compile (took <duration>)
```

## Common Pitfalls

- **`qmk compile` builds for host's default ARM target** — RP2040 uses pico SDK; AVR uses avr-gcc. Both should be installed (doctor checks).
- **Compile takes 5–10 minutes** — first build of a keyboard always slow; subsequent builds faster (cached).
- **Submodule missing** — `git submodule update --init --recursive` in `~/qmk_firmware`.
- **`rules.mk` says wrong `MCU`** — verify with `cat ~/qmk_firmware/keyboards/<kb>/rules.mk`.

## Next Lesson

`lessons/0010-flash-and-verify.md` (Phase 10) — flash to keyboard.