# LazyQMK — Agent Interaction Guide

> **Purpose**: This guide is written for AI agents helping users set up or enhance keyboard layouts in LazyQMK.  
> It provides an interactive, conversational flow — ask the user questions, then act on their answers.  
> Cover every step from first-run setup to advanced features like combos and RGB effects.

> **Quick routing**: If the user already has an existing layout to enhance, jump directly to **Phase 8** first, then return to the relevant feature phases.

---

## How to Use This Guide

Work through the guide **top to bottom** with the user in an interactive chat.  
At each decision point, **ask first — then proceed**. Never skip ahead.  
Sections are ordered by dependency: complete each before moving to the next.

---

## Phase 1 — Environment Check

Before anything else, verify the environment is ready.

### Step 1.1 — Run the doctor

```
Ask: "Have you already installed LazyQMK and cloned the custom QMK firmware fork?
      If not, I can walk you through that first."
```

**If not installed**, guide through installation:

1. Install LazyQMK:
   - macOS/Linux: `brew install Radialarray/lazyqmk/lazyqmk`
   - Other: download from https://github.com/Radialarray/LazyQMK/releases/latest
2. Clone the custom QMK fork (the official fork will not work — custom QMK features are required):
   ```bash
   git clone --recurse-submodules https://github.com/Radialarray/qmk_firmware.git ~/qmk_firmware
   ```

**Once installed**, run the doctor:

```bash
lazyqmk doctor
```

Expected healthy output:
```
✓ QMK CLI           v1.1.5
✓ ARM GCC           arm-none-eabi-gcc (11.3.1)
✓ AVR GCC           avr-gcc (5.4.0)
✓ QMK Firmware Path /Users/user/qmk_firmware
✓ Git Configuration Configured
```

If anything shows `✗` or `?`, fix those issues before continuing. Use `lazyqmk doctor --verbose` for details.

---

## Phase 2 — First Launch & Onboarding Wizard

### Step 2.1 — Start LazyQMK

```
Ask: "What keyboard are you setting up? (e.g. Corne crkbd, Ferris Sweep, Planck, etc.)"
```

Then launch:

```bash
lazyqmk
```

The **Onboarding Wizard** will ask for three things:

| Prompt | What to enter | Example |
|--------|--------------|---------|
| QMK Firmware Path | Path to your cloned QMK fork | `/Users/user/qmk_firmware` |
| Keyboard | QMK keyboard name | `crkbd/rev1` |
| Layout Variant | Physical layout ID from QMK | `LAYOUT_split_3x6_3` |

> **Tip for agents**: If the user is unsure of the keyboard/variant names, they can be found in the `info.json` file inside the QMK firmware folder at `keyboards/<keyboard>/info.json`.

**Alternatively, load an existing layout:**  
If the user already has a `.md` layout file, the welcome screen offers "Load Existing Layout" — detect and select it from the list.

---

## Phase 3 — Basic Layout Editing

### Step 3.1 — Navigation

```
Ask: "Would you like me to walk you through the basic editing flow, or do you already know your way around?"
```

If they need the basics:

| Action | Keyboard shortcut |
|--------|-------------------|
| Move cursor to a key | Arrow keys `↑↓←→` or VIM-style `h j k l` |
| Open keycode picker | `Enter` |
| Search for a keycode | Type to fuzzy-search (e.g. `ctrl`, `esc`, `layer`) |
| Assign keycode | `Enter` in the picker |
| Clear a key (set to transparent) | `x` or `Delete` |
| Switch to another layer | `Tab` / `Shift+Tab` |
| Open layer manager | `Shift+L` |
| Save layout | `Ctrl+S` |
| Build firmware | `Ctrl+B` |
| Show full help | `?` |

### Step 3.2 — Layer setup

```
Ask: "How many layers does your layout use? What are the layers for?
      (e.g. base layer, symbol layer, navigation layer, gaming layer)"
```

Common layer setup pattern:
- Layer 0: Base / QWERTY / Home
- Layer 1: Symbols / Numbers
- Layer 2: Navigation / Media
- Layer 3+: Application-specific or gaming

Use `Shift+L` to rename layers for clarity.

### Step 3.3 — Color organization (optional)

LazyQMK uses a **four-level color priority system**:

