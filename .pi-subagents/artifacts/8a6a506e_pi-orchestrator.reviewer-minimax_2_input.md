# Task for pi-orchestrator.reviewer-minimax

Review the git diff in /Users/svenlochner/dev/LazyQMK/src/web/mod.rs for simplicity and code cleanliness. Read-only.

DIFF SUMMARY: 
1. Removed 8 eprintln! debug logs from swap_keys handler (~25 lines deleted)
2. Replaced field-by-field KeyDefinition swap with `layer.keys.swap(idx1, idx2)` (~12 lines → 1 line)
3. Added a comment explaining the invariant

Net: -29 lines, +5 lines. The handler is much cleaner.

CHECKS:
1. Is the comment "Position is structurally guaranteed equal for keys on the same layer" accurate? (Yes — see parse logic and layer validation)
2. Are there other handlers in this file (e.g. save_layout, generate_firmware, swap_keys' neighbors) that have similar debug-leftover patterns?
3. Does the handler still handle errors properly? (the swap() can panic if idx1 == idx2, but the existing check at line 1718 prevents same-key swap)
4. Should the verify_filename / with_json_ext helpers be reviewed? (out of scope but flag if obvious)

Read-only. Return blockers / suggested / nitpicks. Concise.

---
**Output:**
Write your findings to exactly this path: /tmp/review2-simplicity.md
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