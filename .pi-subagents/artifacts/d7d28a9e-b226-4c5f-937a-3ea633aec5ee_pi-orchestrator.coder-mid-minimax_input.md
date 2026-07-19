# Task for pi-orchestrator.coder-mid-minimax

Fix TWO bugs in the swap_keys HTTP handler in /Users/svenlochner/dev/LazyQMK/src/web/mod.rs (around lines 1675-1790) and verify with cargo test + cargo clippy.

These are tracked as bd issues LazyQMK-aopx.2 and LazyQMK-aopx.3.

==== BUG 1 (LazyQMK-aopx.2): swap_keys loses 3 fields ====

LOCATION: inside `async fn swap_keys(...)` in src/web/mod.rs.

CURRENT BUGGY CODE (around lines 1755-1769):
```rust
match (first_idx, second_idx) {
    (Some(idx1), Some(idx2)) => {
        // Swap all properties: keycode, color_override, category_id
        let first_key = layer.keys[idx1].clone();
        let second_key = layer.keys[idx2].clone();

        layer.keys[idx1].keycode = second_key.keycode;
        layer.keys[idx1].color_override = second_key.color_override;
        layer.keys[idx1].category_id = second_key.category_id;

        layer.keys[idx2].keycode = first_key.keycode;
        layer.keys[idx2].color_override = first_key.color_override;
        layer.keys[idx2].category_id = first_key.category_id;
```

PROBLEM: `KeyDefinition` (see /Users/svenlochner/dev/LazyQMK/src/models/layer.rs:96-115) has 7 fields:
  - position
  - keycode
  - label
  - color_override
  - category_id
  - combo_participant
  - description

The current swap silently drops `label`, `combo_participant`, and `description`. User-added key descriptions stay on the original position when swapped.

FIX: Two options - pick the cleaner one:

OPTION A (preferred — uses std::mem):
```rust
layer.keys.swap(idx1, idx2);
```
Since position is identical for keys on the same layer, swapping whole structs is correct.

OPTION B (defensive, only if `position` could differ):
Copy all 7 fields.

VERIFICATION: Read src/models/layer.rs around lines 90-115 to confirm position is structurally guaranteed equal for keys within one layer, then pick Option A.

==== BUG 2 (LazyQMK-aopx.3): eprintln! debug logging ====

LOCATION: same `async fn swap_keys` body, lines 1722-1742 (8 eprintln! calls).

EXAMPLE:
```rust
eprintln!(
    "Swap request: layer={}, first=({},{}), second=({},{})",
    ...
);
```

FIX: Delete all 8 eprintln! calls entirely. No replacement. (They're production code path; proper logging if needed would use tracing like web/build_jobs.rs does.)

==== STEPS ====

1. Read /Users/svenlochner/dev/LazyQMK/src/web/mod.rs lines 1675-1790 to see the exact current code.
2. Read /Users/svenlochner/dev/LazyQMK/src/models/layer.rs lines 90-115 to confirm KeyDefinition shape and verify positions on same layer are structurally guaranteed equal.
3. Pick Option A or B based on that.
4. Use edit tool to apply BOTH fixes in the same handler.
5. Revert any unrelated rustfmt-style changes in nearby lines.
6. Run `cargo test --lib 2>&1 | tail -10` from /Users/svenlochner/dev/LazyQMK. Must show 560 passed.
7. Run `cargo test --tests 2>&1 | tail -10` from /Users/svenlochner/dev/LazyQMK. Must show existing tests pass.
8. Run `cargo clippy --all-features -- -D warnings 2>&1 | tail -10` from /Users/svenlochner/dev/LazyQMK. Must be clean.

==== OUTPUT FINAL RESPONSE WITH ====

- The exact `git diff` of the swap_keys area (lines only you touched, no diff context for unchanged code)
- cargo test result (must be 1474 passed or unchanged count)
- cargo clippy result (must show no errors/warnings from your changes)

==== CONSTRAINTS ====

- Do NOT edit tests/ files (existing test at tests/cli_layer_refs_tests.rs:549 mentions swap_keys behavior — verify it still passes; if it breaks, find the test and update it minimally; otherwise leave it alone).
- Do NOT add #[allow(...)] attributes.
- Do NOT widen scope to other handlers.
- Do NOT skip the fix for any of the 2 bugs.

If you discover another related issue, STOP and report it as a residual risk instead of fixing.

cwd: /Users/svenlochner/dev/LazyQMK

---
**Output:**
Write your findings to exactly this path: /tmp/chain-step2-impl.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: reviewed
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Implement the requested change without widening scope
- criterion-2: Return evidence sufficient for an independent acceptance review

Required evidence: changed-files, tests-added, commands-run, validation-output, residual-risks, no-staged-files

Review gate: required by reviewer.

Finish with a fenced JSON block tagged `acceptance-report` in this shape:
Use empty arrays when no items apply; array fields contain strings unless object entries are shown.
```acceptance-report
{
  "criteriaSatisfied": [
    {
      "id": "criterion-1",
      "status": "satisfied",
      "evidence": "specific proof"
    }
  ],
  "changedFiles": [
    "src/file.ts"
  ],
  "testsAddedOrUpdated": [
    "test/file.test.ts"
  ],
  "commandsRun": [
    {
      "command": "command",
      "result": "passed",
      "summary": "short result"
    }
  ],
  "validationOutput": [
    "validation output or concise summary"
  ],
  "residualRisks": [
    "none"
  ],
  "noStagedFiles": true,
  "diffSummary": "short description of the diff",
  "reviewFindings": [
    "blocker: file.ts:12 - issue found, or no blockers"
  ],
  "manualNotes": "anything else the parent should know"
}
```