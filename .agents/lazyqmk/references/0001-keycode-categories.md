# Reference 0001: Keycode Categories

> The QMK keycode database embedded in LazyQMK. 21 loaded core categories + 10 language modules. Total 859 keycodes.

## Discovering Keycodes

```bash
# All keycodes (table output)
lazyqmk keycodes

# One category
lazyqmk keycodes --category navigation

# JSON for scripting
lazyqmk keycodes --category modifiers --json | jq '.keycodes[] | .code'
```

## Built-in Categories

| ID | Name | Examples |
|---|---|---|
| `basic` | Basic Keys | KC_A-KC_Z, KC_0-KC_9, KC_ENT, KC_ESC, KC_TAB, KC_SPC |
| `navigation` | Navigation | KC_UP, KC_DOWN, KC_LEFT, KC_RGHT, KC_HOME, KC_END, KC_PGUP, KC_PGDN |
| `media` | Media | KC_MPLY, KC_MSTP, KC_MNXT, KC_MPRV, KC_VOLU, KC_VOLD, KC_MUTE |
| `function` | Function | KC_F1-KC_F12 |
| `modifiers` | Modifiers | KC_LSFT, KC_RSFT, KC_LCTL, KC_RCTL, KC_LALT, KC_RALT, KC_LGUI, KC_RGUI |
| `mod_tap` | Mod-Tap | LCTL_T(), LALT_T(), LSFT_T(), LGUI_T() and right-hand variants |
| `mod_combo` | Mod-Combo | C_S(), C_S_T(), etc. |
| `one_shot` | One-Shot Mod | OS_LSFT, OS_LCTL, OS_LALT, OS_LGUI, OSL() |
| `layers` | Layer Switching | MO(), TG(), TO(), TT(), LT(), LM(), DF() |
| `tap_dance` | Tap Dance | TD(<name>) — custom definitions in layout |
| `mouse` | Mouse | MS_UP, MS_DOWN, MS_LEFT, MS_RGHT, MS_BTN1-8, MS_WHLU, MS_WHLD |
| `system` | System | QK_BOOT, QK_RBT, EE_CLR, RGB_TOG, BL_TOG, BL_UP, BL_DOWN |
| `backlight` | Backlight | BL_TOG, BL_STEP, BL_ON, BL_OFF, BL_UP, BL_DOWN |
| `bluetooth` | Bluetooth | BT_SEL, BT_NEXT, OUT_USB, OUT_BT (only on BT-capable boards) |
| `audio` | Audio | AU_TOG, MU_TOG, MU_NEXT (if QMK audio enabled) |
| `haptic` | Haptic | HPT_TOG, HPT_DWL_*, HPT_BUZ_* (if haptic feedback enabled) |
| `international` | International | (Language-neutral symbols) |
| `shifted` | Shifted symbols | KC_LPRN (shifted 9), KC_RPRN (shifted 0), etc. |
| `symbols` | Symbols | KC_EXLM, KC_AT, KC_HASH, KC_DLR, KC_PERC, KC_CIRC, KC_AMPR, KC_ASTR |
| `magic` | Magic | KC_CAPSLOCK, KC_NUMLOCK, KC_LCAP, KC_LNUM |
| `rgb` | RGB Lighting | RGB_TOG, RGB_MOD, RGB_HUI, RGB_HUD, RGB_SAI, RGB_SAD, RGB_VAI, RGB_VAD, RGB_SPI, RGB_SPD |
| `joystick` | Joystick | JS_BUTTON, JS_AXIS (only on joystick-capable boards) |

## Language Categories

LazyQMK bundles 10 language modules. Each adds a `DE_`, `FR_`, etc. prefix to common keycodes (using `keymap_extras/` from QMK).

CLI category IDs are prefixed with `lang_` (use `lazyqmk keycodes --category lang_german`, not `--category german`):

