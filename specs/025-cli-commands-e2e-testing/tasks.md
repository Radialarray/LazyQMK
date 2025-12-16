# Spec 025: CLI Commands & End-to-End Testing - Tasks

## Phase 1: Core Commands & Fixtures (Week 1)
**Priority: High**

### Setup & Architecture
- [ ] Create `src/cli/` module structure
- [ ] Add CLI module declarations to `src/lib.rs`
- [ ] Set up clap subcommands in `src/main.rs`
- [ ] Define common CLI types (exit codes, JSON schemas)
- [ ] Create `tests/fixtures/mod.rs` module

### Test Infrastructure
- [ ] Implement shared fixture builders:
  - [ ] `test_layout_basic(rows, cols) -> Layout`
  - [ ] `test_layout_with_tap_dances() -> Layout`
  - [ ] `test_layout_with_idle_effect(enabled) -> Layout`
  - [ ] `test_geometry_basic(rows, cols) -> KeyboardGeometry`
  - [ ] `test_mapping_basic(rows, cols) -> VisualLayoutMapping`
  - [ ] `temp_config_with_qmk(path) -> Config`
- [ ] Create `tests/golden/` directory structure
- [ ] Implement golden test helper:
  - [ ] `assert_golden(actual, golden_path)`
  - [ ] `normalize_output(content) -> String` (strip timestamps/UUIDs)
  - [ ] Support `UPDATE_GOLDEN=1` env var

### Validate Command
- [ ] Create `src/cli/validate.rs`
- [ ] Implement `ValidateArgs` struct with clap derive
- [ ] Implement validation logic (delegate to existing validator)
- [ ] Add JSON output format
- [ ] Add `--strict` mode
- [ ] Define exit codes (0=success, 1=errors, 2=IO)
- [ ] Add E2E tests:
  - [ ] Valid layout → exit 0
  - [ ] Invalid keycode → exit 1
  - [ ] Missing position → exit 1
  - [ ] JSON output structure
  - [ ] Strict mode behavior

### Generate Command
- [ ] Create `src/cli/generate.rs`
- [ ] Implement `GenerateArgs` struct
- [ ] Add `--deterministic` flag logic
- [ ] Implement generation (delegate to FirmwareGenerator)
- [ ] Add format selection (keymap|config|all)
- [ ] Add E2E tests:
  - [ ] Basic generation succeeds
  - [ ] Deterministic mode produces stable output
  - [ ] Format selection works
- [ ] Add golden tests:
  - [ ] `tests/golden/keymap_basic.c`
  - [ ] `tests/golden/config_basic.h`
  - [ ] Idle effect on vs off
  - [ ] RGB timeout precedence

### Inspect Command
- [ ] Create `src/cli/inspect.rs`
- [ ] Implement `InspectArgs` struct
- [ ] Add section parsers (metadata, layers, categories, tap-dances, settings)
- [ ] Add JSON output for each section
- [ ] Add E2E tests:
  - [ ] Each section type
  - [ ] Invalid section → exit 1

### Keycode Utilities
- [ ] Create `src/cli/keycode.rs`
- [ ] Implement `keycode resolve` subcommand
- [ ] Add UUID→index resolution logic
- [ ] Add JSON output
- [ ] Add E2E tests:
  - [ ] LT/LM/MO/TG with UUIDs
  - [ ] Invalid UUID → exit 1
  - [ ] Non-parameterized keycode passthrough

### Documentation
- [ ] Add inline help text for all commands
- [ ] Update `--help` output
- [ ] Create initial `docs/TESTING.md`

---

## Phase 2: Tap Dance & Layer Utilities (Week 2)
**Priority: High**

### Tap Dance Commands
- [ ] Create `src/cli/tap_dance.rs`
- [ ] Implement `tap-dance list`:
  - [ ] Args struct
  - [ ] JSON output
  - [ ] E2E test
- [ ] Implement `tap-dance add`:
  - [ ] Args with name, single, double, hold
  - [ ] Validation (duplicate name, invalid keycodes)
  - [ ] File modification (preserve formatting)
  - [ ] E2E tests (2-way, 3-way)
- [ ] Implement `tap-dance delete`:
  - [ ] Args with name, force flag
  - [ ] Reference checking
  - [ ] E2E tests (unused, referenced, force)
- [ ] Implement `tap-dance validate`:
  - [ ] Orphan detection
  - [ ] Missing definition detection
  - [ ] JSON output
  - [ ] E2E tests

### Tap Dance E2E Flows
- [ ] Add→validate→generate flow test
- [ ] Orphan detection test
- [ ] Golden tests for generated tap dance code:
  - [ ] `tests/golden/keymap_tap_dance_2way.c`
  - [ ] `tests/golden/keymap_tap_dance_3way.c`
  - [ ] Mixed 2-way and 3-way
- [ ] Delete with references test
- [ ] Round-trip serialization test

### Layer Refs Command
- [ ] Create `src/cli/layer_refs.rs`
- [ ] Implement `layer-refs` command
- [ ] Add JSON output (inbound refs, warnings)
- [ ] Add E2E tests:
  - [ ] Detects inbound references
  - [ ] Reports transparency conflicts
  - [ ] Multiple refs to same position

### Documentation
- [ ] Update `docs/TESTING.md` with tap dance examples
- [ ] Add CLI reference for tap dance commands