| Priority | Level | How to set |
|----------|-------|------------|
| 1 (highest) | Individual key override | Select key → `c` (set individual key color) |
| 2 | Key category color | Select key → `Ctrl+K` (assign category to key) |
| 3 | Layer category color | `Ctrl+L` (assign category to current layer) |
| 4 (lowest) | Layer default color | `Shift+C` (set layer default color) |

```
Ask: "Do you want to set up color coding for your layout?
      For example, you can color-code modifiers, navigation keys, symbols, etc."
```

To create and manage categories: `Shift+K` opens the Category Manager.

---

## Phase 4 — Settings Manager Overview

Most advanced features (combos, RGB effects, idle screensaver) live in the **Settings Manager**.

Open with: `Shift+S`

The Settings Manager is organized into groups:
- **Firmware [Global]** — QMK path, keyboard, layout variant
- **RGB Lighting [Layout]** — RGB effects, speed, overlay ripple
- **Idle Effect [Layout]** — Screensaver animation
- **Combos [Layout]** — Two-key hold combos
- **Tap Dance [Layout]** — Tap dance actions (also accessible via `Shift+D`)

Navigate with `↑↓` (or `j/k`), select an item with `Enter`, confirm edits with `Enter`, cancel with `Esc`.

> **Web UI alternative**: All settings are also accessible at `http://localhost:3001` after running `lazyqmk web`. The web editor uses a tabbed interface per layout.

---

## Phase 5 — RGB Enhancements

> **Prerequisites**: The keyboard must have RGB Matrix LEDs and `RGB_MATRIX_ENABLE = yes` in `rules.mk`. The QMK custom fork must be used. Without these, ripple code will compile but have no effect.

```
Ask: "Does your keyboard have RGB LEDs? Would you like to set up RGB effects?"
```

If yes, continue with steps 5.1 and 5.2.

---

### Step 5.1 — RGB Matrix Default Speed

This controls the animation speed for all RGB effects.

**Range**: 0 (slowest) → 127 (default/mid) → 255 (fastest)

#### TUI (Settings Manager)
1. Open Settings Manager: `Shift+S`
2. Navigate to **"RGB Lighting [Layout]"** group
3. Select **"RGB Matrix Speed"**
4. Type a number or use `↑↓` to adjust, then `Enter` to confirm

#### Web UI
1. Navigate to the layout editor for your layout
2. Find the **RGB settings panel**
3. Adjust the **RGB Matrix Speed** field

**What gets generated in `config.h`** (only if changed from default 127):
```c
// RGB Matrix Default Animation Speed
#define RGB_MATRIX_DEFAULT_SPD 200
```

**What gets written to the layout `.md` file**:
```markdown
**RGB Matrix Speed**: 200
```

---

### Step 5.2 — RGB Overlay Ripple Effect

A ripple effect overlaid on top of your base layer colors. When a key is pressed, a circular ripple expands outward from that key's LED position.

```
Ask: "Would you like to enable the RGB overlay ripple effect?
      It creates an expanding ring of light from each keypress, layered on top of your normal colors."
```

#### TUI (Settings Manager)
1. Open Settings Manager: `Shift+S`
2. Navigate to **"RGB Lighting [Layout]"** group
3. Select **"Overlay Ripple Enabled"** → `Enter` to toggle **On**
4. Configure the settings below as desired

#### Web UI
1. Navigate to the layout editor
2. Click the **Overlay Ripple tab** (🌊 icon)
3. Toggle **"Enable Overlay Ripple"**
4. Configure settings in the panel

#### All ripple settings

```
Ask each of these in turn, showing the default and range:
```

| Setting | Default | Range | Description |
|---------|---------|-------|-------------|
| **Max Concurrent Ripples** | 4 | 1–8 | How many ripples can be active at once |
| **Ripple Duration** | 500ms | any ms | How long each ripple lasts before fading out |
| **Ripple Speed** | 128 | 0–255 | How fast the ring expands outward |
| **Band Width** | 3 | LED units | How wide the ring is |
| **Amplitude** | 50% | 0–100% | Brightness boost of the ripple over base color |
| **Color Mode** | Fixed | see below | How ripple color is determined |
| **Fixed Color** | `#00FFFF` (cyan) | hex color | Color used when mode is Fixed Color |
| **Trigger on Press** | On | On/Off | Fire ripple when key is pressed |
| **Trigger on Release** | Off | On/Off | Fire ripple when key is released |
| **Ignore Transparent Keys** | On | On/Off | Skip keys assigned KC_TRNS |
| **Ignore Modifier Keys** | Off | On/Off | Skip Shift, Ctrl, Alt, GUI keys |
| **Ignore Layer Switch Keys** | Off | On/Off | Skip MO/LT/TG keys |

