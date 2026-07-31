# WebUI Feature Gaps — Hidden or Unreachable Capabilities

> **Scope**: Audit of features present in the LazyQMK Rust backend, TUI, or CLI that are **missing, hidden, or hard to reach** in the WebUI (`web/src/`).
>
> **Method**: Walked every Rust surface (`src/cli/`, `src/tui/popup_type.rs`, `src/web/routes/`, `src/services/`), cross-referenced against every WebUI surface (`web/src/routes/`, `web/src/lib/components/`, `web/src/lib/api/client.ts`).
>
> **Generated**: 2026-07-31, immediately after the Settings tab + modal layer picker release (v0.24.0).
>
> **Prioritization**: P0 = clear bug or parity gap that blocks a real user flow; P1 = discoverability gap; P2 = nice-to-have.

---

## P0 — Missing Capability (no workaround in WebUI)

### 1. Layer References & Transparency Warnings (`layer-refs`)
- **Backend**: `src/cli/layer_refs.rs:1` exposes `lazyqmk layer-refs <file>`. Parses all `LT/TO/MO/TT/LM/TG/OSL/...` references, builds inbound index per layer, and emits transparency conflicts where a hold-like reference points at a non-`KC_TRNS` key.
- **TUI**: `src/tui/handlers/layer_refs.rs` renders the same in-editor.
- **WebUI**: `/api/layouts/{filename}/validate` (`src/web/routes/validate.rs:1`) only emits `ValidationErrorKind::EmptyLayer` for layer-refs failures. The full inbound-reference index and per-position transparency warnings are unreachable from the browser.
- **Impact**: A user editing `corne_choc_pro_enhanced` cannot see which keys on Layer 2 are referenced by `LT(2, …)` keys on Layer 0, nor get warned when a hold key on Layer 0 lands on a non-`KC_TRNS` slot on Layer 2.
- **Suggested fix**: Add `/api/layouts/{filename}/layer-refs` endpoint mirroring `src/cli/layer_refs.rs:60`, render as a section in the Review tab (`web/src/routes/layouts/[name]/+page.svelte:3175`) and surface warnings as a banner in the Editor tab.

### 2. Keycode parameter picker (two-stage flow for `LT/MT/LM/LCG/MEH/HYPR/TD`)
- **Backend**: `src/keycode_db/parameterized.rs` defines `KeycodeParam` + `ParamType` (Layer / Modifier / Keycode / TapDance). `src/tui/handlers/popups/parameterized.rs:120` opens `LayerPicker`, `ModifierPicker`, `TapKeycodePicker`, or `TapDanceForm` as a second stage after the user selects the prefix keycode.
- **WebUI**: `web/src/lib/components/KeycodePicker.svelte:1` is a search-and-select-only picker. There is no `LayerPicker`, `ModifierPicker`, or `TapKeycodePicker` component. Users must hand-type `LT(2, KC_SPC)` strings.
- **Impact**: Power users are slowed down; new users hit a brick wall because `LT`, `MT`, `LM`, `LCG`, `MEH`, `HYPR`, `TD(...)` keycodes have no discoverable authoring path in the WebUI.
- **Suggested fix**: Add `LayerPicker` / `ModifierPicker` components (`src/tui/picker/layer_picker.rs:1`, `src/tui/picker/modifier_picker.rs:1` already define the models). Detect parameterized keycodes in `KeycodePicker` and chain to the right second-stage picker before returning the assembled keycode.

### 3. Combo keycode part-editing (e.g. only change the hold or tap of `LT(2, KC_SPC)`)
- **Backend**: `src/tui/editor/key_editor.rs:46` (`ComboKeycodeType` enum) parses `LayerTap` / `ModTapNamed` / `ModTapCustom` / `LayerMod` / `ModCombo`. `key_editor.rs:177` tracks `combo_edit: Option<(ComboEditPart, ComboKeycodeType)>`. `key_editor.rs:227` lets the user edit only the hold or only the tap without rewriting the whole keycode.
- **WebUI**: `web/src/routes/layouts/[name]/+page.svelte:2436` exposes a single "Choose New Keycode" button. Selecting a key in the picker replaces the whole keycode.
- **Impact**: Editing a layer-tap's layer while preserving the tap key (or vice versa) requires retyping the full `LT(2, KC_SPC)` string in the picker.
- **Suggested fix**: Surface `Edit hold / Edit tap` buttons in the Key Details panel when the active keycode is parameterized; reuse the new `LayerPicker` / `ModifierPicker` / `KeycodePicker` from gap #2.

