# Dependency Updates Plan

**Status:** In Progress  
**Created:** 2025-12-10  
**Last Updated:** 2025-12-10

## Overview

Update outdated dependencies identified in the dependency audit. Packages will be updated incrementally in priority order, with testing after each phase to ensure stability.

## Current State

### Dependencies Audit Summary

Based on analysis of `Cargo.toml` vs latest crates.io versions:

**Current Versions:**
```toml
[dependencies]
anyhow = "1.0"
arboard = "3.4"
clap = { version = "4.0", features = ["derive"] }
crossterm = "0.27"
dirs = "5.0"
json5 = "0.4"
ratatui = "0.26"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"
toml = "0.8"
uuid = { version = "1.0", features = ["v4", "serde"] }

[dev-dependencies]
tempfile = "3.3"
```

### Priority Levels

#### **Critical Updates (Major Versions Behind)**
1. **json5**: `0.4` → `1.3.0` (3 major versions)
   - Just added in previous commit for QMK JSON5 support
   - API likely changed significantly

2. **dirs**: `5.0` → `6.0.0` (1 major version)
   - Used for finding user config directories
   - Breaking changes expected

#### **High Priority (Multiple Versions Behind)**
3. **ratatui**: `0.26` → `0.29.0` (4 minor versions)
   - Core TUI framework, affects entire UI
   - May have new features/optimizations
   - Pre-1.0, minor versions can have breaking changes

4. **crossterm**: `0.27` → `0.29.0` (2 minor versions)
   - Terminal backend for ratatui
   - Should be updated in sync with ratatui

5. **clap**: `4.0` → `4.5.53` (5 minor versions)
   - CLI argument parsing
   - Likely bug fixes and new features

#### **Medium Priority**
6. **serde_yaml**: `0.9` → **DEPRECATED** → Migrate to `serde_yml` `0.0.12`
   - Original maintainer archived the project
   - New fork: `serde_yml` is the recommended replacement
   - Used for YAML frontmatter in layout files

7. **toml**: `0.8` → `0.9.8` (1 minor + patches)
   - Used for config.toml parsing
   - Likely minor improvements

8. **arboard**: `3.4` → `3.6.1` (2 patch versions)
   - Clipboard functionality
   - Bug fixes likely

#### **Low Priority**
9. **uuid**: `1.0` → `1.19.0` (19 patch versions)
   - Used for generating unique IDs
   - Stable API, just patches

#### **Up-to-Date Packages**
- ✅ `anyhow`: 1.0 (latest stable)
- ✅ `serde`: 1.0 (latest stable)
- ✅ `serde_json`: 1.0 (latest stable)
- ✅ `tempfile`: 3.3 (dev dependency, relatively recent)

## Implementation Plan

### Phase 1: Critical Updates (Breaking Changes Expected)

#### Task 1.1: Update json5 (0.4 → 1.3.0)
**Files:** `Cargo.toml`, `src/parser/keyboard_json.rs`

**Steps:**
1. Update `Cargo.toml`: `json5 = "1.3"`
2. Run `cargo build` to check for API changes
3. Fix any breaking changes in parsing code
4. Run tests: `cargo test --test qmk_info_json_tests`
5. Verify splitkb/aurora/lily58/rev1 still parses

**Expected Changes:**
- API may have changed (different error types, method names)
- Check release notes: https://github.com/callum-oakley/json5-rs/releases

#### Task 1.2: Update dirs (5.0 → 6.0.0)
**Files:** `Cargo.toml`, `src/config.rs`

**Steps:**
1. Update `Cargo.toml`: `dirs = "6.0"`
2. Run `cargo build` to check for API changes
3. Fix any breaking changes in config path resolution
4. Run full test suite: `cargo test`
5. Manually test config loading on startup

**Expected Changes:**
- Method renames or signature changes
- Possibly different platform-specific behavior
- Check migration guide: https://github.com/soc/dirs-rs

### Phase 2: High Priority (UI Framework + CLI)

#### Task 2.1: Update ratatui (0.26 → 0.29.0)
**Files:** `Cargo.toml`, potentially all `src/tui/*.rs` files

**Steps:**
1. Update `Cargo.toml`: `ratatui = "0.29"`
2. Run `cargo build` and note all breaking changes
3. Check changelog: https://github.com/ratatui-org/ratatui/releases
4. Fix rendering issues systematically:
   - Start with `src/tui/mod.rs`
   - Fix each component file
5. Run full test suite: `cargo test`
6. Manual UI testing: `cargo run` and test all screens

**Risk Level:** High - affects entire UI
**Rollback Plan:** Revert commit if UI breaks significantly

#### Task 2.2: Update crossterm (0.27 → 0.29.0)
**Files:** `Cargo.toml`, `src/tui/mod.rs`, `src/main.rs`

**Steps:**
1. Update `Cargo.toml`: `crossterm = "0.29"`
2. Run `cargo build` to check for API changes
3. Check changelog: https://github.com/crossterm-rs/crossterm/releases
4. Fix any terminal backend issues
5. Run full test suite: `cargo test`
6. Manual testing: terminal initialization, cleanup, input handling

**Note:** Should be compatible with ratatui 0.29

#### Task 2.3: Update clap (4.0 → 4.5.53)
**Files:** `Cargo.toml`, `src/main.rs`

**Steps:**
1. Update `Cargo.toml`: `clap = { version = "4.5", features = ["derive"] }`
2. Run `cargo build` (likely no breaking changes within v4)
3. Test CLI args: `cargo run -- --help`
4. Test all CLI flags if any exist
5. Run full test suite: `cargo test`