#### Color Mode options

| Mode | Description | Extra setting |
|------|-------------|---------------|
| **Fixed Color** | All ripples use one fixed color | Set **Ripple Fixed Color** (hex, e.g. `#00FFFF`) |
| **Key Color** | Ripple color matches the key's base layer color | — |
| **Hue Shift** | Ripple shifts hue relative to the key's base color | Set **Hue Shift** (-180° to 180°) |

> **Note**: Hue Shift mode currently falls back to Key Color behavior — this is a known limitation.

#### What gets generated

**In `config.h`:**
```c
// RGB Overlay Ripple Configuration
#define LQMK_RIPPLE_OVERLAY_ENABLED
#define LQMK_RIPPLE_MAX_RIPPLES 4
#define LQMK_RIPPLE_DURATION_MS 500
#define LQMK_RIPPLE_SPEED 128
#define LQMK_RIPPLE_BAND_WIDTH 3
#define LQMK_RIPPLE_AMPLITUDE_PCT 50
#define LQMK_RIPPLE_TRIGGER_ON_PRESS 1
#define LQMK_RIPPLE_TRIGGER_ON_RELEASE 0
```

**In `keymap.c`:**  
A `ripple_t` array is declared, and `rgb_matrix_indicators_advanced_user()` is hooked to apply the ripple effect on top of the base layer colors each frame. The hook uses LED coordinates to calculate distance from the ripple origin and fades out over time.

**In the `.md` layout file** (only non-default values are written; this example shows Fixed Color mode):
```markdown
**Ripple Overlay**: On
**Max Ripples**: 6
**Ripple Duration**: 750ms
**Ripple Speed**: 200
**Ripple Band Width**: 5
**Ripple Amplitude**: 75%
**Ripple Color Mode**: Fixed Color
**Ripple Fixed Color**: #FF00FF
**Ripple Trigger on Press**: Off
**Ripple Trigger on Release**: On
**Ripple Ignore Transparent**: Off
**Ripple Ignore Modifiers**: On
**Ripple Ignore Layer Switch**: On
```

> **Color mode values in `.md` file**: `Fixed Color`, `Key Color`, `Hue Shift`.  
> The `Ripple Fixed Color` field is only relevant when mode is `Fixed Color`.  
> The `Ripple Hue Shift` field is only relevant when mode is `Hue Shift`.

---

## Phase 6 — Two-Key Hold Combos

A "two-key hold combo" fires a special action when two specific keys on the **base layer (layer 0)** are held simultaneously for a configurable duration. Up to **3 combos** can be defined per layout.

Each combo slot has a fixed action:
- **Combo 1** → Disable RGB Effects (revert to base layer colors)
- **Combo 2** → Disable RGB Lighting entirely
- **Combo 3** → Enter Bootloader (for flashing firmware)

```
Ask: "Would you like to set up two-key hold combos?
      These let you trigger actions (like disabling RGB or entering bootloader mode)
      by holding two keys together for a moment — no extra keys needed."
```

> **Important constraints**:
> - Combos only fire when you are on **layer 0** (base layer)
> - Both keys must be different physical positions
> - Max 3 combos total
> - Hold duration: 50–2000ms

---

### Step 6.1 — Enable combos

#### TUI (Settings Manager)
1. Open Settings Manager: `Shift+S`
2. Navigate to **"Combos [Layout]"** group
3. Select **"Combos Enabled"** → `Enter` to toggle **On**

#### Web UI
1. Navigate to the layout editor
2. Click the **Combos tab** (🔗 icon)
3. Toggle **"Enable Combos"** checkbox

---

### Step 6.2 — Configure each combo

For each combo you want to enable:

```
Ask: "For Combo [1/2/3] ([action name]):
      - Which key should be Key 1? (navigate to it on the keyboard grid)
      - Which key should be Key 2?
      - How long should both keys be held before it fires? (default: 500ms)"
```

