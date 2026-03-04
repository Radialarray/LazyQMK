# LazyQMK — Agent Interaction Guide

> **Purpose**: This guide is written for AI agents helping users set up or enhance keyboard layouts in LazyQMK.  
> It defines an interactive, conversational flow with a clear division of labour between the agent and the user.

---

## Agent Behaviour Principles

**Act first, ask only when necessary.**

The agent should do as much as possible autonomously — running commands, reading files, inspecting state, inferring context — and only pause to ask the user for things that genuinely require human input or decision.

### When to act autonomously (no asking needed)
- Checking if a tool is installed (`lazyqmk doctor`, `which qmk`, etc.)
- Reading existing layout files to understand current state
- Looking up keyboard/variant names from `info.json`
- Detecting the OS, existing config files, installed paths
- Verifying the environment is healthy before proceeding

### When to ask the user
- **Hardware facts the agent cannot know**: Which keyboard do you have? Does it have RGB LEDs?
- **User preferences and intent**: Which keys should be combos? What color do you want? How many layers?
- **Confirmation before irreversible actions**: About to flash firmware. Ready?
- **Ambiguous situations**: Multiple keyboards found — which one?

### Workflow pattern
```
1. Run checks / read files silently
2. Report what was found (briefly)
3. Ask only for what is missing or requires a human decision
4. Act on the answer
```

---

## Quick Routing

Before starting Phase 1, check silently:

```bash
# Is lazyqmk installed?
which lazyqmk

# Does a config file already exist?
ls ~/Library/Application\ Support/LazyQMK/config.toml   # macOS
ls ~/.config/LazyQMK/config.toml                         # Linux
ls $env:APPDATA\LazyQMK\config.toml                      # Windows

# Are there existing layout files?
find ~ -name "*.md" -path "*/LazyQMK/*" 2>/dev/null | head -10
```

**Route based on findings:**
- No `lazyqmk` installed → start at **Phase 1**
- Installed, no config → start at **Phase 2** (first-run wizard)
- Installed, config exists, user wants to enhance → start at **Phase 8**, then return to relevant feature phases
- Installed, config exists, user wants a new layout → start at **Phase 2**, step 2.1

---

## Phase 1 — Installation

> **When to run**: Only if `lazyqmk` is not found in PATH.

### Step 1.1 — Install LazyQMK

Detect the OS and install:

**macOS / Linux (Homebrew — recommended):**
```bash
brew install Radialarray/lazyqmk/lazyqmk
```

**Other platforms:**  
Download the appropriate binary from https://github.com/Radialarray/LazyQMK/releases/latest and place it in PATH.

### Step 1.2 — Clone the custom QMK firmware fork

> **Important**: LazyQMK requires its own QMK fork. The official QMK repo is missing the custom features (ripple overlay, idle effect, etc.) and will produce incomplete firmware.

Check if a QMK directory already exists before cloning:

```bash
ls ~/qmk_firmware 2>/dev/null && echo "exists" || echo "missing"
```

If missing, clone:
```bash
git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git ~/qmk_firmware
```

---

## Phase 2 — Environment Verification

> **Run this silently every session** before doing anything else. Report results to the user only if something needs fixing.

```bash
lazyqmk doctor
```

**Healthy output** (all `✓`):
```
✓ QMK CLI           v1.1.5
✓ ARM GCC           arm-none-eabi-gcc (11.3.1)
✓ AVR GCC           avr-gcc (5.4.0)
✓ QMK Firmware Path /Users/user/qmk_firmware
✓ Git Configuration Configured
```

**If any item shows `✗` or `?`:**
```bash
lazyqmk doctor --verbose
```
Read the output and fix the reported issues before continuing. Common fixes:
- QMK CLI missing: `python3 -m pip install --user qmk`
- ARM GCC missing: install via your OS package manager or QMK MSYS (Windows)
- QMK Firmware Path wrong: update path in `~/.config/LazyQMK/config.toml`

Only proceed to the next phase once `lazyqmk doctor` is fully green.

---

## Phase 3 — First Launch & Onboarding Wizard

> **When to run**: First time the user launches LazyQMK (no config file exists yet).

### Step 3.1 — Identify the keyboard

Before launching, look up available keyboards from the QMK fork to avoid guessing:

```bash
ls ~/qmk_firmware/keyboards/ | head -30
```

Then check the specific keyboard's layout variants:
```bash
cat ~/qmk_firmware/keyboards/<keyboard>/info.json | python3 -m json.tool | grep -A5 '"layouts"'
```

