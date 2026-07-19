# Task for pi-orchestrator.reviewer-minimax

Review the git diff below for correctness. Read-only — do not modify files.

LOCATION: /Users/svenlochner/dev/LazyQMK/src/web/mod.rs, function `async fn swap_keys`.

DIFF SUMMARY:
- Removed 8 eprintln! debug logs (lazyqmk-aopx.3)
- Replaced field-by-field swap (keycode/color_override/category_id only) with `layer.keys.swap(idx1, idx2)` (bug 2 fix)
- Added comment explaining the invariant

DIFF SNIPPET:
```
-    // Debug logging
-    eprintln!(
-        "Swap request: layer={}, first=({},{}), second=({},{})",
-        ...);
-    eprintln!("Layer has {} keys", layer.keys.len());
-    for (i, k) in layer.keys.iter().enumerate() {
-        eprintln!("  Key {}: pos=({},{}) keycode={}", ...);
-    }
-
     // Find indices of both keys by position
     let first_idx = layer.keys.iter().position(...);
     ...
-    eprintln!("Found first_idx={:?}, second_idx={:?}", ...);
-
     match (first_idx, second_idx) {
         (Some(idx1), Some(idx2)) => {
-            // Swap all properties: keycode, color_override, category_id
-            let first_key = layer.keys[idx1].clone();
-            let second_key = layer.keys[idx2].clone();
-
-            layer.keys[idx1].keycode = second_key.keycode;
-            layer.keys[idx1].color_override = second_key.color_override;
-            layer.keys[idx1].category_id = second_key.category_id;
-
-            layer.keys[idx2].keycode = first_key.keycode;
-            layer.keys[idx2].color_override = first_key.color_override;
-            layer.keys[idx2].category_id = first_key.category_id;
+            // Position is structurally guaranteed equal for keys on the same layer,
+            // so swapping whole KeyDefinition structs preserves all fields (keycode,
+            // label, color_override, category_id, combo_participant, description).
+            layer.keys.swap(idx1, idx2);
```

CRITICAL CHECKS:
1. `KeyDefinition` lives in /Users/svenlochner/dev/LazyQMK/src/models/layer.rs. Read lines 90-200 and confirm whether two keys on the same layer at different positions can have DIFFERENT positions. (They should NOT — same-layer positions are guaranteed equal because the layer enforces a single key per (row, col).)
2. Does `Vec::swap` actually move all fields including label/combo_participant/description? (It swaps the elements; all fields transfer.)
3. Could the layer validation reject the swap or corrupt state if a key had unusual state (e.g. invalid color_override, None combos)?
4. The existing test `test_swap_keys_swaps_keycodes_and_colors` must still pass — confirm by reading /Users/svenlochner/dev/LazyQMK/tests/web_api_tests.rs:1809-1920.

Run `git diff src/web/mod.rs` from /Users/svenlochner/dev/LazyQMK if you want exact context. Read-only.

Return blockers / suggested / nitpicks. Be concise.

---
**Output:**
Write your findings to exactly this path: /tmp/review2-correctness.md
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