#### TUI flow — picking key positions

Each combo has three settings: **Key 1**, **Key 2**, and **Hold Duration**.

1. Select **"Combo N Key 1 ([Action])"** → `Enter`
   - The editor enters key-selection mode
   - Use arrow keys to navigate the keyboard grid to the desired physical key
   - Press `Enter` to confirm the selection
2. Repeat for **"Combo N Key 2 ([Action])"**
3. Select **"Combo N Hold Duration"** → type a number (ms) or use `↑↓` to adjust by 10ms increments → `Enter`

Setting display names in TUI:
- `Combo 1 Key 1 (Disable Effects)` / `Combo 1 Key 2 (Disable Effects)` / `Combo 1 Hold Duration`
- `Combo 2 Key 1 (Disable Lighting)` / `Combo 2 Key 2 (Disable Lighting)` / `Combo 2 Hold Duration`
- `Combo 3 Key 1 (Bootloader)` / `Combo 3 Key 2 (Bootloader)` / `Combo 3 Hold Duration`

#### Web UI flow

1. Click **"Add Combo"** button (max 3)
2. For each combo row, fill in:
   - **Key 1** row and column (matrix coordinates)
   - **Key 2** row and column
   - **Action**: `DisableEffects` / `DisableLighting` / `Bootloader`
   - **Hold Duration** (ms)
3. Remove unwanted combos with the delete button on each row

> **How to find matrix coordinates**: In the TUI, enable matrix position display in the status bar (see debug mode). Alternatively, check the keyboard's `info.json` at `keyboards/<keyboard>/info.json` for the `matrix` field under each key.

---

### Step 6.3 — What gets generated

**In `config.h`:**
```c
// Combo Configuration
#define COMBO_ENABLE
#define COMBO_COUNT 2
```

> **Important**: You must also add `COMBO_ENABLE = yes` to your `rules.mk` file for QMK to compile combo support. LazyQMK generates this alongside `config.h` when combos are enabled.

**In `keymap.c`** (example with 2 combos):
```c
#ifdef COMBO_ENABLE

enum combo_events {
    COMBO_0,
    COMBO_1
};

const uint16_t PROGMEM combo_0_keys[] = {KC_A, KC_B, COMBO_END};
const uint16_t PROGMEM combo_1_keys[] = {KC_C, KC_D, COMBO_END};

combo_t key_combos[] = {
    [COMBO_0] = COMBO_ACTION(combo_0_keys),
    [COMBO_1] = COMBO_ACTION(combo_1_keys),
};

void process_combo_event(uint16_t combo_index, bool pressed) {
    // Only activate combos on base layer (layer 0)
    if (get_highest_layer(layer_state) != 0) {
        return;
    }
    if (pressed) {
        combo_state[combo_index].timer = timer_read();
        combo_state[combo_index].active = true;
    } else {
        if (combo_state[combo_index].active) {
            uint16_t elapsed = timer_elapsed(combo_state[combo_index].timer);
            switch (combo_index) {
                case COMBO_0:
                    if (elapsed >= 500) {
                        // Disable RGB effects — reverts to TUI layer colors (or solid color as fallback)
                        // Actual emitted call depends on whether the layout has custom layer colors:
                        //   rgb_matrix_mode_noeeprom(RGB_MATRIX_TUI_LAYER_COLORS);  // if custom colors defined
                        //   rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);       // otherwise
                        rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);
                    }
                    break;
                case COMBO_1:
                    if (elapsed >= 800) {
                        rgb_matrix_disable_noeeprom();
                    }
                    break;
            }
            combo_state[combo_index].active = false;
        }
    }
}

#endif // COMBO_ENABLE
```

**In the `.md` layout file:**
```markdown
**Combos**: On
**Combo 1**: (0,0)+(0,1) → Disable Effects [500ms]
**Combo 2**: (0,5)+(0,6) → Disable Lighting [800ms]
**Combo 3**: (1,0)+(1,11) → Bootloader [1000ms]
```

---

## Phase 7 — Build & Flash

Once all settings are configured:

```
Ask: "Are you ready to build the firmware and flash it to your keyboard?"
```

### Step 7.1 — Save

```
Ctrl+S
```

