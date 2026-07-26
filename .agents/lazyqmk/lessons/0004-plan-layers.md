# Lesson 0004: Plan Layers (Phase 4)

> Decide which layers to create beyond Base. End: layer count + names locked, NOTES.md updated.

## Goal

- Number of layers decided (typically 4–8)
- Names for each layer
- Empty layer skeletons added to layout (all `KC_TRNS`)

## Discovery Gate (silent)

```bash
LAYOUT=~/.../my.json

# Current state
lazyqmk inspect --layout "$LAYOUT" --section metadata --json
lazyqmk inspect --layout "$LAYOUT" --section layers --json
```

## User Questions

| Ask | Default if "you decide" |
|---|---|
| "How many layers do you need?" | 4-5 (Base + Symbols + Navigation + Numbers + 1 more) |
| "What is each layer for?" | Suggest standard set, let user customize |
| "Any custom layers (gaming, app-specific)?" | None |
| **"Which host OS?"** (carried over from Mission Gate) | Reuse from MISSION.md if present. If you skipped the question earlier in Phase 3, ask now. Gates which media/launcher/brightness keys make sense in layer 5+ (see `references/0009-platform-special-keys.md`). |

## Platform-Aware Layer Suggestions

After confirming the host (from MISSION.md or this turn), adjust layer recommendations:

| Host | Recommended default layers | Notes |
|---|---|---|
| **macOS** | Base + Symbols + Navigation + Numbers + **macMedia** (KC_MCTL, KC_LPAD, KC_ASST, KC_MPLY, KC_VOLU) + Globals (RGB) | Mission Control & Launchpad are HID Consumer natives — include them. |
| **Windows** | Base + Symbols + Navigation + Numbers + **winMedia** (KC_MPLY, KC_VOLU, KC_CPNL, KC_MAIL, KC_CALC, browser-nav `KC_W*`) + Globals (RGB) | Mission Control/Launchpad don't fire here — leave them off. Add `KC_CPNL`, mail, calc. |
| **Linux** | Base + Symbols + Navigation + Numbers + **lxMedia** (KC_MPLY, KC_VOLU — anything else needs xkb remap) + Globals (RGB) | Most HID Consumer codes don't fire out of the box. Keep Media lean or teach the user about xkb/hwdb. |
| **iOS / iPadOS** | Base + Symbols + Navigation + Numbers + Globals (RGB off or dim) | Mostly typing + brightness. Power-draw from RGB matrix drains iPad batteries fast. |
| **Cross-platform** | Base + Symbols + Navigation + Numbers + **univMedia** (KC_MPLY, KC_VOLU, KC_VOLD, KC_MUTE only) + Globals (RGB off) | Stick to media transport that's universal across hosts. Skip host-launcher keys. |

Always restate the chosen host in NOTES.md and MISSION.md after this phase. If you later add a layer 5+ with platform-specific keys, re-check the host hasn't changed.

## Common Layer Sets

| Tier | Layers | Use case |
|---|---|---|
| **Minimal** | Base only | Just QWERTY + mods on home row |
| **Standard** | Base, Symbols, Navigation, Numbers (+ host media) | Most users |
| **Extended** | Base, Symbols, Navigation, Numbers, (+ host Media/Mouse), Function | Power users |
| **Power** | Base, Code, Symbols, Navigation, Numbers, (+ host Media), Function, Mouse, Gaming | Custom keyboards, programmers |

## Recommended Layer Names

LazyQMK doesn't enforce names but conventions help with exports and clarity:

```text
Layer 0: Base              (always; main typing)
Layer 1: Symbols           (brackets, math, programming)
Layer 2: Navigation        (arrows, page up/down)
Layer 3: Numbers           (numpad + math operators)
Layer 4: Code              (optimized for coding — extra brackets)
Layer 5: Media             (play/pause, vol, brightness)
Layer 6: Mouse             (mouse keys + scroll)
Layer 7: Function          (F-keys for app shortcuts)
```

QMK actually defaults to `LAYER_STATE_16BIT` (16 layers) when no flag is set (`qmk_firmware/quantum/action_layer.h:45-46`). Boards like Corne support 16 layers out of the box. For 17+ layers, you must enable `LAYER_STATE_32BIT` in the keyboard's `config.h` (NOT `rules.mk` — these are C defines). LazyQMK's model allows indices up to 32; firmware runtime is whatever the keyboard configures.

## Phase 4: Steps

### 1. Decide layers

Ask user:

