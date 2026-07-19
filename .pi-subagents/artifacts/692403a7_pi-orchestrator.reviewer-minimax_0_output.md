# Dead-Code Audit: `src/models/layout.rs`

Read-only analysis of the 16 `#[allow(dead_code)]` annotations in
`src/models/layout.rs`. Each site is classified as **KEEP** (annotation can be
removed; method has callers) or **DELETE** (annotation can be removed and
method body deleted; no callers). Cascade deletes (dead methods living inside
block-level `#[allow(dead_code)]` impl blocks) are listed separately.

Search corpus: `src/` and `tests/` only (per prompt contract). Test code in
`#[cfg(test)] mod tests` counts as callers.

> Note on prompt labels: a few prompt labels do not match the actual code at
> the stated line numbers. Specifically:
> - L912 is `ComboAction::all()` (not `RgbOverlayRippleSettings::validate`).
> - L934 is `ComboAction::description()` (not `RgbOverlayRippleSettings::has_custom_settings`).
> - L1082 is `ComboSettings::new` (the "const fn" hint matches; signature is `pub const fn new(enabled: bool) -> Self`).
> - L1092 is `ComboSettings::add_combo`.
> - L1121 is `ComboSettings::remove_combo` (prompt said "TapDanceAction").
> - L1132 is `ComboSettings::validate` (prompt said "TapDanceAction").
> - L1171 is the `impl TapDanceAction` block header (matches "TapDanceAction or HoldDecisionMode" description loosely).
> - L1599 is the `impl LayoutMetadata` block header (prompt said "const fn in HoldDecisionMode or TapHoldPreset" — incorrect).
> - L1779 is the `impl Layout` block header (prompt said "LayoutMetadata" — incorrect).
>
> Decisions below use the actual function/impl at each line number.

---

## Direct Findings (16 Sites)

