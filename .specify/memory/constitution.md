<!--
Sync Impact Report:
- Version: 0.0.0 → 1.0.0 (MAJOR: Initial constitution ratified)
- Modified Principles: N/A (initial version)
- Added Sections: All sections (initial creation)
- Removed Sections: None
- Templates Status:
  ✅ plan-template.md - Reviewed, aligns with constitution principles
  ✅ spec-template.md - Reviewed, aligns with constitution principles
  ✅ tasks-template.md - Reviewed, aligns with constitution principles
- Follow-up TODOs: None
-->

# Keymapper TUI Constitution

## Core Principles

### I. User-Centered Design

Every feature MUST prioritize user experience and accessibility. The TUI MUST provide:
- Clear, intuitive keyboard-driven navigation (VIM-inspired shortcuts)
- Comprehensive help documentation accessible via `?` key
- Visual feedback for all state changes
- Error messages that guide users toward resolution
- Onboarding experience for first-time users

**Rationale**: Terminal interfaces lack GUI affordances; explicit guidance and keyboard-first design ensure users can discover and use features efficiently without external documentation.

### II. Human-Readable Persistence

All persistent data MUST be stored in human-readable, version-control-friendly formats:
- Keyboard layouts stored as Markdown files with structured tables
- Configuration stored in TOML format
- Metadata embedded in YAML frontmatter or Markdown comments
- Generated firmware source code readable and editable

**Rationale**: Human-readable formats enable manual editing, version control, debugging, and portability. Users can understand and fix their data without specialized tools.

### III. Modular Architecture

Code MUST be organized into clear, single-responsibility modules:
- Data models separated from business logic
- Parsers independent of UI components
- UI widgets stateless and reusable
- Coordinate transformations isolated in dedicated modules
- File I/O operations abstracted from core logic

**Rationale**: Modularity enables independent testing, parallel development, maintenance, and future extensions without cascading changes.

### IV. State Management Discipline

Application state MUST follow centralized state management patterns:
- Single `AppState` object as source of truth
- All UI components read from state immutably
- State mutations explicit and traceable
- Dirty flag tracking for unsaved changes
- No hidden state in UI components

**Rationale**: Centralized state prevents synchronization bugs, simplifies debugging, makes data flow predictable, and ensures consistency across UI updates.

### V. Testing Strategy

Testing MUST be comprehensive and appropriate to component type:
- Unit tests for parsers, data models, and pure functions
- Integration tests for file I/O and QMK firmware generation
- End-to-end tests for critical user workflows
- Test coverage for edge cases and error conditions
- Validation of keyboard geometry transformations

**Rationale**: The application handles complex coordinate transformations, user data, and firmware generation. Bugs in these areas can lead to data loss or bricked keyboards. Testing prevents regressions and validates correctness.

### VI. Performance Awareness

The application MUST remain responsive under all conditions:
- Target 60fps rendering (16ms per frame max)
- Event-driven rendering (only on state changes)
- Background threads for long-running operations (firmware compilation)
- Lazy loading and caching for expensive operations
- Viewport culling for rendering optimizations

**Rationale**: Terminal rendering is inherently slower than GPU-accelerated GUIs. Performance discipline ensures the TUI remains usable even on resource-constrained systems.

### VII. Firmware Integration Safety

Firmware generation and flashing MUST prioritize safety:
- Validation of all layouts before firmware generation
- Clear error messages for invalid keycodes or matrix configurations
- Build logs captured and displayed with scrollback
- Explicit user confirmation before flashing
- Recovery instructions provided for failed flashes

**Rationale**: Incorrectly generated firmware can render keyboards unusable. Safety checks and clear guidance minimize risk of user error.

## Monorepo Structure

The project follows a monorepo structure containing:

1. **Main TUI Application** (repository root):
   - Rust codebase using Ratatui and Crossterm
   - Source code in `src/` with clear module separation
   - Tests in `tests/` covering unit, integration, and contract levels
   - Documentation in root-level Markdown files

2. **QMK Firmware Submodule** (`vial-qmk-keebart/`):
   - Tracked as a Git submodule
   - Contains keyboard firmware definitions
   - Used for geometry parsing and firmware compilation
   - MUST NOT be modified by TUI application (read-only)

3. **Specification System** (`.specify/`):
   - Contains project constitution, templates, and scripts
   - Maintains feature specifications and implementation plans
   - Enforces governance and consistency

### Submodule Management Rules

- The `vial-qmk-keebart` submodule MUST be pinned to specific commits
- Updates to the submodule MUST be deliberate and documented
- The TUI application MUST treat the submodule as read-only
- Changes to keyboard definitions MUST be upstreamed to the submodule's origin
- Build artifacts from the submodule MUST NOT be committed

## Development Workflow

### Feature Development Process

1. **Specification First**: New features MUST begin with a specification document following `.specify/templates/spec-template.md`
2. **Implementation Planning**: Create implementation plan using `.specify/templates/plan-template.md`
3. **Task Breakdown**: Generate tasks using `.specify/templates/tasks-template.md`
4. **Test-Driven Development**: Write tests first for critical paths, ensure they fail, then implement
5. **Constitution Compliance**: Verify adherence to principles before merging
6. **Documentation**: Update architecture guide and quickstart as needed

### Code Review Requirements

All code changes MUST be reviewed for:
- Adherence to state management patterns
- Module boundary preservation
- Human-readable file format compatibility
- Error handling completeness
- Performance implications
- User experience impact

### Quality Gates

Before merging, verify:
- [ ] All tests pass
- [ ] No regressions in existing functionality
- [ ] Documentation updated (if applicable)
- [ ] Dirty flag handling correct
- [ ] Error messages clear and actionable
- [ ] Performance benchmarks within targets

## Governance

### Amendment Procedure

1. Propose amendment with rationale and impact analysis
2. Document affected principles or sections
3. Update Sync Impact Report at top of constitution
4. Increment version according to semantic versioning
5. Propagate changes to dependent templates and documentation

### Versioning Policy

Constitution version follows MAJOR.MINOR.PATCH:
- **MAJOR**: Backward incompatible governance changes, principle removals, or redefinitions
- **MINOR**: New principles added or materially expanded guidance
- **PATCH**: Clarifications, wording improvements, non-semantic refinements

### Complexity Justification

Any violation of these principles MUST be explicitly justified:
- Document in feature specification under "Complexity Tracking"
- Explain why the principle cannot be followed
- Describe simpler alternatives considered and rejected
- Obtain explicit approval before proceeding

### Constitution Authority

This constitution supersedes all other development practices. In case of conflict, the constitution takes precedence unless explicitly amended through the amendment procedure.

### Runtime Development Guidance

For detailed implementation guidance, refer to:
- `TUI_ARCHITECTURE_GUIDE.md` for technical architecture decisions
- `QUICKSTART.md` for getting started with development
- `.specify/templates/` for feature development workflows

**Version**: 1.0.0 | **Ratified**: 2024-11-24 | **Last Amended**: 2024-11-24