> "How many layers do you want? Common choices:
> - **4 layers** (Base, Symbols, Navigation, Numbers) — most users
> - **5 layers** (above + Code) — programmers
> - **6+ layers** (above + Media/Mouse/Function) — power users
>
> What would you like?"

### 2. Decide names

For each layer, suggest a name. User can rename later via TUI/Web UI but JSON edit works too.

### 3. Add empty layers to layout

For each new layer (Base already exists), add a skeleton:

```bash
LAYOUT=~/.../my.json
KEY_COUNT=$(jq '.layers[0].keys | length' "$LAYOUT")
NEXT_LAYER_NUM=$(jq '.layers | length' "$LAYOUT")
LAYER_NAME="Symbols"  # adjust per layer
LAYER_COLOR='{r: 107, g: 114, b: 128}'  # adjust

# Add layer
jq --argjson num "$NEXT_LAYER_NUM" --arg name "$LAYER_NAME" \
   --argjson kc "$KEY_COUNT" --argjson color "$LAYER_COLOR" '
  .layers += [{
    id: ("00000000-0000-0000-0000-" + ($num | tostring | ("000000000000" + .)[-12:])),
    number: $num,
    name: $name,
    default_color: $color,
    category_id: null,
    keys: [.layers[0].keys | .[] | {
      position: .position,
      keycode: "KC_TRNS",
      label: null,
      color_override: null,
      category_id: null,
      combo_participant: false
    }],
    layer_colors_enabled: true
  }]
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

Loop this for each new layer. Or, in a single jq:

```bash
LAYOUT=~/.../my.json

jq '
  .layers += [
    {
      id: "11111111-1111-1111-1111-111111111111",
      number: (.layers | length),
      name: "Symbols",
      default_color: {r: 107, g: 114, b: 128},
      category_id: null,
      keys: [.layers[0].keys | .[] | {
        position: .position,
        keycode: "KC_TRNS",
        label: null,
        color_override: null,
        category_id: null,
        combo_participant: false
      }],
      layer_colors_enabled: true
    },
    {
      id: "22222222-2222-2222-2222-222222222222",
      number: (.layers | length + 1),
      name: "Navigation",
      default_color: {r: 128, g: 128, b: 128},
      category_id: null,
      keys: [.layers[0].keys | .[] | {
        position: .position,
        keycode: "KC_TRNS",
        label: null,
        color_override: null,
        category_id: null,
        combo_participant: false
      }],
      layer_colors_enabled: true
    }
  ]
' "$LAYOUT" > "$LAYOUT.new"
mv "$LAYOUT.new" "$LAYOUT"
```

### 4. Validate

```bash
lazyqmk validate --layout ~/.../my.json
# Expect: "✓ Validation passed"

# Check layer count
lazyqmk inspect --layout ~/.../my.json --section layers --json | jq '.count'
```

## Validation

```bash
# All layers have correct key count
jq '.layers | map({name, key_count: (.keys | length)})' ~/.../my.json

# All layers sequential numbering
jq '.layers | map(.number)' ~/.../my.json
# Should be [0, 1, 2, 3, ...]
```

## Workspace Update

Add to NOTES.md:

```markdown
## Layers
- Layer 0: Base
- Layer 1: Symbols
- Layer 2: Navigation
- Layer 3: Numbers
- (etc.)
```

## Layer Switching Access

Once layers exist, plan how to access them from Base:

| Pattern | Description | Use for |
|---|---|---|
| LT(1, KC_SPC) | Tap Space, hold Symbols | Thumb spacebar (most common) |
| MO(1) | Hold for layer | Less-used layer access |
| TG(2) | Toggle on/off | Layers you want to leave on |
| TT(3) | Tap-Toggle (5 taps) | Rarely used layer |

Common pattern:

- LT(@symbols-uuid, KC_SPC) on left thumb
- LT(@nav-uuid, KC_TAB) on a top-row key (e.g., Esc position)
- LT(@numbers-uuid, KC_BSPC) on right thumb

The actual `LT()` setup happens in Phase 6 (populate base layer).

## Common Pitfalls

- **Too many layers** — more than 8-10 makes navigation hard. Stick to 4-6 unless you have specific needs.
- **Empty layer with no keys** — every layer must have `key_count == first_layer.keys.length`. Validate catches this.
- **Duplicate layer number** — must be sequential 0, 1, 2, ...
- **Layer names > 50 chars** — rejected by validate
- **Layer name empty** — rejected by validate
- **Forgetting to update Base layer** with `LT()` references to the new layers — Phase 6 handles this.

## Next Lesson

`lessons/0005-semantic-categories.md` (Phase 5) — define categories + assign colors.