**Ask the user** (this cannot be inferred):
```
Ask: "Which keyboard are you setting up?
      I found these in your QMK firmware folder: [list from above]"
```

Once they name the keyboard, read its `info.json` silently to extract available layout variants and present them — the user should not need to look this up themselves.

### Step 3.2 — Launch and complete the wizard

```bash
lazyqmk
```

The **Onboarding Wizard** asks for:

| Prompt | Source |
|--------|--------|
| QMK Firmware Path | Agent already knows from Phase 1 — provide it |
| Keyboard | From user's answer in Step 3.1 |
| Layout Variant | Agent read from `info.json` — present options, let user choose |

**Alternatively — load an existing layout:**  
If the user already has a `.md` layout file, the welcome screen offers **"Load Existing Layout"** — select it from the detected list.

---

## Phase 4 — Basic Layout Editing

### Navigation reference

Share this table with the user when they start editing:

| Action | Keyboard shortcut |
|--------|-------------------|
| Move cursor | Arrow keys `↑↓←→` or VIM-style `h j k l` |
| Open keycode picker | `Enter` |
| Search for a keycode | Type to fuzzy-search (e.g. `ctrl`, `esc`, `layer`) |
| Assign keycode | `Enter` in picker |
| Clear a key → KC_TRNS | `x` or `Delete` |
| Switch layer | `Tab` / `Shift+Tab` |
| Open layer manager | `Shift+L` |
| Save layout | `Ctrl+S` |
| Build firmware | `Ctrl+B` |
| Full shortcut list | `?` |

### Layer setup

**Ask the user** (their intent cannot be inferred):
```
Ask: "How many layers do you need, and what is each layer for?
      (e.g. layer 0 = base QWERTY, layer 1 = symbols, layer 2 = navigation)"
```

Common pattern:
- Layer 0: Base / QWERTY / Home row
- Layer 1: Symbols / Numbers
- Layer 2: Navigation / Media
- Layer 3+: App-specific / gaming

Use `Shift+L` to rename layers for clarity.

### Color organization (optional)

LazyQMK uses a **four-level color priority system** (highest wins):

| Priority | Level | TUI shortcut |
|----------|-------|--------------|
| 1 (highest) | Individual key override | Select key → `c` |
| 2 | Key category color | Select key → `Ctrl+K` |
| 3 | Layer category color | `Ctrl+L` |
| 4 (lowest) | Layer default color | `Shift+C` |

Open Category Manager with `Shift+K` to create, rename, and assign colors to categories.

---

## Phase 5 — Settings Manager Overview

Most advanced features live in the **Settings Manager** (`Shift+S`).

Groups:
- **Firmware [Global]** — QMK path, keyboard, layout variant
- **RGB Lighting [Layout]** — RGB effects, speed, overlay ripple
- **Idle Effect [Layout]** — Screensaver/idle animation
- **Combos [Layout]** — Two-key hold combos
- **Tap Dance [Layout]** — Multi-tap key actions (also `Shift+D`)

Navigate: `↑↓` or `j/k` · Select: `Enter` · Cancel: `Esc`

**Web UI alternative**: Run `lazyqmk web`, then open `http://localhost:3001`. All settings are available via tabs in the layout editor.

---

## Phase 6 — RGB Enhancements

> **Prerequisite check** (run silently): Inspect the keyboard's `info.json` for `"rgb_matrix"` or `"underglow"` to detect whether RGB LEDs are present.

```bash
cat ~/qmk_firmware/keyboards/<keyboard>/info.json | python3 -m json.tool | grep -i rgb
```

**If no RGB is found in `info.json`**: Inform the user and skip this phase.  
**If RGB is found**: Confirm with the user before configuring:
```
Inform: "Your keyboard supports RGB Matrix. Would you like to configure RGB effects?"
```

> **Runtime requirement**: `RGB_MATRIX_ENABLE = yes` must be in the keyboard's `rules.mk`. LazyQMK handles this when generating firmware, but the custom QMK fork must be used.

---

### Step 6.1 — RGB Matrix Default Speed

Controls the animation speed for all RGB effects.

**Range**: 0 (slowest) → 127 (default) → 255 (fastest)

**Ask the user:**
```
Ask: "What animation speed would you like for RGB effects?
      0 = very slow, 127 = default, 255 = very fast."
```

If they want the default, skip this step — the setting is only written when changed.

#### TUI
1. `Shift+S` → Settings Manager
2. Navigate to **"RGB Lighting [Layout]"**
3. Select **"RGB Matrix Speed"** → type value or `↑↓` → `Enter`

