# Task for pi-orchestrator.reviewer-minimax

Review the git diff below for correctness, regressions, and hidden bugs. Read-only — do not modify any files.

The change is a bug fix in /Users/svenlochner/dev/LazyQMK/src/tui/mod.rs inside the run_tui() main loop. It replaces a buggy if-let-else-if pattern (which called event::read() twice on non-Key events) with a single event::read() and an exhaustive match.

DIFF:
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
1. Correctness: Does the new pattern handle Key, Resize, Mouse, Focus, Paste events correctly?
2. Hidden bugs: Are there any crossterm event variants that could cause issues?
3. Regressions: Did the existing behavior change in any way beyond the bug?
4. Style: Is the match arm idiomatic Rust? Is `=> {}` for the catch-all clean?

Return a short list of findings (blockers / suggested / nitpicks). Be honest — if the fix is good, say so concisely.

---
**Output:**
Write your findings to exactly this path: /tmp/review-correctness.md
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