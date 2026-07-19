# Task for pi-orchestrator.reviewer-minimax

Read-only analysis: For each of the 16 `#[allow(dead_code)]` annotations in /Users/svenlochner/dev/LazyQMK/src/models/layout.rs, determine if the annotated method/field is actually used elsewhere in the codebase.

The 16 sites (line numbers):
- 194: RgbMatrixEffect::all() const fn -> &'static [Self]
- 614: PaletteFxEffect::all() const fn -> &'static [Self]
- 628: PaletteFxEffect::display_name() const fn -> &'static str
- 741: PaletteFxPalette::all() const fn -> &'static [Self]
- 765: PaletteFxPalette::display_name() const fn -> &'static str
- 878: PaletteFxSettings::has_custom_settings()
- 912: RgbOverlayRippleSettings::validate(&self) -> Result<()>
- 934: RgbOverlayRippleSettings::has_custom_settings()
- 1042: ComboDefinition::validate(&self) -> Result<()>
- 1082: ComboSettings::...some pub const fn
- 1092: ComboSettings::...some pub method
- 1121: TapDanceAction::...some pub method
- 1132: TapDanceAction::...some pub method
- 1171: <on TapHoldPreset or HoldDecisionMode>
- 1599: <some const fn in HoldDecisionMode or TapHoldPreset>
- 1779: <some const fn in LayoutMetadata>

THE TASK:

For each line N with #[allow(dead_code)]:
1. Read 5-10 lines around it to get the actual function/field name + signature.
2. Run `rg '<FunctionName>' src/ tests/ --type rust` from /Users/svenlochner/dev/LazyQMK.
3. Determine: USED (callers exist) or DEAD (only the definition exists; tests don't use it).
4. For DEAD ones: also note any related helpers nearby that might also be dead (cascade).

Output a table:
```
| Line | Function/Field | Decision | Reason |
| 194  | RgbMatrixEffect::all | KEEP | used in web/mod.rs:2986, settings_manager.rs:1011, ... |
| 628  | PaletteFxEffect::display_name | DELETE | no callers |
| ...
```

Be exhaustive. Check every line. The output will be consumed by another agent that will edit the file to delete the dead items + remove the dead_code allows on the kept ones.

If a function has callers in test code only (#[cfg(test)] mod tests or tests/ directory), it's still USED — those tests are part of the public API surface.

If you find blocks of related dead code (e.g. a `display_name` AND a private table nearby both unused), flag them in a "cascade deletes" section.

Output as a markdown report. Don't modify any files.

---
**Output:**
Write your findings to exactly this path: /tmp/dead-code-audit.md
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