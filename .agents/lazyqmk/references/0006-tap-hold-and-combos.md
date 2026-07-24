# Reference 0006: Tap-Hold and Combos

> Tap-hold config (5 presets) controls how `LT()`, `MT()`, `TT()` behave. Combos are two-key holds that fire one of 3 hardcoded actions (DisableEffects, DisableLighting, Bootloader). Up to 32 combos per layout.

## Tap-Hold Settings (JSON)

```json
{
  "tap_hold_settings": {
    "tapping_term": 200,
    "quick_tap_term": null,
    "hold_mode": "Default",
    "retro_tapping": false,
    "tapping_toggle": 5,
    "flow_tap_term": null,
    "chordal_hold": false,
    "preset": "Default"
  }
}
```

| Setting | Type | Range | Default | Description |
|---|---|---|---|---|
| `tapping_term` | u16 (ms) | 50–1000 | 200 | Max hold time to count as a tap. |
| `quick_tap_term` | u16 (ms) | 0–1000, null | null | Auto-repeat window. Tap then hold within this = repeat the tap (no hold action). Null = same as tapping_term. |
| `hold_mode` | enum | — | Default | When other keys are pressed during hold: Default (timing), PermissiveHold, HoldOnOtherKeyPress. |
| `retro_tapping` | bool | — | false | Send tap action even if held past tapping_term (when no other key pressed). |
| `tapping_toggle` | u8 | 1–10 | 5 | Number of taps required for `TT()` to toggle layer. |
| `flow_tap_term` | u16 (ms) | 0–500, null | null | Anti-flicker window for fast typing. Tap during this window doesn't trigger hold. |
| `chordal_hold` | bool | — | false | Opposite-hand rule: same-hand combo = tap, opposite-hand = hold. Great for HRM. |
| `preset` | enum | — | Default | Which preset these settings come from. Set to Custom if manually tweaked. |

## 5 Tap-Hold Presets

| Preset | Use case | Settings |
|---|---|---|
| **Default** | Conservative, beginner-friendly | tapping_term 200, hold_mode Default, no extras |
| **HomeRowMods** | Home-row mods on home row | tapping_term 175, quick_tap 120, hold_mode PermissiveHold, retro_tapping true, flow_tap 150, chordal_hold true |
| **Responsive** | Gaming / fast modifier access | tapping_term 150, quick_tap 100, hold_mode HoldOnOtherKeyPress, no extras |
| **Deliberate** | Requires intentional holds | tapping_term 250, no extras, conservative |
| **Custom** | User has manually tweaked | (whatever the user set) |

### Preset Detail: HomeRowMods (recommended for productivity)

```json
{
  "tapping_term": 175,
  "quick_tap_term": 120,
  "hold_mode": "PermissiveHold",
  "retro_tapping": true,
  "tapping_toggle": 5,
  "flow_tap_term": 150,
  "chordal_hold": true,
  "preset": "HomeRowMods"
}
```

This preset combines 5 features that prevent accidental modifier activation during fast typing:

- **PermissiveHold** — hold fires when another key is tapped during hold
- **retro_tapping** — tap fires even if held past tapping_term (no other key)
- **flow_tap_term: 150** — fast typing doesn't trigger hold
- **chordal_hold** — opposite-hand rule makes same-hand combo = tap, opposite-hand = hold
- **quick_tap_term: 120** — tap-then-hold = repeat, not hold

### Choosing a Preset

| Your situation | Recommended preset |
|---|---|
| Beginner, first time with LT/MT | Default |
| Using `LSFT_T(KC_A)` etc. on home row | HomeRowMods |
| Gaming (instant modifier access) | Responsive |
| Frustrated by accidental mods | Deliberate → HomeRowMods |
| Tweaking values one by one | Custom (start from preset, edit) |

## Hold Decision Modes (3 options)

| Mode | Description | When it fires hold |
|---|---|---|
| Default | Timing only | Hold past tapping_term |
| PermissiveHold | + tap detection | Hold past tapping_term OR when another key is tapped during hold |
| HoldOnOtherKeyPress | + press detection | Hold past tapping_term OR when another key is pressed during hold |

