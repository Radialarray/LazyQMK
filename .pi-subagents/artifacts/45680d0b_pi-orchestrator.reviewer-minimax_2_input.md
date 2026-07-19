# Task for pi-orchestrator.reviewer-minimax

Review the git diff below for simplicity, maintainability, and code cleanliness. Read-only.

DIFF (in /Users/svenlochner/dev/LazyQMK/src/tui/mod.rs, run_tui function):
```
@@ -933,12 +933,16 @@ pub fn run_tui(
-            if let Event::Key(key) = event::read()? {
-                if handle_key_event(state, key)? {
-                    break; // User quit
+            match event::read()? {
+                Event::Key(key) => {
+                    if handle_key_event(state, key)? {
+                        break;
+                    }
                   }
-            } else if let Event::Resize(_, _) = event::read()? {
-                // Terminal resized, will re-render on next loop
+                Event::Resize(_, _) => {
+                    // Terminal resized, will re-render on next loop
+                }
+                _ => {}
               }
           }
```

CHECK:
1. Is the match arm cleaner than the if-let-else-if it replaced?
2. Is the `_ => {}` catch-all reasonable, or should each unused event variant be commented for clarity?
3. Are there any docs/comments that should accompany the change?
4. Could this be even simpler with helpers (e.g. extracting event dispatch)?

Return findings (blockers / suggested / nitpicks). Be concise.

---
**Output:**
Write your findings to exactly this path: /tmp/review-simplicity.md
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