| CLI ID | Prefix | Header | Language |
|---|---|---|---|
| `lang_german` | `DE_` | `keymap_extras/keymap_german.h` | DE |
| `lang_german_mac` | `DE_` | `keymap_extras/keymap_german_mac_iso.h` | DE Mac ISO |
| `lang_french` | `FR_` | `keymap_extras/keymap_french.h` | FR |
| `lang_french_mac` | `FR_` | `keymap_extras/keymap_french_mac_iso.h` | FR Mac ISO |
| `lang_spanish` | `ES_` | `keymap_extras/keymap_spanish.h` | ES |
| `lang_italian` | `IT_` | `keymap_extras/keymap_italian.h` | IT |
| `lang_danish` | `DK_` | `keymap_extras/keymap_danish.h` | DK |
| `lang_norwegian` | `NO_` | `keymap_extras/keymap_norwegian.h` | NO |
| `lang_swedish` | `SE_` | `keymap_extras/keymap_swedish.h` | SE |
| `lang_uk` | `UK_` | `keymap_extras/keymap_uk.h` | UK |

```bash
# Example: list German umlaut keycodes (use lang_german, not german)
lazyqmk keycodes --category lang_german --json | jq -r '.keycodes[] | "\(.code) - \(.name)"' | head -20
```

Note: `bluetooth`, `haptic`, and `joystick` JSON files exist in `src/keycode_db/categories/` but are not currently loaded by `KeycodeDb::load()` — they don't appear in `lazyqmk keycodes` output.

Notable German keycodes: `DE_Y`, `DE_Z` (swapped), `DE_UDIA`, `DE_ADIA`, `DE_ODIA`, `DE_SS`, `DE_CIRC`, `DE_EURO`, `DE_LABK`, `DE_RABK`, `DE_LBRC`, `DE_RBRC`, `DE_LCBR`, `DE_RCBR`, `DE_LPRN`, `DE_RPRN`, `DE_PIPE`, `DE_BSLS`, `DE_SLSH`, `DE_QUES`, `DE_EXLM`, `DE_AMPR`, `DE_AT`, `DE_DLR`, `DE_PERC`, `DE_TILD`, `DE_HASH`, `DE_MINS`, `DE_EQL`, `DE_QUOT`, `DE_DQUO`, `DE_GRV`, `DE_ACUT`, `DE_LESS`, etc.

## Parameterized Keycodes (4 types)

LazyQMK's keycode picker differentiates by parameter type:

### 1. `keycode` — opens keycode picker

```c
MT(MOD_LCTL, KC_A)         // Mod-Tap: Ctrl when held, A when tapped
OS_LSFT                    // One-Shot Shift
LCTL(KC_C)                 // Ctrl modifier combo
LCG(KC_Q)                  // Ctrl+Alt+Gui combo
```

### 2. `layer` — opens layer picker

```c
MO(1)                      // Momentary layer 1 (active while held)
TG(2)                      // Toggle layer 2 (sticky, on/off)
TO(3)                      // Switch exclusively to layer 3 (and stay)
TT(4)                      // Tap-Toggle layer 4 (5 taps = lock)
OSL(5)                     // One-Shot layer 5 (next key only)
DF(0)                      // Set default layer to 0
```

UUID-based (preferred for portability):

```c
MO(@<layer-uuid>)          // Resolves to current layer index at validate time
LT(@<layer-uuid>, KC_SPC)  // Layer-Tap: hold for layer, tap for Space
LM(@<layer-uuid>, MOD_LSFT) // Layer-Mod: layer + mod active simultaneously
```

### 3. `modifier` — opens modifier picker (for `MT()` / `LM()`)

Valid: `MOD_LCTL`, `MOD_LSFT`, `MOD_LALT`, `MOD_LGUI`, `MOD_RCTL`, `MOD_RSFT`, `MOD_RALT`, `MOD_RGUI`, `MOD_HYBRID`, `MOD_BIT(KC_X)`, etc.

