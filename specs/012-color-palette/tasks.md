# Tasks: 012-color-palette

## Phase 1: Color Palette Data
- [ ] Create `src/data/color_palette.json` with Tailwind-inspired colors
- [ ] Create `src/models/color_palette.rs` with data structures
- [ ] Implement palette loading from JSON
- [ ] Export from `src/models/mod.rs`
- [ ] Add unit tests for palette loading

## Phase 2: Update ColorPickerState
- [ ] Add `ColorPickerMode` enum (Palette, CustomRgb)
- [ ] Add palette selection fields to `ColorPickerState`
- [ ] Add mode switching methods
- [ ] Sync RGB values when selecting from palette

## Phase 3: Render Palette UI
- [ ] Create layout with two sections (palette + shades)
- [ ] Render base color grid (4x3) with colored dots and names
- [ ] Render shade bar for selected color
- [ ] Highlight selected color and shade
- [ ] Show preview and hex code
- [ ] Update instructions based on mode

## Phase 4: Handle Input
- [ ] Arrow key navigation in palette mode
- [ ] Mode switching (`c` for custom, `p` for palette)
- [ ] Maintain existing RGB mode input handling
- [ ] Apply color on Enter

## Phase 5: Update Help Overlay
- [ ] Document new palette mode controls
- [ ] Document mode switching keys