### 4. Firmware flash tool (one-click upload to keyboard)
- **TUI**: `src/tui/handlers/firmware/qmk_flash.rs:1` (or equivalent) implements a guided flash flow over USB bootloader.
- **Backend**: `src/cli/qmk/` has `list-keyboards`, `list-layouts`, `geometry` subcommands. No `flash` CLI subcommand yet, but the firmware pipeline emits UF2/Hex/Bin artifacts (`src/firmware/`).
- **WebUI**: WebUI builds artifacts (`web/src/routes/layouts/[name]/+page.svelte:3272`) and offers them as downloads. No "Flash to keyboard" button.
- **Impact**: Browser users must `scp` / drag the .uf2 into a mounted drive after each rebuild, even though the WebUI already started the build.
- **Suggested fix**: Surface a "Flash" button next to the Build Artifact download button. First iteration: open `webflasher`-style instructions or instruct the user to copy the .uf2 to a mounted RP2040/STM32 drive; longer-term, use WebUSB (`navigator.usb`) to push directly to the bootloader.

---

## P1 — Discoverability gap (technically reachable but easy to miss)

### 5. No "?" Help / Keybindings overlay
- **Backend**: `src/tui/dialog/help_overlay.rs:1` + `src/tui/dialog/help_registry.rs:1` ship an interactive help system with keybindings sourced from `src/data/help.toml`. CLI exposes `lazyqmk show-help`.
- **WebUI**: No keyboard-shortcut overlay, no `?`-key handler in `+layout.svelte` or any `+page.svelte`. The AGENTS.md mentions "ALL help text must come from `src/data/help.toml`" but the WebUI never loads it.
- **Impact**: Mouse-first users miss shortcuts (`Ctrl+S` save, `Ctrl+G` generate, `Ctrl+B` build). Power users have no way to learn them in-product.
- **Suggested fix**: Add a `Help` route (`/help`) or a `?`-key handler that opens an `AccessibleDialog` populated from `src/data/help.toml` via a `/api/help` endpoint. The data source is already structured for this — only the transport and a Svelte renderer are missing.

### 6. Per-key "open in KeyEditor" deep link
- **TUI**: KeyEditor popup (`src/tui/popup_type.rs:81`) opens on `Enter` and lets the user edit just one key's description, color, category, or split-edit a combo in place.
- **WebUI**: The Key Details panel (`web/src/routes/layouts/[name]/+page.svelte:2400`) shows the same fields inline in the right sidebar. Functionally equivalent, but there's no way to deep-link from the export markdown or a URL into a specific (layer, key) view.
- **Impact**: Sharing "look at this key" requires a screenshot, not a URL.
- **Suggested fix**: Add `?layer=2&key=4,3` query-param parsing to the layout page that auto-selects + scrolls to the key. ~30 lines of Svelte + a small effect.

### 7. Export filename override
- **TUI**: `PopupType::ExportFilenameDialog` (`src/tui/popup_type.rs:62`) prompts the user for an export filename.
- **WebUI**: `web/src/routes/layouts/[name]/+page.svelte:3325` hardcodes `exportResult.suggested_filename` from the server; no input field.
- **Impact**: Browser users always get `corne_choc_pro_export_2026-07-31.md`; the TUI lets you override.
- **Suggested fix**: Add an `Input` next to the export Download button that lets the user rename before download. Trivial change.