#### Web UI
Layout editor → RGB settings panel → **RGB Matrix Speed** field

**Generated `config.h`** (only emitted when ≠ 127):
```c
#define RGB_MATRIX_DEFAULT_SPD 200
```

**Written to `.md` file** (only when ≠ 127):
```markdown
**RGB Matrix Speed**: 200
```

---

### Step 6.2 — RGB Overlay Ripple Effect

An expanding ring of light radiates from each keypress, layered on top of base layer colors.

**Ask the user:**
```
Ask: "Would you like to enable the ripple effect — an expanding ring of light on each keypress?"
```

If yes, collect preferences for the settings below. Use the defaults as suggestions — the user only needs to specify what they want to change.

#### Enable

**TUI**: `Shift+S` → **"RGB Lighting [Layout]"** → **"Overlay Ripple Enabled"** → `Enter`  
**Web UI**: Layout editor → 🌊 **Overlay Ripple** tab → toggle **"Enable Overlay Ripple"**

#### Ripple settings — ask only for values the user wants to change from defaults

| Setting | Default | Range | Description |
|---------|---------|-------|-------------|
| Max Concurrent Ripples | 4 | 1–8 | How many ripples can run at once |
| Ripple Duration | 500ms | any ms | How long each ripple lasts |
| Ripple Speed | 128 | 0–255 | How fast the ring expands |
| Band Width | 3 | LED units | How wide the ring is |
| Amplitude | 50% | 0–100% | Brightness boost over base color |
| Color Mode | Fixed Color | see below | How ripple color is chosen |
| Fixed Color | `#00FFFF` (cyan) | hex | Color when mode is Fixed Color |
| Trigger on Press | On | On/Off | Fire on keydown |
| Trigger on Release | Off | On/Off | Fire on keyup |
| Ignore Transparent Keys | On | On/Off | Skip KC_TRNS keys |
| Ignore Modifier Keys | Off | On/Off | Skip Shift/Ctrl/Alt/GUI |
| Ignore Layer Switch Keys | Off | On/Off | Skip MO/LT/TG keys |

#### Color Mode options

| Mode | Description | Extra setting needed |
|------|-------------|----------------------|
| **Fixed Color** | All ripples use one fixed color | **Fixed Color** (hex, default `#00FFFF`) |
| **Key Color** | Ripple matches the key's base layer color | — |
| **Hue Shift** | Hue-shifted from the key's base color | **Hue Shift** degree (-180° to 180°) |

> **Known limitation**: Hue Shift mode currently falls back to Key Color behavior.

#### Generated output

**`config.h`** (values shown are defaults):
```c
#define LQMK_RIPPLE_OVERLAY_ENABLED
#define LQMK_RIPPLE_MAX_RIPPLES 4
#define LQMK_RIPPLE_DURATION_MS 500
#define LQMK_RIPPLE_SPEED 128
#define LQMK_RIPPLE_BAND_WIDTH 3
#define LQMK_RIPPLE_AMPLITUDE_PCT 50
#define LQMK_RIPPLE_TRIGGER_ON_PRESS 1
#define LQMK_RIPPLE_TRIGGER_ON_RELEASE 0
```

**`keymap.c`**: A `ripple_t` array is declared. `rgb_matrix_indicators_advanced_user()` is hooked to apply ripple colors each frame, calculating distance from the ripple origin and fading over time.

**`.md` file** (only non-default values written; example shows Fixed Color mode):
```markdown
**Ripple Overlay**: On
**Ripple Color Mode**: Fixed Color
**Ripple Fixed Color**: #FF6600
**Max Ripples**: 6
**Ripple Duration**: 750ms
**Ripple Speed**: 200
```

> **Color mode values**: `Fixed Color`, `Key Color`, `Hue Shift`.  
> `Ripple Fixed Color` is only relevant when mode is `Fixed Color`.  
> `Ripple Hue Shift` is only relevant when mode is `Hue Shift`.

---

## Phase 7 — Two-Key Hold Combos

A combo fires a special action when two physical keys on **layer 0** are held simultaneously for a set duration.

**Fixed action slots:**
- **Combo 1** → Disable RGB Effects (revert to base layer colors)
- **Combo 2** → Disable RGB Lighting entirely
- **Combo 3** → Enter Bootloader (for flashing)

> **Constraints**: Layer 0 only · Keys must differ · Max 3 combos · Hold duration: 50–2000ms · If no combos are defined, no combo code is generated even if combos are enabled.

