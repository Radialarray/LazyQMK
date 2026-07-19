# Correctness Review — `src/web/mod.rs::swap_keys`

## Summary

The diff is correct. Replacing the field-by-field copy with `Vec::swap` is sound: it exchanges whole `KeyDefinition` structs (all 7 fields: `position`, `keycode`, `label`, `color_override`, `category_id`, `combo_participant`, `description`) atomically. The earlier implementation silently dropped three fields — that bug is now fixed. `cargo test test_swap_keys_swaps_keycodes_and_colors` passes; `cargo clippy --all-features -- -D warnings` is clean (0 errors).

## Critical Checks

### Check 1 — Layer uniqueness of `position`
Read `src/models/layer.rs:60-200`. `Position { row: u8, col: u8 }` is a unique-per-`Layer` key by validation (doc comment on `Layer`, line ~162: *"Position must be unique within parent Layer"*). Two distinct keys in a layer always have **different** `(row, col)`. So `Vec::swap` exchanges two distinct elements.

### Check 2 — `Vec::swap` moves every field
Standard library semantics: `Vec::swap(a, b)` exchanges the two slots by `mem::swap` of the inner type. All fields of `KeyDefinition` — including `label: Option<String>`, `combo_participant: bool`, `description: Option<String>` — move together. The previously-dropped fields are now preserved.

### Check 3 — Save path can't reject the swap
`LayoutService::save` (`src/services/layouts.rs:121`) is just `parser::save_json_layout(layout, &json_path)` — no per-key validation. `Option<RgbColor>`, `bool`, and `Option<String>` cannot be made "invalid" by an in-memory swap. Position-uniqueness is preserved (we just permute two valid elements). No save-time rejection risk.

### Check 4 — Existing test still passes
`tests/web_api_tests.rs:1809-1920` (`test_swap_keys_swaps_keycodes_and_colors`) verifies by **array index** (`swapped_keys[0]` / `swapped_keys[1]`), not by position. After `swap(0, 1)`:
- `keys[0]` now holds the old `keys[1]` (position (0,1), keycode `KC_W`, color (0,255,0)) → matches assertions
- `keys[1]` now holds the old `keys[0]` (position (0,0), keycode `KC_Q`, color (255,0,0)) → matches assertions

Confirmed by `cargo test test_swap_keys_swaps_keycodes_and_colors --lib --tests`: **1 passed**.

## Findings

### Blockers
None.

### Warnings
- **`src/tui/handlers/action_handlers/selection.rs:48` (`handle_swap_keys`)** still contains the **identical LazyQMK-aopx.2 bug pattern** (lines 86-95: field-by-field copy of only `keycode`, `color_override`, `category_id`). The web handler was fixed; the TUI handler was not. Users invoking Shift+W in the TUI still silently lose `label`, `description`, and `combo_participant` on swap. Recommend a follow-up bd issue + one-line fix mirroring this diff.
- **Comment is technically wrong.** The new comment says *"Position is structurally guaranteed equal for keys on the same layer"*. That is the inverse of the actual invariant — the layer guarantees positions are **unique / different**, not equal. The fix works *because* each key has its own position baked into the struct; `Vec::swap` carries the position along with the rest. Suggested rewording: *"Each layer enforces unique `position` per key (see `Layer` invariant). `Vec::swap` exchanges whole `KeyDefinition` structs, so `position`, `label`, `description`, and `combo_participant` are preserved alongside `keycode`, `color_override`, and `category_id`."*

### Suggestions
- **Add a regression test** for the previously-lost fields. The diff changes observable behavior: post-fix, swapping two keys also exchanges `label` / `description` / `combo_participant`. There is currently no test pinning that contract. A 5-line test in `tests/web_api_tests.rs` setting `label = Some("alpha")` on key 0 and `label = Some("beta")` on key 1, swapping, and asserting they move, would lock in the fix.
- **Helper extraction**: a shared `Layer::swap_keys(pos_a, pos_b) -> Result<()>` on the model would prevent the field-by-field bug from recurring a third time (it already exists in two places).

### Nitpicks
- The `Vec::swap` panic contract (`idx` must be < len) is satisfied because both indices come from `position()` over the same slice — safe.

## Positive Notes
- Idiomatic use of `Vec::swap` — zero allocation, single `mem::swap`, exactly the right primitive.
- Debug-log cleanup (`lazyqmk-aopx.3`) is appropriate; those `eprintln!`s had no place in a production handler.
- Existing test still green; clippy clean.
- Field-by-field copy was the actual bug source (silent data loss); this diff replaces it with a non-lossy primitive — strictly a bug fix on top of the cleanup.

## Commands Run

| Command | Result |
|---|---|
| `cargo test test_swap_keys_swaps_keycodes_and_colors --lib --tests` | passed (1/1) |
| `cargo clippy --all-features -- -D warnings` | clean (0 errors, 0 code warnings) |
| `git diff src/web/mod.rs` | +4 / -33 |