### 8. Save as Template: missing tags input affordance
- **WebUI**: `web/src/routes/layouts/[name]/+page.svelte:3471` has the tags input. But `layout.metadata.tags` editing on the Metadata tab (`web/src/routes/layouts/[name]/+page.svelte:2017`) is the only place tags are surfaced.
- **TUI**: Template browser popup (`src/tui/picker/template_browser.rs:1`) lets you tag templates at view time with quick filters.
- **Impact**: Minor — tag-based template discovery is missing in WebUI's `/templates` route.
- **Suggested fix**: Wire `tags` filters into `web/src/routes/templates/+page.svelte` (currently the route doesn't exist as a route file; only `/web/src/routes/templates/` path mentions templates in the breadcrumb). Check first whether this is a TODO or already in-progress.

### 9. No "Open file on disk" / "Reveal in Finder"
- **TUI**: `src/tui/dialog/` includes a layout-picker that shows file paths inline.
- **WebUI**: Settings page (`web/src/routes/settings/+page.svelte:135`) shows `workspace_root` as a `<code>` block; no click-to-open handler.
- **Impact**: WebUI users have to navigate to the workspace folder manually with their OS file browser.
- **Suggested fix**: If served from Tauri, use `@tauri-apps/api`'s opener. If served standalone, at minimum surface a `file://` link where the OS allows it. Otherwise copy-to-clipboard as fallback.

---

## P2 — Nice-to-have / future enhancement

### 10. Tap dance validator (`lazyqmk tap-dance validate`)
- **Backend**: `src/cli/tap_dance.rs:85` `ValidateArgs` checks tap dance entries for orphan keycodes and unused definitions.
- **WebUI**: Validate endpoint covers tap-dance warnings under the `tap_dances` check (`src/cli/common.rs:119`), but the dedicated `tap-dance validate` deep analysis is unreachable. WebUI validate (`web/src/routes/layouts/[name]/+page.svelte:3199`) only shows a generic green/red panel.

### 11. Layout reachability audit (`scripts/audit-layout.sh` / `layout-reachability.sh`)
- **Tooling**: Per `.beads/issues.jsonl`, there are dedicated audit scripts (`references/0008-layout-reachability.md`, `lessons/0006-populate-layers.md`) that produce per-character reachability tables.
- **WebUI**: No equivalent. The Review tab (`+page.svelte:3175`) shows layer/category counts but no character-level reachability.
- **Impact**: Hard for users to detect gaps (e.g. "I want `é` on this layout but no layer has it").
- **Suggested fix**: Add a `/api/layouts/{filename}/reachability?char=X` endpoint and a search panel under Review.

### 12. Doctor command (`lazyqmk doctor`)
- **Backend**: `src/cli/doctor.rs:1` checks for missing QMK CLI, Python version, module installs.
- **WebUI**: None. The preflight endpoint (`src/web/routes/config.rs` `get_preflight`) is the closest, but only checks QMK path presence, not the rest.
- **Suggested fix**: Add `/api/doctor` and surface a "Setup health" panel in `/onboarding`.

### 13. CLI-only `geometry` / `inspect` flags
- **Backend**: `lazyqmk geometry <keyboard> <layout>` (`src/cli/qmk/geometry.rs:1`) prints a matrix-coord dump. `lazyqmk inspect --section settings` (`src/cli/inspect.rs:1`) lets you target a single section.
- **WebUI**: Geometry data is fetched (`/api/keyboards/{k}/geometry/{l}`), but only consumed for rendering. No way to view the raw matrix map or single-section inspect output.
- **Suggested fix**: Add a "Raw" toggle in the Review tab showing the JSON.

### 14. Show layer references in the Layer Manager (currently silent)
- **Backend**: `src/services/layer_refs.rs:1` `build_layer_ref_index` is a public function.
- **WebUI**: Layer Manager (`web/src/lib/components/LayerManager.svelte:1`) shows key count and color per layer but not inbound reference count.
- **Suggested fix**: Surface `inbound_refs.length` next to `layer.keys.length` so users can spot orphan or over-referenced layers at a glance.

### 15. Tap dance picker for parameterized keycode flow (`ParamType::TapDance`)
- **Backend**: `src/tui/handlers/popups/parameterized.rs:136` launches `TapDanceForm` with `FromKeycodePicker` context, so `TD(name)` flows seamlessly.
- **WebUI**: No equivalent. A user wanting `TD(some_name)` would have to first create the tap dance in the Tap Dance tab, then remember the name and type it.
- **Suggested fix**: Same fix as gap #2 — chain into a TapDanceForm modal when the user picks `TD()`.

### 16. Status bar hotkeys reference
- **TUI**: `src/tui/dialog/status_bar.rs:1` shows current shortcut hints in the bottom bar.
- **WebUI**: No persistent status bar. Keyboard shortcuts are mentioned in `docs/WEB_FEATURES.md` but not discoverable in-product.
- **Suggested fix**: Add a thin footer hint strip with the top 3–4 shortcuts (`Ctrl+S`, `Ctrl+G`, `Ctrl+B`, `?`).

### 17. Custom keymap name + output format editor beyond what Metadata tab exposes
- **Backend**: `LayoutMetadata.keymap_name`, `output_format` (`src/models/layout/layout_core.rs:1`) are settable via YAML frontmatter.
- **WebUI**: Metadata tab shows them only in the read-only `System Information` section (`web/src/routes/layouts/[name]/+page.svelte:2080`).
- **Impact**: Users can't rename `corne_choc_pro` to `corne_choc_pro_enhanced` from the UI.
- **Suggested fix**: Add a small form for `keymap_name` + `output_format` dropdown (uf2/hex/bin) to the Metadata tab.

### 18. `description` on `Layer`
- **Backend**: `Layer.description` is a settable field (`src/models/layout/layout_core.rs`).
- **WebUI**: Layer Manager shows `layer.name` and color but not description.
- **Suggested fix**: Add a description input to the layer rename dialog in LayerManager.

### 19. Workspace switching at runtime
- **Backend**: Web server takes `--workspace` at startup (`src/main.rs:481`).
- **WebUI**: Workspace is fixed for the server lifetime. No `/api/workspace/switch` endpoint.
- **Suggested fix**: Either add `?workspace=...` query handling on a route, or document the limitation and link to restart instructions.

### 20. CLI `preview` (single-layer ASCII art with highlights)
- **Backend**: `src/cli/preview.rs:1` renders one layer with highlight markers, used by agents to embed inline previews in chat.
- **WebUI**: Full keyboard preview exists (`web/src/lib/components/KeyboardPreview.svelte:1`), but no way to render a single layer as ASCII for sharing.
- **Suggested fix**: Add an "ASCII preview" action to the Review tab.

---

## Quick Triage

| # | Area | Backend file:line | Effort |
|---|---|---|---|
| 1 | Layer refs | `src/cli/layer_refs.rs:60` | M |
| 2 | Parameter picker | `src/tui/picker/{layer,modifier,tap_dance}_picker.rs:1` | M |
| 3 | Combo part editing | `src/tui/editor/key_editor.rs:46` | M |
| 4 | Flash tool | `src/tui/handlers/firmware/qmk_flash.rs:1` | XL |
| 5 | Help overlay | `src/tui/dialog/help_overlay.rs:1` + `src/data/help.toml` | S |
| 6 | Key deep link | new (UI only) | XS |
| 7 | Export filename | new (UI only) | XS |
| 8 | Template tags | `web/src/routes/templates/` | S |
| 9 | Reveal in Finder | Tauri-only | S |
| 10 | TD validate | `src/cli/tap_dance.rs:85` | S |
| 11 | Reachability | `scripts/audit-layout.sh` | M |
| 12 | Doctor | `src/cli/doctor.rs:1` | S |
| 13 | Raw JSON | new (UI only) | XS |
| 14 | Layer refs in Manager | `src/services/layer_refs.rs:1` | XS |
| 15 | TD in picker flow | rolled into #2 | – |
| 16 | Status bar hints | new (UI only) | S |
| 17 | Metadata form | `LayoutMetadata` | S |
| 18 | Layer description | `Layer.description` | XS |
| 19 | Workspace switch | new endpoint | M |
| 20 | ASCII preview | `src/cli/preview.rs:1` | S |

---

## Suggested Bead Candidates (priority order)

1. **LazyQMK-xxx**: `/api/layouts/{filename}/layer-refs` endpoint + Review-tab banner.
2. **LazyQMK-xxx**: Parameterized keycode second-stage picker chain (LayerPicker / ModifierPicker / TD form).
3. **LazyQMK-xxx**: Combo keycode hold/tap split editor in Key Details panel.
4. **LazyQMK-xxx**: WebUI help overlay backed by `src/data/help.toml`.
5. **LazyQMK-xxx**: Per-layer description editing.
6. **LazyQMK-xxx**: Tap-dance validator endpoint + Review-tab integration.
7. **LazyQMK-xxx**: ASCII preview action in Review tab.
8. **LazyQMK-xxx**: Export filename override input.
9. **LazyQMK-xxx**: Inline reachability audit (`/api/layouts/{filename}/reachability`).
10. **LazyQMK-xxx**: Firmware flash (WebUSB or copy-to-drive UX).