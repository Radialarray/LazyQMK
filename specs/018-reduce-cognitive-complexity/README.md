# Spec 018: Reduce Cognitive Complexity

## Status: PLANNED

## Overview

Refactor functions with high cognitive complexity (>25) by extracting handler functions into dedicated modules. This eliminates the need for `#[allow(clippy::cognitive_complexity)]` suppressions and improves code maintainability.

## Problem

We have 2 functions exceeding Clippy's cognitive complexity threshold:

1. **`dispatch_action()`** - Score: 73/25 (647 lines, 38 action handlers)
   - Location: `src/tui/handlers/actions.rs:143-789`
   - Problem: Giant match statement with nested logic
   
2. **`test_tap_hold_settings_round_trip()`** - Score: 31/25
   - Location: `src/parser/template_gen.rs:623-691`
   - Problem: Tests 3 scenarios with 40+ assertions in one function

## Solution

### For `dispatch_action()`:
Extract 38 action handlers into categorized handler modules:
- `action_handlers/navigation.rs` (8 actions)
- `action_handlers/popups.rs` (9 actions)
- `action_handlers/file_ops.rs` (3 actions)
- `action_handlers/key_ops.rs` (6 actions)
- `action_handlers/selection.rs` (2 actions)
- `action_handlers/color.rs` (4 actions)
- `action_handlers/category.rs` (2 actions)
- `action_handlers/firmware.rs` (2 actions)
- `action_handlers/layout.rs` (1 action)

Result: `dispatch_action()` becomes a thin dispatcher (score: ~10/25)

### For `test_tap_hold_settings_round_trip()`:
Split into 3 separate test functions with helper assertion functions:
- `test_tap_hold_home_row_mods_preset_round_trip()`
- `test_tap_hold_custom_settings_round_trip()`
- `test_tap_hold_default_settings_not_written()`

Result: Each test function scores ~8/25

## Benefits

- **Maintainability**: Easy to find, modify, and add action handlers
- **Readability**: Small, focused functions instead of 647-line monsters
- **Testability**: Can unit test handlers independently
- **Code Quality**: Eliminates cognitive complexity warnings without suppressions

## Files Affected

- `src/tui/handlers/actions.rs` (refactor)
- `src/tui/handlers/action_handlers/*.rs` (new)
- `src/parser/template_gen.rs` (split test)

## Estimated Time

10-13 hours total (can be parallelized)

## See Also

- [Detailed Plan](./plan.md)