**Ask the user:**
```
Ask: "Would you like to set up two-key hold combos?
      For example: hold two keys together for 500ms to enter bootloader mode."
```

If yes, ask which combos they want and which keys to use for each:
```
Ask: "Which combo actions do you want?
        1 — Disable RGB Effects
        2 — Disable RGB Lighting
        3 — Bootloader (for flashing)
      For each one, which two keys should trigger it?"
```

**Agent tip**: Before asking for key positions, look up the keyboard's matrix layout from `info.json` so you can describe keys by their visual labels (e.g. "the Q key" or "the spacebar") rather than raw coordinates.

---

### Step 7.1 — Enable combos

**TUI**: `Shift+S` → **"Combos [Layout]"** → **"Combos Enabled"** → `Enter`  
**Web UI**: Layout editor → 🔗 **Combos tab** → toggle **"Enable Combos"**

---

### Step 7.2 — Configure each combo

#### TUI — key position picking

For each combo:
1. Select **"Combo N Key 1 ([Action])"** → `Enter`  
   → Key-selection mode activates. Navigate with arrow keys, press `Enter` to confirm.
2. Select **"Combo N Key 2 ([Action])"** → same flow
3. Select **"Combo N Hold Duration"** → type ms value, or `↑↓` adjusts by 10ms → `Enter`

TUI setting names:
- `Combo 1 Key 1 (Disable Effects)` / `Combo 1 Key 2 (Disable Effects)` / `Combo 1 Hold Duration`
- `Combo 2 Key 1 (Disable Lighting)` / `Combo 2 Key 2 (Disable Lighting)` / `Combo 2 Hold Duration`
- `Combo 3 Key 1 (Bootloader)` / `Combo 3 Key 2 (Bootloader)` / `Combo 3 Hold Duration`

#### Web UI

1. Click **"Add Combo"** (max 3)
2. Fill in per combo row:
   - **Key 1** `{row, col}` and **Key 2** `{row, col}` — matrix coordinates
   - **Action**: `DisableEffects` / `DisableLighting` / `Bootloader`
   - **Hold Duration** (ms)
3. Delete unwanted combos with the per-row delete button

> **Finding matrix coordinates**: Read `keyboards/<keyboard>/info.json` — each key entry has a `"matrix": [row, col]` field. Cross-reference with the key label to identify the right position.

---

### Step 7.3 — Generated output

**`config.h`**:
```c
#define COMBO_ENABLE
#define COMBO_COUNT 2
```

**`rules.mk`** (also generated by LazyQMK when combos are enabled):
```makefile
COMBO_ENABLE = yes
```

**`keymap.c`** (example with 2 combos):
```c
#ifdef COMBO_ENABLE

enum combo_events { COMBO_0, COMBO_1 };

const uint16_t PROGMEM combo_0_keys[] = {KC_A, KC_B, COMBO_END};
const uint16_t PROGMEM combo_1_keys[] = {KC_C, KC_D, COMBO_END};

combo_t key_combos[] = {
    [COMBO_0] = COMBO_ACTION(combo_0_keys),
    [COMBO_1] = COMBO_ACTION(combo_1_keys),
};

void process_combo_event(uint16_t combo_index, bool pressed) {
    if (get_highest_layer(layer_state) != 0) return;  // base layer only
    if (pressed) {
        combo_state[combo_index].timer = timer_read();
        combo_state[combo_index].active = true;
    } else if (combo_state[combo_index].active) {
        uint16_t elapsed = timer_elapsed(combo_state[combo_index].timer);
        switch (combo_index) {
            case COMBO_0:
                if (elapsed >= 500) {
                    // Reverts to TUI layer colors, or solid color if no custom colors:
                    rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);
                }
                break;
            case COMBO_1:
                if (elapsed >= 800) { rgb_matrix_disable_noeeprom(); }
                break;
        }
        combo_state[combo_index].active = false;
    }
}

#endif
```

**`.md` file**:
```markdown
**Combos**: On
**Combo 1**: (0,2)+(0,3) → Disable Effects [500ms]
**Combo 3**: (1,0)+(1,11) → Bootloader [1000ms]
```

---

## Phase 8 — Build & Flash

### Step 8.1 — Save

Press `Ctrl+S` in the TUI. The title bar asterisk (`*`) disappears when the file is saved.

### Step 8.2 — Build

Press `Ctrl+B`. The build runs in the background.

- `Shift+B` opens the build log viewer (scrollable)
- `Ctrl+C` inside log viewer copies the log to clipboard
- Build errors include doctor suggestions — run `lazyqmk doctor --verbose` if needed

