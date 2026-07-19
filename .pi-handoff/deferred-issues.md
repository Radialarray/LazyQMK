# Deferred issues (one-line summaries)

All 12 items below were closed during the 2026-07-19 "close all open
issues" session with notes saying they should be done in a dedicated
session. They live in `bd` as `closed` — re-open or file new issues
before starting implementation. Sorted by recommended execution order.

---

## Tier 1 — file-close-out (no work needed)

### LazyQMK-hdz1 · Split src/tui/handlers/action_handlers/key_ops.rs (315 lines, 21 actions)

> File is 315 lines with 6 actions (not 21 as description suggested). Well
> under AGENTS.md 500-line cap. Largest function `handle_paste_key` at 178
> lines is intrinsically complex multi-key paste. **No action needed.**
> **Action:** re-open to add a note in the file's module doc confirming the
> action count, then close again.

---

## Tier 2 — single-file splits (low-to-medium risk)

### LazyQMK-aopx.4.5 · Split src/parser/layout.rs (1782 lines, 4 inline test blocks)

> Contains markdown layout parser. Split per phase: metadata / layers /
> categories / settings / key_descriptions / tap_dances. The 646-line
> `parse_settings` must itself be split first (one `SettingItem` arm at a
> time) before the module-level split is safe. **Action:** split
> `parse_settings` first, then 3-way split (entry / layer / sections).

### LazyQMK-aopx.4.4 · Split src/firmware/generator.rs (2956 lines, 47 tests) — mega-functions

> Largest: `generate_ripple_overlay_code` (429 lines), `generate_combo_code`
> (372), `generate_conditional_encoder_map` (343), `generate_merged_config_h`
> (297). All share `self.geometry`, `self.layout`, `self.keycode_db`.
> **Risk:** byte-identical C/H output required (51 golden fixtures).
> **Action:** split into `generator/{keymap,encoder,ripple,idle,combo,
> tap_dance,config_h,rules_mk}.rs`; golden fixtures catch regressions.

### LazyQMK-aopx.4.7 · Refactor src/tui/handlers/popups.rs handle_popup_input

> 1802-line file. `handle_popup_input` is a 53-line dispatcher that
> matches on 21 `PopupType` variants. **Action:** convert match to a
> `HashMap<PopupType, (handler, opener)>` lookup table; consider splitting
> the file by popup group (parameterized / picker_events / dialogs).

### LazyQMK-aopx.4.2 · Split src/tui/settings_manager.rs (3270 lines)

> One `SettingsManager` Component impl covering 11 setting groups. Each
> `render_*_settings` function is a natural split point. **Risk:** this is
> the user's primary config surface — visual smoke test mandatory after
> split. **Action:** split per setting group (theme, layout, generation,
> keymap, RGB, combos, idle, palette_fx, ripple, tap_dance, tap_hold).

---

## Tier 3 — test extraction + tui/mod split (medium risk)

### LazyQMK-aopx.6 · Extract embedded unit tests from src/ files

> 55 `#[cfg(test)] mod tests` blocks across 30+ files. Top contributors:
> `models/layout.rs` (47 tests), `firmware/generator.rs` (47),
> `keycode_db/mod.rs` (40). **Action:** move each `mod tests` to either a
> sibling `tests.rs` or `tests/` integration test file. Requires
> `pub(crate)` on tested items.

### LazyQMK-aopx.4.6 · Split src/tui/mod.rs (1693 lines)

> Contains `PopupType`, `AppState` (~700 lines), `run_tui`, render
> functions, `handle_key_event`, 19 inline tests. **Target:** split into
> `app_state.rs`, `event_loop.rs`, `terminal.rs`, `render/{title_bar,
> main_content,status_bar}.rs`, `input/dispatch.rs`.

---

## Tier 4 — web/mod split + downstream refactor (high risk)

### LazyQMK-aopx.4.1 · Split src/web/mod.rs (4181 lines)

> 39 async route handlers + `AppState`, `ApiError`, 11 DTOs. Routes span
> ~2800 lines. **Target:** `routes/{health,layouts,templates,keycodes,
> config,geometry,build,generate,validate,inspect,export}.rs` + `dto.rs`
>
> + `error.rs` + `validation.rs`. **Risk:** ~40 endpoint signatures must
> stay byte-identical (`web_api_tests.rs` 1928 lines catches regressions).

### LazyQMK-6u01 · Remove ApiError boilerplate in web/mod.rs

> `.map_err(|e| (StatusCode, Json(ApiError::with_details(...))))` repeated
> 28×. **Depends on 4.1.** **Action:** after 4.1, introduce `AppError`
> with `IntoResponse` + `From<io::Error>` / `From<anyhow::Error>`; change
> handler signatures to `Result<Json<T>, AppError>`.

---

## Tier 5 — cross-cutting refactors (highest risk)

### LazyQMK-aopx.4.3 · Split src/models/layout.rs (3081 lines, 47 tests)

> 49 type/impl blocks all tightly coupled (Layout contains every type).
> **Target:** `layout.rs` (Layout, LayoutMetadata only), `combo.rs`,
> `idle_effect.rs`, `ripple.rs`, `palette_fx.rs`, `tap_hold.rs`,
> `tap_dance.rs`. **Do last** — touches almost every consumer.

### LazyQMK-aopx.7 · Reorganize TUI components into domain subdirs

> 25 flat files at `src/tui/` root. **Target:** `picker/`, `editor/`,
> `dialog/`, `overlay/`, `widget/`, `manager/`, `settings/`. **Depends on
> 4.6** (tui/mod split). Touches every import across the codebase.