The title bar asterisk (`*`) disappears when the layout is saved.

### Step 7.2 — Build

```
Ctrl+B
```

- Build runs in the background (non-blocking)
- Watch the progress in the status bar
- Press `Shift+B` to open the build log viewer
- Press `Ctrl+C` inside the log viewer to copy the log to clipboard

If the build fails, the error message will suggest running `lazyqmk doctor --verbose` for diagnostics.

### Step 7.3 — Flash

Once the build succeeds, put your keyboard into bootloader mode:
- Use **Combo 3** (Bootloader combo) if configured
- Or use a physical RESET button / BOOT+USB shortcut (keyboard-specific)

Then flash with QMK Toolbox or the QMK CLI:
```bash
qmk flash
```

---

## Phase 8 — Enhance an Existing Layout

```
Ask: "Do you have an existing layout file (.md) you want to enhance,
      or are you starting from scratch?"
```

For existing layouts, the quickest path is:

1. **Launch with the existing file:**
   ```bash
   lazyqmk --layout /path/to/your/layout.md
   ```
   Or use the "Load Existing Layout" option on the welcome screen.

2. **Review current settings:**
   - Open Settings Manager (`Shift+S`) to see all current settings
   - Check which features are enabled/configured

3. **Apply enhancements** using the relevant phases above:
   - Add combos → Phase 6
   - Add ripple effects → Phase 5.2
   - Tune RGB speed → Phase 5.1
   - Add/modify layers → Phase 3.2

4. **Save and rebuild** → Phase 7

---

## Quick Reference Card

| Want to... | TUI shortcut | Web tab |
|------------|-------------|---------|
| Open Settings Manager | `Shift+S` | All settings visible in editor |
| Configure combos | `Shift+S` → Combos group | 🔗 Combos tab |
| Configure ripple effect | `Shift+S` → RGB Lighting group | 🌊 Overlay Ripple tab |
| Set RGB speed | `Shift+S` → RGB Lighting group | RGB settings panel |
| Set up tap dance | `Shift+D` or `Shift+S` → Tap Dance | Tap Dance tab |
| Open keycode picker | `Enter` on a key | Click a key |
| Manage categories | `Shift+K` | Category panel |
| Export layout doc | `Ctrl+E` | Export button |
| Full shortcut list | `?` | — |

---

## Troubleshooting

### Combos not firing
- Verify you are on **layer 0** when holding the keys
- Check hold duration — increase it if accidental activations, decrease if it feels slow
- Confirm `COMBO_ENABLE = yes` is in `rules.mk` and `#define COMBO_ENABLE` is in `config.h`
- If combos are enabled but no combo definitions exist, no combo code is generated — at least one combo must be fully configured

### Ripple effect not visible
- Ensure `RGB_MATRIX_ENABLE` is defined and the keyboard supports RGB Matrix
- Check that **Trigger on Press** is enabled
- Try increasing **Amplitude** (the brightness boost may be too low)
- Verify the custom QMK fork is being used

### RGB speed change not applied
- The default (127) is not written to `config.h` — only non-default values emit the define. This is correct.

### Build errors
```bash
lazyqmk doctor --verbose
```
Follow the suggestions printed alongside each error.

---

## File Format Reference

LazyQMK layouts are stored as `.md` files with YAML frontmatter. All feature settings live in a `## Settings` section.

### Key syntax in layer tables

| Format | Meaning |
|--------|---------|
| `KC_A` | Plain keycode |
| `KC_A{#FF0000}` | Keycode with color override |
| `KC_A@navigation` | Keycode with category |
| `KC_A{#FF0000}@navigation` | Keycode with color and category |

### Settings section example (all features)

```markdown
## Settings

**RGB Matrix Speed**: 180
**Ripple Overlay**: On
**Max Ripples**: 4
**Ripple Duration**: 500ms
**Ripple Speed**: 200
**Ripple Band Width**: 3
**Ripple Amplitude**: 60%
**Ripple Color Mode**: Fixed Color
**Ripple Fixed Color**: #00FFFF
**Combos**: On
**Combo 1**: (0,2)+(0,3) → Disable Effects [500ms]
**Combo 3**: (0,0)+(2,5) → Bootloader [1000ms]
```

> Only non-default values are written. Missing settings use their built-in defaults.