---

## Phase 3: QMK Metadata & Categories (Week 3)
**Priority: Medium**

### QMK Metadata Commands
- [ ] Create `src/cli/qmk.rs`
- [ ] Implement `list-keyboards`:
  - [ ] Args with qmk-path, filter, json
  - [ ] Delegate to existing scan logic
  - [ ] Contract test (marked `#[ignore]`)
- [ ] Implement `list-layouts`:
  - [ ] Args with qmk-path, keyboard, json
  - [ ] Delegate to parser
  - [ ] Contract test (marked `#[ignore]`)
- [ ] Implement `geometry`:
  - [ ] Args with qmk-path, keyboard, layout-name, json
  - [ ] Output matrix/LED/visual mappings
  - [ ] Contract test for coordinate transforms

### Feature Gating
- [ ] Add runtime checks for QMK path validity
- [ ] Add clear error messages when QMK unavailable
- [ ] Document `#[ignore]` test pattern
- [ ] Add optional cargo feature `qmk` for tests

### Category Commands
- [ ] Create `src/cli/category.rs`
- [ ] Implement `category list`:
  - [ ] Args and JSON output
  - [ ] E2E test
- [ ] Implement `category add`:
  - [ ] Args with id, name, color
  - [ ] Validation (duplicate ID, invalid color)
  - [ ] E2E test
- [ ] Implement `category delete`:
  - [ ] Args with id, force
  - [ ] Reference checking (keys, layers)
  - [ ] E2E tests

### Category E2E Tests
- [ ] Add→assign to key→validate flow
- [ ] Delete in-use category without force → error
- [ ] Delete with force removes references
- [ ] Color priority rules (golden test)

### Documentation
- [ ] Update `docs/TESTING.md` with QMK gating info
- [ ] Document running ignored tests
- [ ] Add category CLI reference

---

## Phase 4: Templates, Config, Utilities (Week 4)
**Priority: Low**

### Template Commands
- [ ] Create `src/cli/template.rs`
- [ ] Implement `template list`:
  - [ ] Scan template directory
  - [ ] JSON output
  - [ ] E2E test
- [ ] Implement `template save`:
  - [ ] Args with layout, name, tags
  - [ ] Copy to template dir
  - [ ] E2E test
- [ ] Implement `template apply`:
  - [ ] Args with name, output file
  - [ ] Template loading
  - [ ] E2E round-trip test

### Config Commands
- [ ] Create `src/cli/config.rs`
- [ ] Implement `config show`:
  - [ ] JSON output
  - [ ] E2E test
- [ ] Implement `config set`:
  - [ ] Args for qmk-path, output-dir, theme
  - [ ] Validation
  - [ ] E2E test (set→show round-trip)

### Utility Commands
- [ ] Create `src/cli/keycodes.rs`
- [ ] Implement `keycodes` command:
  - [ ] List all keycodes
  - [ ] Category filter
  - [ ] JSON output
  - [ ] E2E test
- [ ] Create `src/cli/help.rs`
- [ ] Implement `help` command:
  - [ ] Load from help.toml (source of truth)
  - [ ] Topic listing
  - [ ] Specific topic display
  - [ ] E2E test

### Complete Documentation
- [ ] Finalize `docs/TESTING.md`:
  - [ ] Running tests section
  - [ ] Golden test workflow
  - [ ] Fixture usage guide
  - [ ] CI integration examples
- [ ] Create `docs/CLI_REFERENCE.md`:
  - [ ] All commands with examples
  - [ ] JSON schemas
  - [ ] Exit code reference
- [ ] Update main README with CLI section
- [ ] Add examples directory with sample scripts

### Final Integration
- [ ] Review all exit codes for consistency
- [ ] Ensure all commands have `--help`
- [ ] Validate JSON output schemas
- [ ] Run full test suite
- [ ] Performance check (target <2min for fast suite)

---

## Testing Checklist

### Coverage Goals
- [ ] 50+ E2E CLI tests
- [ ] 10+ golden tests for firmware generation
- [ ] All commands have JSON output tests
- [ ] All validation paths tested (success + error)
- [ ] All file operations use temp dirs

### Test Quality
- [ ] No hardcoded paths (use fixtures/temp dirs)
- [ ] Clear test names describing scenarios
- [ ] Assertions on exit codes, not just success()
- [ ] JSON parsing validates structure
- [ ] Golden tests use normalization

### Documentation Quality
- [ ] Every command has example in docs
- [ ] Error messages guide users to solutions
- [ ] Help text matches command behavior
- [ ] Testing guide includes troubleshooting

---

## Success Metrics

- [ ] `cargo test --tests` passes in <2 minutes
- [ ] `cargo test --features qmk -- --ignored` passes in <5 minutes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] All priority commands implemented with tests
- [ ] Documentation complete and reviewed
- [ ] Golden files reviewed for correctness
- [ ] CLI integrated into CI pipeline

---

## Notes

- Keep CLI code in `src/cli/` separate from TUI code
- All CLI commands delegate to existing services (no duplication)
- Use temp dirs for all file operations in tests
- Mark slow/QMK-dependent tests with `#[ignore]`
- Document `UPDATE_GOLDEN=1` workflow clearly
- Ensure deterministic mode works across platforms