| Line | Function / Field                                  | Decision       | Reason |
|------|---------------------------------------------------|----------------|--------|
| 194  | `RgbMatrixEffect::all() -> &'static [Self]`       | **KEEP**       | USED in 6+ call sites: `src/tui/handlers/settings.rs:731`, `src/tui/settings_manager.rs:1011, 1513, 1518, 2833`, `src/web/mod.rs:2986`. |
| 614  | `PaletteFxEffect::all() -> &'static [Self]`       | **KEEP**       | USED in 5+ call sites: `src/tui/handlers/settings.rs:755`, `src/tui/settings_manager.rs:1029, 1561, 1566, 2867`. |
| 628  | `PaletteFxEffect::display_name(&self) -> &'static str` | **DELETE** | DEAD. No callers anywhere in `src/` or `tests/`. Verified: zero hits. |
| 741  | `PaletteFxPalette::all() -> &'static [Self]`      | **KEEP**       | USED in 9+ call sites: `src/tui/handlers/settings.rs:767, 785`, `src/tui/settings_manager.rs:1038, 1050, 1585, 1590, 1604, 2884, 2901`. |
| 765  | `PaletteFxPalette::display_name(&self) -> &'static str` | **DELETE** | DEAD. No callers anywhere. Verified: zero hits. |
| 878  | `PaletteFxSettings::has_custom_settings(&self) -> bool` | **DELETE** | DEAD. No callers anywhere. Note: `template_gen.rs:297-300, 363, 423` calls `idle_effect_settings.has_custom_settings()`, `combo_settings.has_custom_settings()`, `rgb_overlay_ripple.has_custom_settings()` — none on `PaletteFxSettings`. |
| 912  | `ComboAction::all() -> &'static [Self]`           | **KEEP**       | USED in `src/models/layout.rs:2889` (within `#[cfg(test)] mod tests`). |
| 934  | `ComboAction::description(&self) -> &'static str` | **DELETE**     | DEAD. No callers anywhere. |
| 1042 | `ComboDefinition::validate(&self) -> Result<()>`  | **KEEP**       | USED in `src/models/layout.rs:2961, 2972, 2980, 2988` (within `#[cfg(test)] mod tests`). |
| 1082 | `ComboSettings::new(enabled: bool) -> Self`       | **KEEP**       | USED in `src/models/layout.rs:3001, 3038, 3069` (within `#[cfg(test)] mod tests`). |
| 1092 | `ComboSettings::add_combo(&mut self, ComboDefinition) -> Result<()>` | **KEEP** | USED in `src/models/layout.rs:3008, 3016, 3024, 3033, 3045, 3053, 3061, 3078` (within `#[cfg(test)] mod tests`). |
| 1121 | `ComboSettings::remove_combo(&mut self, usize) -> Option<ComboDefinition>` | **DELETE** | DEAD. No callers anywhere. |
| 1132 | `ComboSettings::validate(&self) -> Result<()>`    | **DELETE**     | DEAD. No external callers. Internal call from `add_combo` invokes `combo.validate()` directly (not `self.validate()`), so removing this method does not break `add_combo`. |
| 1171 | `impl TapDanceAction` (block-level allow)         | **KEEP**       | Entire impl block is in active use. Per-method callers: `new` (firmware/generator.rs, parser/layout.rs, cli/tap_dance.rs, key_editor.rs, tap_dance_form.rs, validator.rs, popups.rs, tap_dance_docs.rs, tests), `with_double_tap` (parser/layout.rs, cli/tap_dance.rs, validator.rs, tap_dance_docs.rs, tests), `with_hold` (parser/layout.rs, cli/tap_dance.rs, key_editor.rs, tests), `validate` (parser/layout.rs:1273, cli/tap_dance.rs:212, models/layout.rs:2120, tui/tap_dance_form.rs:291, tui/handlers/popups.rs:1292, tests/tap_dance_tests.rs), `is_two_way`/`is_three_way` (cli/tap_dance.rs:140-142, tests), `has_hold` (firmware/generator.rs:1529, tests). All public. No cascade deletes inside. |
| 1599 | `impl LayoutMetadata` (block-level allow)         | **KEEP**       | Most methods are USED externally: `new` (`src/tui/template_browser.rs:504`, plus transitive via `Layout::new`), `touch` (`src/tui/handlers/settings.rs:702, 1083`, `src/tui/handlers/templates.rs:119`, `src/tui/handlers/popups.rs:1166`), `add_tag` (`src/models/layout.rs:2402, 2403, 2408` in `#[cfg(test)]`). **Cascade-delete `set_description` and `set_author` first** (see Cascade section) so the allow can be safely removed. |
| 1779 | `impl Layout` (block-level allow)                 | **KEEP**       | Most methods are USED. Confirmed used: `new`, `add_layer`, `get_layer`, `get_layer_mut`, `add_category`, `get_category`, `remove_category`, `toggle_layer_colors`, `toggle_all_layer_colors`, `resolve_key_color`, `resolve_key_color_if_enabled`, `resolve_display_color`, `apply_rgb_settings`, `get_layer_by_id`, `get_layer_index_by_id`, `add_tap_dance`, `get_tap_dance`, `remove_tap_dance`, `auto_create_tap_dances`, `validate` (parser/layout.rs:135, parser/json_serde.rs:42, web/mod.rs:1659, 2133, firmware/validator.rs:246, tests). **Cascade-delete `any_layer_colors_enabled`, `get_category_mut`, `get_tap_dance_mut` first** (see Cascade section). |

### Summary by decision

- **KEEP** (annotation removed only): 10 sites — 194, 614, 741, 912, 1042, 1082, 1092, 1171, 1599, 1779 (the two block-level allows become safe to remove after cascade-deletes).
- **DELETE** (annotation + method body removed): 6 sites — 628, 765, 878, 934, 1121, 1132.

---

## Cascade Deletes (dead methods inside block-annotated impl blocks)

These methods are NOT directly annotated with `#[allow(dead_code)]`, but they
live inside impl blocks (`1599` and `1779`) that ARE annotated. After
deleting these dead methods, the block-level annotations can be removed.

### Inside `impl LayoutMetadata` (allow at line 1599)

| Line | Method | Reason |
|------|--------|--------|
| 1646 | `LayoutMetadata::set_description(&mut self, impl Into<String>)` | DEAD. `rg '\.set_description\(' src/ tests/` returns zero hits. |
| 1652 | `LayoutMetadata::set_author(&mut self, impl Into<String>)` | DEAD. `rg '\.set_author\(' src/ tests/` returns zero hits. |