### 4. `tapdance` — opens tap dance picker (for `TD()`)

References a tap dance action defined in the layout's `tap_dances` array. Auto-created with `KC_NO` placeholder if referenced but not defined.

## Mod-Tap Naming Convention

QMK provides pre-named mod-taps for common cases:

```c
LCTL_T(KC_A)               // Ctrl when held, A when tapped
LSFT_T(KC_B)               // Shift when held, B when tapped
LALT_T(KC_C)               // Alt when held, C when tapped
LGUI_T(KC_D)               // Gui when held, D when tapped
RCTL_T(KC_E)
RSFT_T(KC_F)
RALT_T(KC_G)
RGUI_T(KC_H)
```

For more flexibility: `MT(MOD_LCTL | MOD_LSFT, KC_SPC)` — two mods at once.

## Layer-Tap (LT)

`LT(layer, kc)` — tap for `kc`, hold for `layer`.

```c
LT(1, KC_SPC)              // Space on tap, layer 1 on hold
LT(2, KC_TAB)              // Tab on tap, layer 2 on hold
LT(@<uuid>, KC_ENT)        // UUID form (preferred)
```

Behavior controlled by `tap_hold_settings` (see `references/0006-tap-hold-and-combos.md`):
- `tapping_term` (default 200ms) — max hold time for tap
- `hold_mode` — when other keys are involved (Default / PermissiveHold / HoldOnOtherKeyPress)
- `retro_tapping` — tap action fires even if held past tapping_term
- `flow_tap_term` — anti-flicker during fast typing
- `chordal_hold` — opposite-hand rule for tap-hold decision

## QMK Reserved Prefixes

Any keycode starting with these prefixes is parsed specially:

- `KC_` — basic keys, modifiers, special keys
- `MO/TG/TO/TT/OSL/DF/LT/LM` — layer switching (parameter: layer index or `@<uuid>`)
- `LCTL/LSFT/LALT/LGUI/RCTL/RSFT/RALT/RGUI` — modifier combos (parameter: keycode)
- `LCG/LCA/LSG/LCSG/MEH/HYPR` — multi-modifier combos
- `MT/OS_/C()/S()/SG()/LCG()/...` — other parameterized
- `DE_/FR_/...` — language keycodes
- `MS_` — mouse keys
- `RGB_` — RGB control
- `BL_` — backlight control
- `QK_` — QMK special (QK_BOOT for bootloader, QK_RBT for reboot)
- `AU_/MU_` — audio
- `TD(<name>)` — tap dance (custom, name must be defined in `tap_dances[]`)
- `USER<N>` — custom user keycodes (not in LazyQMK db; must add to keyboard's `keymap.c`)

## Common Pitfalls

- **LT with KC_TRNS or KC_NO** — semantically broken; the tap action must be a real keycode
- **TD() references auto-created on parse** — if you reference `TD(foo)` but never define it, LazyQMK creates a placeholder `TD(foo) = single: KC_NO`. You must define it before generating.
- **Language keycodes need the language module included** — the generator adds `#include "keymap_extras/keymap_<lang>.h"` to `keymap.c` automatically (no `rules.mk` flag needed). The keyboard's `info.json` must already allow this include.
- **Mouse keys (`MS_*`) require `MOUSEKEY_ENABLE = yes`** in the keyboard's `rules.mk` or `info.json` `features.mousekey: true` — LazyQMK does NOT auto-add this to `rules.mk`. Verify the keyboard has mousekey support before adding `MS_*` keys.
- **RGB keys (`RGB_*`) require `RGB_MATRIX_ENABLE = yes`** — LazyQMK does NOT auto-add this. Verify the keyboard's `info.json` has `features.rgb_matrix: true`. The generator wraps RGB code in `#ifdef RGB_MATRIX_ENABLE`, so missing this define produces empty output (no errors, but no RGB either).