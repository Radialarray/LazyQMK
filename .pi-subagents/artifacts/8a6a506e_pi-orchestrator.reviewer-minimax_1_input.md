# Task for pi-orchestrator.reviewer-minimax

Review the git diff in /Users/svenlochner/dev/LazyQMK/src/web/mod.rs for test coverage. Read-only.

DIFF SUMMARY: replaced field-by-field swap with `layer.keys.swap(idx1, idx2)` + removed 8 eprintln! debug calls.

CHECKS:
1. Existing test `test_swap_keys_swaps_keycodes_and_colors` in tests/web_api_tests.rs:1809 — does it still pass with `Vec::swap` semantics? (It only checks keycode + color_override fields — behavior should be unchanged for these.)
2. Should the test be EXTENDED to verify label, category_id, combo_participant, description all swap too? That would be regression coverage for this exact bug.
3. Are there any integration tests that round-trip through JSON serialization which might miss fields? Search test files for "swap" or "swap_keys".

Read-only. Run `cargo test --tests 2>&1 | tail -10` to verify the test still passes.

Return blockers / suggested / nitpicks. Concise.

---
**Output:**
Write your findings to exactly this path: /tmp/review2-tests.md
This path is authoritative for this run.
Ignore any other output filename or output path mentioned elsewhere, including output destinations in the base agent prompt, system prompt, or task instructions.

## Acceptance Contract
Acceptance level: attested
Completion is not accepted from prose alone. End with a structured acceptance report.

Criteria:
- criterion-1: Return concrete findings with file paths and severity when applicable

Required evidence: review-findings, residual-risks

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