# 006 – Layer Management Popup

## Problem Statement

The keyboard layout editor currently supports switching between existing layers (Tab / Shift+Tab) but does not provide any UI to add, remove, or rename layers. Users must edit layout files on disk or start from templates with the desired number of layers. This makes common workflows (e.g., adding a new FN layer, reorganizing layers) cumbersome and error-prone.

## Goals / Outcomes

- Allow users to manage layers directly from the TUI without editing files by hand.
- Support adding new layers with a name, default color, and initial key contents policy.
- Support reordering and deleting layers in a safe, undoable-feeling way (with confirmation prompts where destructive).
- Keep the mental model consistent with existing UI: a dedicated popup, keyboard-driven, with clear status messages.

## Non‑Goals

- No full “history/undo” system; we only provide confirmations for destructive actions.
- No per-key cloning/wizard beyond simple policies (e.g., copy from existing layer, start empty, fill KC_TRNS).
- No changes to on-disk layout format beyond what is already supported for multiple layers.

## User Stories

- As a user, I can open a **Layer Manager** popup from the TUI so I can see all layers and their names in one place.
- As a user, I can **add a new layer** by choosing:
  - A layer name
  - A base layer to copy from, or an empty/transparent template
  - (Optionally) a default color
- As a user, I can **delete a layer** (except layer 0) after confirming, so I can clean up unused layers.
- As a user, I can **rename a layer** so that the layer list matches my mental model.
- As a user, I can **reorder layers** (move up/down) so that layer numbers and semantics line up.
- As a user, I can close the popup at any time with Escape without applying changes.

## UX Overview

- Entry point: new keybinding from main editor, e.g. `Ctrl+L` (if free) or another mnemonic to open **Layer Manager**.
- Popup layout:
  - Left: list of layers with index, name, and maybe a small color swatch.
  - Right/bottom: help text showing available actions and confirmations.
- Keyboard controls inside popup (TBD precisely in implementation plan):
  - Up/Down: select layer in list.
  - `n`: new layer.
  - `r`: rename layer.
  - `d`: delete layer (with confirmation).
  - `u` / `j` or similar: move layer up/down.
  - `Enter`: confirm an in-progress edit.
  - `Esc`: cancel current edit or close popup.

## Data Model Impact

- The existing `Layout` and `Layer` structs already support multiple layers; the popup will operate on:
  - `AppState.layout.layers: Vec<Layer>`
  - `AppState.current_layer: usize`
- No new persistent fields are required; changes are persisted when the user saves the layout as usual.

## TUI / Code Integration Points

- New popup type in `PopupType` enum, e.g. `PopupType::LayerManager`.
- New state struct in `tui` module for layer manager (selection index, edit mode, temporary input buffer).
- New keybinding in `handle_key_event` to open the popup from the main editor.
- Dedicated render function for the popup using Ratatui components.
- Wiring into existing save/dirty tracking: any structural change to `layout.layers` should mark the layout dirty and update status messages.

## Risks & Edge Cases

- Deleting a layer that is currently selected (update `current_layer` to a valid index).
- Reordering layers while the current layer is one of them (keep selection on the same logical layer after move).
- Large numbers of layers (ensure the list scrolls rather than overflowing the popup).
- Keyboard shortcuts overlapping with existing bindings (choose non-conflicting keys or adjust help text accordingly).

## Open Questions

- Exact keybinding for opening the Layer Manager (Ctrl+L is already used for build log; we need an alternative, e.g. `Ctrl+Shift+L` or a different mnemonic).
- Default behavior for new layer contents (empty vs KC_TRNS vs copy from another layer) – likely offer a small choice in a sub-dialog.
- Whether reordering should be fully supported in v1, or deferred if complexity is high.
