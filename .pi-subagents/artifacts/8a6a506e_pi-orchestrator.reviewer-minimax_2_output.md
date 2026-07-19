# Review: src/web/mod.rs (swap_keys simplification)

**Reviewed:** `git diff src/web/mod.rs` (+4 / -33, single function)
**Scope:** Read-only. Spot-checked adjacent handlers and tests, plus the
`KeyDefinition`/`Layout::validate`/parser glue to validate the new comment.

## Summary

The diff is a clean win: 8 debug `eprintln!`s and a 12-line field-by-field swap
collapse to a single `Vec::swap`, with a comment explaining the invariant.
**No blockers, no other debug-leftovers in this file**, and no other handler
shows the same anti-pattern. There are two small concerns worth flagging —
the comment slightly overstates the structural guarantee, and the test suite
does not cover the newly-preserved fields or the existing error branches.

## Critical Issues 🔴

_None._

## Warnings ⚠️

- **src/web/mod.rs:1741–1743** — The new comment claims position is
  "structurally guaranteed equal" within a layer. The intended meaning is
  *"unique"*, but the literal phrasing reads as a tautology. More importantly,
  the enforcement is `Layout::validate()` (`models/layout.rs:2331`), which is
  only called from `save_layout` (line 1666) and `migrate_md_to_json`. **`LayoutService::load` does not call `validate()`**, so a hand-crafted or
  partially-corrupted `.json` with duplicate positions could reach `swap_keys`.
  In that case both `position()` lookups return `Some(<first match>)`, the
  indices are equal, and `Vec::swap(a, a)` panics — taking the request worker
  down. This is a **pre-existing** weakness (the old field-by-field code had
  the same blind-spot and would silently swap against itself), but the new
  one-liner is more directly exposed.
  
  Suggestions, any one of which is fine:
  - Reword the comment to "Positions are unique within a layer (enforced by `Layout::validate`, which save_path runs)…"
  - Add `layout.validate()` after `load` in `swap_keys` (cheap, ~microseconds), which simultaneously tightens the invariant and surfaces user-side corruption as a 400.
  - `debug_assert_ne!(idx1, idx2, "duplicate positions in layer")` before `swap`.

- **src/web/mod.rs:1745** — *Behavioral change worth being explicit about.*
  The old code only moved 3 fields (`keycode`, `color_override`,
  `category_id`); **it silently dropped `label`, `combo_participant`, and
  `description` onto the floor** when swapping. The new whole-struct swap
  moves all 6 fields, which is almost certainly the intended/correct
  semantics — but it is observable behavior change. Confirm this matches
  product intent and consider calling it out in the commit message.

## Suggestions 💡

- **tests/web_api_tests.rs:1809** — The single existing test
  (`test_swap_keys_swaps_keycodes_and_colors`) only asserts `keycode` and
  `color_override`. With the new semantics it's worth adding positive
  coverage for `label`, `category_id`, `combo_participant`, and
  `description` round-tripping through a swap, plus the three error
  branches that aren't exercised at all:
  - `Cannot swap a key with itself` (first == second position)
  - `Invalid layer number`
  - `One or both key positions not found`
  
  These are cheap to add and lock in both the correctness fix *and* the
  invariant the new comment leans on.

- **src/web/mod.rs:1740** — Minor: the `_ =>` arm collapses
  `(None, Some)`, `(Some, None)`, and `(None, None)` into one
  `"One or both key positions not found"` error. Functionally fine; not
  blocking, but if you want sharper diagnostics, distinguishing
  `(Some, None)` vs `(None, _)` costs two `match` arms. Optional.

## Positive Notes ✅

- **Excellent grep results.** Searched the whole file for
  `eprintln! / dbg! / println! / TODO / FIXME / XXX / unimplemented! /
  todo!` — **zero matches**. The debug-log cleanup was thorough and the
  file as a whole is clean.
- **Adjacent handlers spot-checked (`save_layout` line 1634,
  `generate_firmware` line 2535, `start_build` line 2624, etc.)** — none
  show the same debug-leftover pattern. This was a one-spot problem, not a
  recurring one.
- **No `allow(dead_code)` / `expect` / `unwrap` creep** in the new code;
  error path still returns the same `Result` shape with descriptive
  `ApiError` payloads.
- **Net -29 lines.** Clear simplification, no information loss, comment
  carries the load that the deleted code used to.

## Out of Scope (Noted, Not Blockers)

- `validate_filename` (line 1122) and `with_json_ext` (line 1151) —
  unchanged by this diff. Worth flagging for a future pass: neither
  rejects NUL bytes (relevant if `Path` APIs are ever given
  attacker-controlled names; CWE-158) and `validate_filename` does not
  block Windows reserved names (`CON`, `PRN`, `NUL`, `COM1`-`COM9`,
  `LPT1`-`LPT9`). Not introduced here — flagging only because the task
  asked.