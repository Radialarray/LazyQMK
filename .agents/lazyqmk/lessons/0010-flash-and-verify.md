# Lesson 0010: Flash and Verify (Phase 10)

> Flash firmware to keyboard and verify it works. End: working keyboard with user's keymap.

## Goal

- Firmware successfully flashed to keyboard
- First keystroke verified
- User confirms keyboard works as expected

## Discovery Gate (silent)

```bash
# Firmware file exists (note: name is <keyboard>_<keymap>.<ext>, not just <keymap>)
ls -la $BUILD_DIR/  # list to find actual filename

# Output format (display only — actual extension depends on bootloader)
jq -r '.metadata.output_format' ~/.../my.json

# Bootloader combo configured? (if so, user can use it)
jq '.combo_settings' ~/.../my.json
```

## User Questions

| Ask | Why |
|---|---|
| "Confirm you want to flash <kb> with <keymap>? (irreversible)" | Explicit confirmation |
| "Is the keyboard in bootloader mode?" | Required for flashing |
| "Want to verify specific keys work?" | Functional verification |

**CRITICAL: Never flash without explicit user confirmation.**

## Phase 10: Steps

### Method 1: Bootloader Combo (recommended)

If user configured a bootloader combo (e.g., W+E for 800ms), they can put the keyboard into bootloader mode without touching it.

```bash
# Tell user:
echo "Hold W and E together for 800ms to enter bootloader mode."
echo "The keyboard will appear as a USB drive named RPI-RP2 (or similar)."

# Wait for user to confirm
read -p "Is the keyboard in bootloader mode? (y/n) "
```

### Method 2: Physical Reset Button

Most RP2040 keyboards have a reset button (often accessible through a hole in the case).

```bash
# Tell user:
echo "Press the reset button (usually a small button accessible via a pin hole)."
echo "The keyboard will appear as a USB drive."
```

### Step 2: Copy firmware

Once the keyboard is in bootloader mode, it appears as a USB mass storage device.

#### Method A: Manual copy (UF2 bootloaders only — macOS/Linux/Windows)

```bash
# UF2 bootloaders (RP2040) appear as a USB mass storage device.
# Detect mount point cross-platform:
MOUNT=""
case "$(uname)" in
  Darwin)
    MOUNT=$(ls -d /Volumes/RPI-RP2 2>/dev/null | head -1)
    ;;
  Linux)
    # Common Linux mount locations for RP2040 bootloader
    MOUNT=$(ls -d /media/*/RPI-RP2 /run/media/*/RPI-RP2 2>/dev/null | head -1)
    ;;
  MINGW*|MSYS*|CYGWIN*)
    MOUNT="/d/RPI-RP2"  # Git Bash typical; adjust for your environment
    ;;
esac

if [ -z "$MOUNT" ] || [ ! -d "$MOUNT" ]; then
  echo "FAIL: RPI-RP2 bootloader not found. Is the keyboard in bootloader mode?"
  exit 1
fi

# Only UF2 can be copied via mass-storage copy
FIRMWARE=$(ls $BUILD_DIR/*.uf2 2>/dev/null | head -1)
[ -z "$FIRMWARE" ] && echo "FAIL: no .uf2 file in $BUILD_DIR; HEX/BIN require Method B (qmk flash)"
cp "$FIRMWARE" "$MOUNT/"

# Eject (macOS)
[ "$(uname)" = "Darwin" ] && diskutil eject "$MOUNT"
# Unmount (Linux)
[ "$(uname)" = "Linux" ] && umount "$MOUNT"
```

For `.hex` and `.bin` files (AVR, ARM), use Method B (`qmk flash`) instead — those targets don't expose a USB drive.

#### Method B: Use qmk flash (alternative)

```bash
KEYBOARD=$(jq -r '.metadata.keyboard' ~/.../my.json)
KEYMAP=$(jq -r '.metadata.keymap_name' ~/.../my.json)

qmk flash -kb "$KEYBOARD" -km "$KEYMAP"
```

`qmk flash` is interactive — it will prompt you to put the keyboard in bootloader mode and automatically flash.

### Step 3: Wait for reboot

After flashing, the keyboard will reboot (usually 1-2 seconds).

### Step 4: Functional verification

Ask user to test:

