# Spec 011: Tap-Hold Settings

## Overview

Add comprehensive tap-hold configuration to the TUI, enabling users to configure timing and behavior for dual-function keys (LT, MT, LM, SH_T, TT). This includes global settings, home-row mod optimizations, and presets for common use cases.

## Background

QMK's tap-hold system allows keys to perform different actions when tapped vs held:
- **LT(layer, kc)**: Hold activates layer, tap sends keycode
- **MT(mod, kc)**: Hold activates modifier, tap sends keycode
- **TT(layer)**: Hold activates layer, multiple taps toggle it

The timing and decision logic for tap vs hold is configurable through various `#define` options in `config.h`.

## Goals

1. Allow users to configure tap-hold behavior globally through the Settings UI
2. Support home-row mod optimizations (Flow Tap, Chordal Hold)
3. Provide presets for common configurations while exposing underlying values
4. Generate appropriate `config.h` defines during firmware build

## Non-Goals

1. Per-key tap-hold configuration (future enhancement)
2. Visual per-key timing indicators in the keyboard view
3. Runtime tapping term adjustment keycodes (DT_UP, DT_DOWN)

## Data Model

### TapHoldSettings Struct

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TapHoldSettings {
    // === Core Timing ===
    /// Time in ms to distinguish tap from hold (default: 200)
    pub tapping_term: u16,
    
    /// Time window for tap-then-hold to auto-repeat instead of hold action
    /// None = same as tapping_term
    pub quick_tap_term: Option<u16>,
    
    // === Decision Mode ===
    /// How to decide between tap and hold when other keys are involved
    pub hold_mode: HoldDecisionMode,
    
    // === Special Behaviors ===
    /// Send tap keycode even if held past tapping term (when no other key pressed)
    pub retro_tapping: bool,
    
    /// Number of taps to toggle layer with TT() keys (default: 5)
    pub tapping_toggle: u8,
    
    // === Home Row Mod Optimizations ===
    /// Flow Tap: rapid typing triggers tap, not hold (for home row mods)
    /// None = disabled, Some(ms) = flow tap term
    pub flow_tap_term: Option<u16>,
    
    /// Chordal Hold: opposite-hand rule for tap-hold decision
    pub chordal_hold: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum HoldDecisionMode {
    /// Only timing-based: hold if key held > tapping_term
    #[default]
    Default,
    
    /// Hold when another key is tapped during hold (nested: A down, B down, B up, A up)
    PermissiveHold,
    
    /// Hold when another key is pressed during hold (rolling: A down, B down, A up, B up)
    HoldOnOtherKeyPress,
}
```

### Default Values

| Setting | Default | Notes |
|---------|---------|-------|
| tapping_term | 200 | QMK default |
| quick_tap_term | None (=tapping_term) | - |
| hold_mode | Default | - |
| retro_tapping | false | - |
| tapping_toggle | 5 | QMK default |
| flow_tap_term | None (disabled) | - |
| chordal_hold | false | - |

### Presets

```rust
pub enum TapHoldPreset {
    /// QMK defaults - conservative timing
    Default,
    
    /// Optimized for fast typists using home row mods
    /// Lower tapping term, permissive hold, flow tap enabled
    HomeRowMods,
    
    /// Very responsive - quick hold detection
    /// Low tapping term, hold on other key press
    Responsive,
    
    /// Deliberate - requires intentional holds
    /// Higher tapping term, default mode
    Deliberate,
    
    /// Custom - user-defined values
    Custom,
}
```

**Preset Values:**

| Preset | tapping_term | quick_tap | hold_mode | retro | flow_tap | chordal |
|--------|--------------|-----------|-----------|-------|----------|---------|
| Default | 200 | None | Default | false | None | false |
| HomeRowMods | 175 | 120 | PermissiveHold | true | 150 | true |
| Responsive | 150 | 100 | HoldOnOtherKeyPress | false | None | false |
| Deliberate | 250 | None | Default | false | None | false |
| Custom | (user) | (user) | (user) | (user) | (user) | (user) |

## Settings UI

### Layout

The settings manager will be extended with a "Tap-Hold" section:

```
┌─ Settings (Shift+S) ─────────────────────────────┐
│                                                   │
│  ═══ Display Settings ═══                         │
│  Inactive Key Behavior: Show Color                │
│                                                   │
│  ═══ Tap-Hold Settings ═══                        │
│  Preset: Home Row Mods                            │
│  ─────────────────────────                        │
│  Tapping Term: 175 ms                             │
│  Quick Tap Term: 120 ms                           │
│  Hold Mode: Permissive Hold                       │
│  Retro Tapping: On                                │
│  Tapping Toggle: 5 taps                           │
│  ─────────────────────────                        │
│  Flow Tap Term: 150 ms                            │
│  Chordal Hold: On                                 │
│                                                   │
│  ↑/↓: Navigate  Enter: Change  Esc: Close         │
└───────────────────────────────────────────────────┘
```

### Interaction

1. **Preset Selection**: Selecting a preset auto-fills all values
2. **Individual Values**: Changing any value switches preset to "Custom"
3. **Numeric Input**: Timing values use left/right arrows (±5ms) or direct entry
4. **Boolean Toggle**: Space or Enter to toggle
5. **Enum Selection**: Opens sub-menu with options

## Firmware Generation

### config.h Output

```c
// Tap-Hold Configuration (generated by keyboard_tui)
#define TAPPING_TERM 175
#define QUICK_TAP_TERM 120
#define PERMISSIVE_HOLD
#define RETRO_TAPPING
#define TAPPING_TOGGLE 5

// Home Row Mod Optimizations
#define FLOW_TAP_TERM 150
#define CHORDAL_HOLD
```

### Conditional Generation

- Only emit defines that differ from QMK defaults
- `QUICK_TAP_TERM` only if different from `TAPPING_TERM`
- `PERMISSIVE_HOLD` or `HOLD_ON_OTHER_KEY_PRESS` only if not Default mode
- `RETRO_TAPPING` only if enabled
- `FLOW_TAP_TERM` only if Some(value)
- `CHORDAL_HOLD` only if enabled

## Implementation Phases

### Phase 1: Data Model
- Add `TapHoldSettings` and `HoldDecisionMode` to `src/models/layout.rs`
- Add `TapHoldPreset` enum with preset definitions
- Add `tap_hold_settings` field to `Layout` struct
- Implement `Default` for `TapHoldSettings`

### Phase 2: Parser Support
- Update `parser/layout.rs` to parse tap-hold settings from frontmatter
- Update `parser/template_gen.rs` to serialize settings
- Add tests for round-trip serialization

### Phase 3: Settings UI
- Extend `SettingItem` enum with tap-hold settings
- Add `ManagerMode` variants for each setting type
- Implement numeric input widget for timing values
- Add preset selector
- Group settings visually

### Phase 4: Firmware Generator
- Update `generate_merged_config_h()` to emit tap-hold defines
- Only emit non-default values
- Add tests for generated output

### Phase 5: Help & Documentation
- Update help overlay with tap-hold settings documentation
- Add tips for home-row mod configuration

## File Changes

| File | Changes |
|------|---------|
| `src/models/layout.rs` | Add TapHoldSettings, HoldDecisionMode, TapHoldPreset |
| `src/models/mod.rs` | Export new types |
| `src/parser/layout.rs` | Parse tap_hold_settings |
| `src/parser/template_gen.rs` | Serialize tap_hold_settings |
| `src/tui/settings_manager.rs` | Extend UI for tap-hold settings |
| `src/tui/mod.rs` | Handle new settings input |
| `src/firmware/generator.rs` | Generate config.h defines |
| `src/tui/help_overlay.rs` | Document tap-hold settings |

## Testing

1. **Unit Tests**
   - TapHoldSettings default values
   - Preset application
   - Serialization round-trip

2. **Integration Tests**
   - Firmware generation with various settings
   - Parser handles missing/partial settings (backwards compatibility)

3. **Manual Testing**
   - Settings UI navigation
   - Preset switching updates all values
   - Custom values persist correctly

## Future Enhancements

1. Per-key tap-hold configuration via key properties
2. Visual indicator showing which keys have tap-hold behavior
3. Runtime tapping term adjustment keycodes
4. Import/export of tap-hold presets
