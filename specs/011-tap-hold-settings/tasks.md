# Tasks: 011-tap-hold-settings

## Phase 1: Data Model
- [x] Add `HoldDecisionMode` enum to `src/models/layout.rs`
- [x] Add `TapHoldSettings` struct with all fields
- [x] Add `TapHoldPreset` enum with preset definitions
- [x] Implement `Default` for `TapHoldSettings`
- [x] Implement preset application methods
- [x] Add `tap_hold_settings` field to `Layout` struct
- [x] Export types from `src/models/mod.rs`
- [x] Add unit tests for defaults and presets

## Phase 2: Parser Support  
- [x] Update frontmatter parsing in `parser/layout.rs`
- [x] Update serialization in `parser/template_gen.rs`
- [x] Add round-trip tests

## Phase 3: Settings UI
- [x] Extend `SettingItem` enum with tap-hold settings
- [x] Add `ManagerMode` variants for selection modes
- [x] Implement grouped settings display
- [x] Add preset selector
- [x] Add numeric input handling for timing values
- [x] Add boolean toggle handling
- [x] Update help text for new settings

## Phase 4: Firmware Generator
- [x] Update `generate_merged_config_h()` to emit tap-hold defines
- [x] Only emit non-default values
- [x] Add integration tests

## Phase 5: Help & Documentation
- [x] Update help overlay with tap-hold settings section
- [x] Add tips for home-row mod configuration

## Status: COMPLETE ✅
All phases implemented and tested. Ready for commit.