PermissiveHold and HoldOnOtherKeyPress make home-row mods more usable during fast typing but can cause unintended holds. Try HomeRowMods preset first.

## Combos (Up to 32, 3 Hardcoded Actions)

Combos are two-key holds on layer 0 that fire special actions after a hold duration.

### Combo Structure (JSON)

```json
{
  "combo_settings": {
    "enabled": true,
    "combos": [
      {
        "key1": { "row": 0, "col": 2 },
        "key2": { "row": 0, "col": 3 },
        "action": { "type": "disable_effects" },
        "hold_duration_ms": 500
      }
    ]
  }
}
```

`action` is internally tagged (snake_case), NOT a string. The `placeholder` field is `#[serde(skip)]` and never appears in persisted JSON.

### Fixed Actions (only 3)

| Action | JSON value (`action.type`) | What it does |
|---|---|---|
| Disable RGB Effects | `disable_effects` | Revert to base layer colors |
| Disable RGB Lighting | `disable_lighting` | Toggle RGB on/off (calls `rgb_matrix_disable_noeeprom`) |
| Bootloader | `bootloader` | Enter bootloader for flashing |

No custom actions — these 3 are hardcoded in `keymap.c`.

### Combo Rules

- **Layer 0 only** — combos only fire when both keys are on the base layer
- **2 keys must differ** — `key1 != key2`
- **Up to 32 combos** — model limit is `MAX_COMBOS = 32` (`src/models/layout/combo.rs:7`); the validator rejects more. No reason to need this many, but it works.
- **Each combo uses one of 3 hardcoded actions** — `DisableEffects`, `DisableLighting`, `Bootloader`. No custom actions are supported.
- **Hold duration**: 50–2000ms (default 500)
- **Keys in visual coordinates** — `{row, col}` as used in layout
- **If no combos defined, no code emitted** — even if `enabled: true`
- **Activation timing** — action fires on key RELEASE after elapsed duration, not while both keys are held. Tell user: hold both keys for `hold_duration_ms`, then release to trigger.

### Generator Quirks

**The combo generator (`src/firmware/generator/combo.rs:70-77`) uses the raw keycode at the combo's positions.** It looks up `base_layer.get_key(combo.key1).keycode` and inserts that string verbatim into `const uint16_t PROGMEM combo_X_keys[]`. Implications:

- If the base layer key at `key1` or `key2` is a parameterized expression like `LT(@<uuid>, KC_W)`, the generator will emit that expression unchanged into the combo array, producing invalid C (`combo_0_keys[]` is `uint16_t[]`, not strings). QMK won't compile.
- Workaround: only place combos on base-layer keys with **simple** keycodes (`KC_A`, `MO(1)`, etc.). Avoid placing combos on `LT()`, `MT()`, `LCTL_T()`, `TD()`, or other parameterized positions.

**Direct JSON edits bypass combo safeguards.** `Layout::validate()` does NOT call `ComboSettings::add_combo`, so the following checks are skipped if you edit `combos[]` directly via jq:

- Max 32 combos (`add_combo` enforces)
- Distinct `key1 != key2` (`add_combo` enforces)
- Hold duration 50–2000ms (`add_combo` enforces)
- Duplicate key pairs (`add_combo` enforces)
- Valid visual position (must exist in layer 0)

Use `lazyqmk combo add --layout <path> --key1 ...` to get these checks. Or wrap your jq edit with a preflight (see Recipes below).

### Combo Generated `config.h` and `rules.mk`

`config.h` (only combo count):

```c
#define COMBO_COUNT N  // N = number of configured combos
```

`rules.mk` (only when combos enabled AND at least one defined):

```makefile
COMBO_ENABLE = yes
```

Note: `COMBO_ENABLE = yes` lives in `rules.mk`, NOT `config.h`. Don't emit `#define COMBO_ENABLE` in `config.h` — QMK's build system expects it in `rules.mk`.