1. **Type a few characters** — does each key produce the expected character?
2. **Test layer access** — hold the LT() key, type — does it produce the upper-layer character?
3. **Test mods** — hold LSFT_T(KC_F), press F — does it type 'F' (capital)? Hold past tapping_term — does it Shift?
4. **Test tap dance** (if configured) — single tap, double tap, hold — each produces correct keycode?
5. **Test combo** (if configured) — hold both keys for `hold_duration_ms`, then release — action fires on release (not while held)
6. **Test RGB** — do layer colors display correctly? Idle effect plays after timeout?
7. **Test bootloader combo** (if configured) — hold both keys for 800ms, then release — keyboard enters bootloader. There is no longer a built-in fallback chord — if the user wants a bootloader shortcut they must configure it as a combo with action `bootloader`.

Report results back. If anything is wrong, debug.

## Common Issues After Flash

### Issue: Keys don't produce expected characters

**Cause**: wrong layer active, or wrong keycode.

**Debug**:

```bash
# Open TUI briefly to inspect (NOT in agent mode — ask user to do this)
lazyqmk ~/.../my.json
```

**Fix**: re-edit the keycode, regenerate, re-flash.

### Issue: Home-row mods trigger on every press

**Cause**: tapping_term too short, or no flow_tap_term/chordal_hold.

**Fix**:

```bash
jq '.tap_hold_settings.tapping_term = 250' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
# Or switch to HomeRowMods preset (if not already)
```

### Issue: Layer access doesn't work

**Cause**: wrong UUID in `LT(@<uuid>, ...)`, or layer doesn't exist.

**Debug**:

```bash
# Show layer UUIDs
jq '.layers | map({name, id})' ~/.../my.json

# Show all LT() references
jq '.layers[].keys[] | select(.keycode | test("^LT\\(")) | .keycode' ~/.../my.json
```

**Fix**: ensure UUIDs match.

### Issue: Tap dance doesn't behave as expected

**Cause**: auto-created placeholder not replaced with real definition.

**Debug**:

```bash
# Show tap dances
lazyqmk tap-dance list --layout ~/.../my.json
```

**Fix**: replace placeholder with real `single_tap` / `double_tap` / `hold`.

### Issue: Combo doesn't fire

**Cause**: keys not on layer 0, or hold_duration too long/short.

**Fix**:

```bash
# Verify keys are on layer 0
jq '.layers[0].keys[] | select(.position.row == 0 and .position.col == 2)' ~/.../my.json

# Adjust duration
jq '.combo_settings.combos[0].hold_duration_ms = 500' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Issue: RGB looks wrong

**Cause**: wrong layer active, color_override too saturated, or `uncolored_key_behavior` too high/low.

**Fix**:

```bash
# Reduce uncolored_key_behavior
jq '.uncolored_key_behavior = 30' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json

# Or disable RGB entirely
jq '.rgb_enabled = false' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Issue: Idle effect never activates

**Cause**: `idle_effect_settings.enabled = false` or `idle_timeout_ms = 0`.

**Debug**:

```bash
jq '.idle_effect_settings' ~/.../my.json
```

**Fix**: enable + set timeout.

### Issue: Idle effect doesn't turn off

**Cause**: `idle_effect_duration_ms = 0` (immediate off — probably not what you want).

**Fix**:

```bash
jq '.idle_effect_settings.idle_effect_duration_ms = 300000' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

## Workspace Update

Add to NOTES.md:

```markdown
## Flash results
- YYYY-MM-DD HH:MM:SS: Flashed to <kb> with <keymap>
  - Method: bootloader combo / qmk flash / manual copy
  - Result: success / partial / failed
  - Verified: <which tests passed>
  - Issues: <any open issues>
```

## End State

You're done when:

1. User confirms keyboard types correctly
2. User confirms all features work
3. MISSION.md "must-have features" all checked
4. NOTES.md has flash results logged
5. Optional: `lazyqmk export` produced a shareable `*.md`

## Optional: Export Layout Documentation

```bash
LAYOUT=~/.../my.json
QMK_PATH=~/qmk_firmware

lazyqmk export \
  --layout "$LAYOUT" \
  --qmk-path "$QMK_PATH" \
  --output ~/.../layout-export-$(date +%Y%m%d).md
```

Produces a human-readable markdown with all layers, colors, tap dances, settings. Shareable on GitHub, in chat, etc.

## Pipeline Complete

The user has gone from blank slate → polished UF2 → working keyboard.

For future tweaks, see:

- `lessons/0001-pipeline-overview.md` (re-enter at any phase)
- `references/0000-cli-cheatsheet.md` (single source of CLI truth)
- Feature references (0001-0007) for deep dives on specific features

The workspace (`~/.../workspaces/<slug>/`) is the persistent context for future sessions.