### Inside `impl Layout` (allow at line 1779)

| Line | Method | Reason |
|------|--------|--------|
| 1855 | `Layout::get_category_mut(&mut self, &str) -> Option<&mut Category>` | DEAD. `rg '\.get_category_mut\(' src/ tests/` returns zero hits. |
| 1897 | `Layout::any_layer_colors_enabled(&self) -> bool` | DEAD. `rg 'any_layer_colors_enabled' src/ tests/` returns only the definition. (Note: `Layer::toggle_layer_colors` at `models/layer.rs:308` and `tui/keyboard.rs` use `has_hold_like_inbound`, which is unrelated.) |
| 2139 | `Layout::get_tap_dance_mut(&mut self, &str) -> Option<&mut TapDanceAction>` | DEAD. `rg '\.get_tap_dance_mut\(' src/ tests/` returns zero hits. |

---

## Recommended Editing Sequence

1. **Delete dead methods** at lines 628, 765, 878, 934, 1121, 1132 (and their
   `#[allow(dead_code)]` annotations).
2. **Delete cascade-dead methods** at lines 1646, 1652, 1855, 1897, 2139.
3. **Remove `#[allow(dead_code)]` annotation lines** at: 194, 614, 741, 912,
   1042, 1082, 1092, 1171 (the impl-block attribute line), 1599, 1779.

After this, `cargo clippy --all-features -- -D warnings` and
`cargo test` should both still pass.

---

## Verification Commands Run

```
rg -n --type rust 'RgbMatrixEffect::all' src/ tests/
rg -n --type rust 'PaletteFxEffect::all|PaletteFxEffect::display_name' src/ tests/
rg -n --type rust 'PaletteFxPalette::all|PaletteFxPalette::display_name' src/ tests/
rg -n --type rust 'PaletteFxSettings::has_custom_settings' src/ tests/
rg -n --type rust 'ComboAction::(all|description|display_name|from_name)' src/ tests/
rg -n --type rust 'ComboDefinition::validate' src/ tests/
rg -n --type rust 'ComboSettings::(new|add_combo|remove_combo|validate|has_custom_settings)' src/ tests/
rg -n --type rust 'TapDanceAction::(new|with_double_tap|with_hold|validate|is_two_way|is_three_way|has_hold)' src/ tests/
rg -n --type rust 'LayoutMetadata::(new|validate_name|touch|set_description|set_author|add_tag)' src/ tests/
rg -n --type rust 'Layout::(new|add_layer|get_layer|get_layer_mut|add_category|get_category|get_category_mut|remove_category|toggle_layer_colors|toggle_all_layer_colors|any_layer_colors_enabled|resolve_key_color|resolve_key_color_if_enabled|resolve_display_color|apply_rgb_settings|get_layer_by_id|get_layer_index_by_id|add_tap_dance|get_tap_dance|get_tap_dance_mut|remove_tap_dance|auto_create_tap_dances|validate|has_custom_settings)' src/ tests/
```

No build/test commands were executed (read-only task per contract).

---

## Residual Risks

- **Cascade risk at impl blocks 1599 and 1779**: If the editing agent removes
  only the block-level `#[allow(dead_code)]` without first deleting the
  cascade-dead methods (`set_description`, `set_author`, `any_layer_colors_enabled`,
  `get_category_mut`, `get_tap_dance_mut`), the compiler will emit new
  dead-code warnings and `cargo clippy -- -D warnings` will fail. The
  Recommended Editing Sequence above is order-sensitive.

- **`ComboAction::all` and `ComboDefinition::validate` rely solely on test
  callers**: They have no production callers, only `#[cfg(test)] mod tests`
  usages at lines 2889, 2961, 2972, 2980, 2988 respectively. Per prompt
  contract these count as USED. If the project ever removes the in-file
  `#[cfg(test)] mod tests` block, these annotations would need to be
  re-evaluated.

- **`RgbMatrixEffect::display_name`** (line 211) and **`PaletteFxEffect::qmk_mode_name` /
  `enable_define`** are NOT in scope of this audit but are similarly
  potentially dead. Out of scope — only the 16 annotated sites were reviewed.