### Combo Generated `keymap.c`

```c
#ifdef COMBO_ENABLE

enum combo_events { COMBO_0, COMBO_1, COMBO_2 };

const uint16_t PROGMEM combo_0_keys[] = {KC_W, KC_E, COMBO_END};
// ... etc

combo_t key_combos[] = {
    [COMBO_0] = COMBO_ACTION(combo_0_keys),
    // ... etc
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
                    rgb_matrix_mode_noeeprom(RGB_MATRIX_SOLID_COLOR);
                }
                break;
            // ... etc for DisableLighting, Bootloader
        }
        combo_state[combo_index].active = false;
    }
}

#endif
```

## CLI Operations

```bash
# Read tap-hold settings
jq '.tap_hold_settings' ~/.../my.json

# Read combo settings
jq '.combo_settings' ~/.../my.json
```

## Recipes

### Apply HomeRowMods preset

```bash
jq '.tap_hold_settings = {
  tapping_term: 175,
  quick_tap_term: 120,
  hold_mode: "PermissiveHold",
  retro_tapping: true,
  tapping_toggle: 5,
  flow_tap_term: 150,
  chordal_hold: true,
  preset: "HomeRowMods"
}' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Configure a bootloader combo (e.g., W+E for 800ms)

```bash
jq '.combo_settings = {
  enabled: true,
  combos: [{
    key1: {row: 0, col: 2},
    key2: {row: 0, col: 3},
    action: {type: "bootloader"},
    hold_duration_ms: 800
  }]
}' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

(`placeholder` is `#[serde(skip)]` and never appears in persisted JSON. `action` is internally tagged snake_case.)

### Three combos (disable_effects, disable_lighting, bootloader)

```bash
jq '.combo_settings = {
  enabled: true,
  combos: [
    { key1: {row:0,col:0}, key2: {row:0,col:1}, action: {type: "disable_effects"},  hold_duration_ms: 500 },
    { key1: {row:1,col:0}, key2: {row:1,col:1}, action: {type: "disable_lighting"}, hold_duration_ms: 500 },
    { key1: {row:0,col:2}, key2: {row:0,col:3}, action: {type: "bootloader"},        hold_duration_ms: 1000 }
  ]
}' ~/.../my.json > ~/.../my.new.json
mv ~/.../my.new.json ~/.../my.json
```

### Find visual coordinates of a key (for combo key1/key2)

```bash
# Show all keys in layer 0 with their (row, col) and keycode
jq '.layers[0].keys | map({pos: .position, kc: .keycode}) | .[0:10]' ~/.../my.json
```

For example, `W` key on a Corne (visual left half, top row, 2nd column) = `{row: 0, col: 2}`.

## Tips for Polished Tap-Hold Setup

1. **Start with Default preset** — get a feel for LT/MT first
2. **Move to HomeRowMods** — once comfortable with home-row mods
3. **If you get accidental modifiers** — reduce `flow_tap_term` (150 → 100) or increase `tapping_term` (175 → 200)
4. **If home-row mods feel sluggish** — enable `chordal_hold` and reduce `tapping_term`
5. **Always enable bootloader combo** — saves you from physically pressing reset button

## Common Pitfalls

- **`tapping_term < 50` rejected** — minimum 50ms to be human-detectable
- **`tapping_term > 1000` rejected** — over 1s feels broken
- **`hold_mode` value must match enum exactly** — `"PermissiveHold"` not `"permissive_hold"`
- **`preset` value must match enum exactly** — `"HomeRowMods"` not `"home_row_mods"`
- **Combo `key1 == key2` rejected** — keys must differ
- **Combo `hold_duration_ms < 50` or > 2000** rejected
- **Combo `placeholder: true` is auto-set for missing slots** — never set this manually; it's internal
- **Combos only work on layer 0** — if the keys are pressed while on layer 1, combo won't fire
- **No combo code generated if zero combos defined** — `enabled: true` with empty `combos: []` = no `keymap.c` combo code