### Step 8.3 — Flash

**Ask the user** (requires physical action):
```
Inform: "Build complete. To flash:
         1. Put your keyboard into bootloader mode
            (use Bootloader combo if configured, or press the physical RESET button)
         2. Then I'll run the flash command."
```

Once they confirm the keyboard is in bootloader mode:
```bash
qmk flash
```

---

## Phase 9 — Enhance an Existing Layout

> Start here if the user already has a layout. Run these steps silently, then report findings.

### Step 9.1 — Discover existing layouts

```bash
# Check LazyQMK config for known layout paths
cat ~/Library/Application\ Support/LazyQMK/config.toml   # macOS
cat ~/.config/LazyQMK/config.toml                         # Linux

# Find all layout files
find ~ -name "*.md" -path "*LazyQMK*" 2>/dev/null
```

### Step 9.2 — Read the current layout state

Open the `.md` file and read:
1. The YAML frontmatter (keyboard, variant, metadata)
2. The `## Settings` section (which features are already enabled)
3. The layer tables (current key assignments)

Report a brief summary to the user:
```
Inform: "Found your layout: [name]
         Keyboard: [keyboard/variant]
         Layers: [count and names]
         Current features enabled: [list any active settings]
         What would you like to change or add?"
```

### Step 9.3 — Apply enhancements

Based on what the user wants:
- Add/modify combos → Phase 7
- Enable/configure ripple → Phase 6.2
- Tune RGB speed → Phase 6.1
- Add/edit layers → Phase 4
- Build and flash → Phase 8

---

## Quick Reference Card

| Goal | TUI | Web UI |
|------|-----|--------|
| Settings Manager | `Shift+S` | All settings in editor sidebar |
| Combos | `Shift+S` → Combos group | 🔗 Combos tab |
| Ripple effect | `Shift+S` → RGB Lighting group | 🌊 Overlay Ripple tab |
| RGB speed | `Shift+S` → RGB Lighting group | RGB settings panel |
| Tap dance | `Shift+D` | Tap Dance tab |
| Keycode picker | `Enter` on a key | Click a key |
| Category manager | `Shift+K` | Category panel |
| Individual key color | `c` | Click key → color picker |
| Assign category to key | `Ctrl+K` | Click key → category |
| Assign category to layer | `Ctrl+L` | Layer settings |
| Layer default color | `Shift+C` | Layer settings |
| Export layout doc | `Ctrl+E` | Export button |
| Full shortcut list | `?` | — |

---

## Troubleshooting

### Combos not firing
- Must be on **layer 0** when holding the keys
- Increase hold duration if not triggering; decrease if triggering accidentally
- Verify `COMBO_ENABLE = yes` is in `rules.mk` and `#define COMBO_ENABLE` in `config.h`
- At least one combo must be fully configured — enabling combos with no definitions emits no code

### Ripple effect not visible
- `RGB_MATRIX_ENABLE = yes` must be in `rules.mk`
- Check **Trigger on Press** is enabled
- Try increasing **Amplitude** — the default 50% boost may not be visible on all keyboards
- Verify the custom QMK fork is used, not the official QMK repo

### RGB speed change has no effect
- Default value (127) is intentionally not written to `config.h` — this is correct behaviour
- Only non-default values emit `#define RGB_MATRIX_DEFAULT_SPD`

### Build fails
```bash
lazyqmk doctor --verbose
```
Follow the suggestions printed alongside each error. Common causes: missing ARM GCC, wrong QMK path, old QMK fork.

---

## File Format Reference

Layouts are stored as `.md` files. Settings live in a `## Settings` section below the layer tables.

### Key syntax

| Format | Meaning |
|--------|---------|
| `KC_A` | Plain keycode |
| `KC_A{#FF0000}` | With color override |
| `KC_A@navigation` | With category |
| `KC_A{#FF0000}@navigation` | With color and category |

### Full settings example

```markdown
## Settings

**RGB Matrix Speed**: 180
**Ripple Overlay**: On
**Ripple Color Mode**: Fixed Color
**Ripple Fixed Color**: #00FFFF
**Max Ripples**: 4
**Ripple Duration**: 500ms
**Ripple Speed**: 200
**Ripple Band Width**: 3
**Ripple Amplitude**: 60%
**Combos**: On
**Combo 1**: (0,2)+(0,3) → Disable Effects [500ms]
**Combo 3**: (1,0)+(1,11) → Bootloader [1000ms]
```

> Only non-default values are written. Absent settings use built-in defaults.
