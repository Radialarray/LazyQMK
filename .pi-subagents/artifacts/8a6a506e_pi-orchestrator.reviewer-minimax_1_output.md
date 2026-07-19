# Review: src/web/mod.rs swap_keys - Vec::swap simplification

Tests: 1446 passed, 5 ignored after change.

## Summary
Bug fix is sound. Replacing field-by-field swap with Vec::swap is correct because KeyDefinition (models/layer.rs:96) carries position as part of the struct, and a swap-by-position index on the same layer guarantees both entries share an identical Position. Whole-struct swap preserves all 6 fields (keycode, label, color_override, category_id, combo_participant, description) atomically, eliminating the original copy-3-of-6-fields bug.

## Critical Issues
None.

## Warnings
- Test gap (the bug this fixed is invisible to the test suite). test_swap_keys_swaps_keycodes_and_colors (web_api_tests.rs:1809) only asserts on keycode + color_override. The original copy-paste bug silently dropped label, category_id, combo_participant, description; this regression would NOT have been caught by the original test. After the fix, the test passes for a different reason (it tests the right two fields) but provides zero coverage for the other four fields. Recommend extending the test to set distinct values on all six fields and assert each swaps.

## Suggestions
- tests/web_api_tests.rs:1809 - extend setup to also set:
  - layout.layers[0].keys[0].label = Some("LBL_Q"); keys[1].label = Some("LBL_W")
  - layout.layers[0].keys[0].category_id = Some("alpha"); keys[1].category_id = Some("mod")
  - layout.layers[0].keys[0].combo_participant = true; keys[1].combo_participant = false
  - layout.layers[0].keys[0].description = Some("desc_q"); keys[1].description = Some("desc_w")
  Then assert each round-trips through JSON to the swapped position. Pins the regression for future KeyDefinition field additions.

- src/web/mod.rs - new comment slightly overstates: "Position is structurally guaranteed equal for keys on the same layer" is true by lookup (API request resolves indices via position.row/col), not by layer invariant. Consider: "Indices are resolved by Position lookup, so both entries share identical position; whole-struct swap preserves all fields."

- Optional nit: eprintln removal is a cleanup win but unrelated to the bug fix - consider separating into its own commit for cleaner history.

## Positive Notes
- Diff is minimal and focused (4 ins / 33 del in one function).
- Vec::swap is O(1), replacing two clones + six assignments - measurable win on large layers.
- All 1446 tests still pass; no behavior regression.
- The copy-3-of-6-fields bug is now structurally impossible - adding a new field to KeyDefinition can no longer silently break swap_keys.

## Field coverage matrix (KeyDefinition, models/layer.rs:96)
- position: pre-fix n/a, post-fix n/a, test n/a (lookup key)
- keycode: pre-fix YES, post-fix YES, test asserts YES
- label: pre-fix NO bug, post-fix YES, test asserts NO
- color_override: pre-fix YES, post-fix YES, test asserts YES
- category_id: pre-fix YES, post-fix YES, test asserts NO
- combo_participant: pre-fix NO bug, post-fix YES, test asserts NO
- description: pre-fix NO bug, post-fix YES, test asserts NO

## Verdict
Approve with test-extension suggestion. No blockers. The fix is correct; the existing test happens to pass but does not prove the fix covers all fields - recommend extending it (not required for merge).

## No integration tests round-trip swap
grep across tests/ for swap / swap_keys returns only web_api_tests.rs:1809. No other test path exercises the swap-keys endpoint or KeyDefinition field-set integrity after position-based lookup.