**Risk Level:** Low - v4.x is stable

### Phase 3: Medium Priority (Format Parsers)

#### Task 3.1: Migrate serde_yaml → serde_yml (DEPRECATED)
**Files:** `Cargo.toml`, `src/parser/layout.rs`, any other YAML parsing

**Steps:**
1. Remove `serde_yaml = "0.9"` from `Cargo.toml`
2. Add `serde_yml = "0.0.12"` to `Cargo.toml`
3. Find all imports: `rg "use serde_yaml"`
4. Replace `serde_yaml::` with `serde_yml::`
5. Run `cargo build` and fix any API differences
6. Run layout parsing tests
7. Test loading actual layout files

**Migration Guide:** https://github.com/sebastienrousseau/serde_yml

**Risk Level:** Medium - different crate, may have subtle differences

#### Task 3.2: Update toml (0.8 → 0.9.8)
**Files:** `Cargo.toml`, `src/config.rs`

**Steps:**
1. Update `Cargo.toml`: `toml = "0.9"`
2. Run `cargo build` to check for breaking changes
3. Fix any API changes in config parsing
4. Test config file loading
5. Run full test suite: `cargo test`

**Risk Level:** Low-Medium - minor version bump

#### Task 3.3: Update arboard (3.4 → 3.6.1)
**Files:** `Cargo.toml`, `src/tui/clipboard.rs`

**Steps:**
1. Update `Cargo.toml`: `arboard = "3.6"`
2. Run `cargo build` (likely no breaking changes)
3. Test clipboard functionality manually
4. Run full test suite: `cargo test`

**Risk Level:** Low - patch updates

### Phase 4: Low Priority (Stable Packages)

#### Task 4.1: Update uuid (1.0 → 1.19.0)
**Files:** `Cargo.toml`

**Steps:**
1. Update `Cargo.toml`: `uuid = { version = "1.19", features = ["v4", "serde"] }`
2. Run `cargo build` (no breaking changes expected)
3. Run full test suite: `cargo test`

**Risk Level:** Very Low - patch updates only

### Phase 5: Final Verification

#### Task 5.1: Full Integration Testing
**Steps:**
1. Run `cargo clean` to ensure fresh build
2. Run `cargo build --release`
3. Run `cargo test` (all tests)
4. Run `cargo clippy` (zero warnings)
5. Manual testing checklist:
   - Launch app: `cargo run`
   - Load keyboard (especially splitkb/aurora/lily58/rev1)
   - Navigate all screens (keycode picker, layer picker, settings)
   - Test clipboard operations
   - Edit keycodes and save layout
   - Build firmware (if possible)
   - Test theme changes
6. Check startup warnings (should be clean)

#### Task 5.2: Documentation Updates
**Files:** `AGENTS.md`, `specs/021-dependency-updates/plan.md`

**Steps:**
1. Update `AGENTS.md` "Active Technologies" section with new versions
2. Mark this plan as "Completed"
3. Document any issues encountered and resolutions
4. Note any new features available from updates

## Testing Strategy

### Per-Phase Testing
- Run `cargo test` after each package update
- Manual testing for UI changes (ratatui/crossterm)
- Specific feature testing (clipboard, config loading, parsing)

### Rollback Strategy
- Each phase is a separate commit
- If critical issues found, can revert specific commits
- Keep detailed notes on what breaks and why

### Success Criteria
- ✅ All 292+ tests passing
- ✅ Zero clippy warnings
- ✅ App launches without errors
- ✅ All core features working (keyboard loading, editing, saving)
- ✅ QMK JSON5 parsing still works (splitkb/aurora/lily58/rev1)

## Risk Assessment

### High Risk Updates
1. **ratatui** (0.26 → 0.29) - Entire UI depends on this
2. **json5** (0.4 → 1.3) - Just added, may have different API

### Medium Risk Updates
3. **serde_yaml → serde_yml** - Different crate entirely
4. **crossterm** (0.27 → 0.29) - Terminal backend
5. **dirs** (5.0 → 6.0) - Config paths

### Low Risk Updates
6. **clap**, **toml**, **arboard**, **uuid** - Stable APIs

## Timeline

**Estimated Time:** 2-4 hours
- Phase 1 (Critical): 30-60 min
- Phase 2 (High Priority): 60-90 min
- Phase 3 (Medium Priority): 30-45 min
- Phase 4 (Low Priority): 10 min
- Phase 5 (Verification): 30-45 min

## Notes

### Why Now?
- Just added json5 in previous commit - good time to update it to latest
- Multiple packages are several versions behind
- Pre-1.0 packages (ratatui, crossterm) need more frequent updates
- serde_yaml is officially deprecated

### Benefits
- Bug fixes and performance improvements
- New features in ratatui/crossterm
- Security patches (especially in older versions)
- Better maintenance - staying current reduces future technical debt

### Alternatives Considered
- **Update all at once**: Rejected - too risky, hard to debug
- **Update only critical**: Rejected - other packages also significantly behind
- **Defer updates**: Rejected - only gets worse over time

## References

- Cargo.toml current state: commit `ba9a7ca`
- Dependency audit notes: (this document)
- Crates.io: https://crates.io
- serde_yml migration: https://github.com/sebastienrousseau/serde_yml
- ratatui releases: https://github.com/ratatui-org/ratatui/releases
- crossterm releases: https://github.com/crossterm-rs/crossterm/releases
