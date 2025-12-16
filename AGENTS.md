# LazyQMK Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-12-12

## Active Technologies
- Rust 1.91.1+ + Ratatui 0.29 (TUI framework), Crossterm 0.29 (terminal backend), Serde 1.0 (serialization) (archived/021-dependency-updates)
- Human-readable files (Markdown layouts, TOML 0.9 configuration, JSON5 1.3 parsing, serde_yml 0.0.12 for YAML) (archived/021-dependency-updates)
- CLI: Clap 4.5, Config: dirs 6.0, Clipboard: arboard 3.6, UUID: 1.19 (archived/021-dependency-updates)

## Project Structure

```text
src/
tests/
```

## Commands

```bash
cargo test                                    # Run all tests
cargo clippy --all-features -- -D warnings   # Run clippy with zero tolerance for warnings
cargo build --release                        # Build release binary
```

## Code Style

Rust 1.75+: Follow standard conventions

## Recent Changes
- 023-idle-effect: Added idle effect screensaver for RGB lighting with configurable timeout (default 1 min), duration (default 5 min), and 9 selectable effects (default Breathing). Generates QMK C state machine for idle → effect → off transitions. Settings managed via TUI Settings Manager and stored per-layout in markdown files.
- archived/021-dependency-updates: Updated all dependencies to latest versions (json5 1.3, dirs 6.0, ratatui 0.29, crossterm 0.29, clap 4.5, serde_yml, toml 0.9, arboard 3.6, uuid 1.19), fixed 43 ratatui deprecation warnings, migrated from deprecated serde_yaml to serde_yml
- archived/020-robust-keyboard-picker: Added JSON5 parser support, robust QMK config discovery and merging
- archived/002-fix-startup-warnings: Added Rust 1.75+ (using Rust 1.88.0 per startup_errors.md) + Ratatui 0.26 (TUI framework), Crossterm 0.27 (terminal backend), Serde 1.0 (serialization)

<!-- MANUAL ADDITIONS START -->

## Development Workflow

### Testing Requirements
- **Always run tests before and after changes**: `cargo test`
- **All tests must pass locally before committing**: No exceptions
- **Run clippy before committing**: `cargo clippy --all-features -- -D warnings`
- **Clippy must pass with zero warnings**: No exceptions, fix all warnings
- **Never use allow or ignore flags**: Fix the underlying issue instead
- **Ensure all tests pass**: Target 100% pass rate
- **Run tests after refactoring**: Verify no regressions
- **Integration tests**: Tests requiring external dependencies (QMK CLI, etc.) should be marked with `#[ignore]`
- **Note**: CI may use newer Rust/clippy versions with stricter lints. If CI fails but local passes, update Rust (`rustup update stable`) and re-run clippy
- **Required Rust version**: 1.91.1 or newer (matches CI to ensure consistent clippy behavior)

### Help System Source of Truth
- **Context help and help menu text must come from `src/data/help.toml`**. Do not hardcode help strings in code; add or update entries in `help.toml` instead.

### Code Quality Standards

#### Privacy & Personal Information
- **NEVER include personal information** in code, comments, or commits:
  - ❌ Real names (use generic placeholders like "user", "developer", "author")
  - ❌ Email addresses (use example.com domains or GitHub no-reply emails)
  - ❌ Personal file paths (use `/home/user/`, `/Users/user/`, `C:\Users\user\`)
  - ❌ Private URLs or server addresses
  - ❌ API keys, tokens, or credentials
- **Use generic examples** in documentation:
  - ✅ `my_custom_keymap` instead of personal keymap names
  - ✅ `user@example.com` instead of real email addresses
  - ✅ `/Users/user/dev/LazyQMK/` instead of personal paths
- **Git author identity**: Always use your GitHub username and no-reply email
  - Configure: `git config user.name "YourGitHubUsername"`
  - Configure: `git config user.email "123456+username@users.noreply.github.com"`

#### Dead Code Management
- Remove truly unused code (functions, structs, enums that are never referenced)
- Keep `#[allow(dead_code)]` annotations when:
  - Compiler has false positives (public API that's unused internally)
  - Code is part of a trait implementation
  - Code is intentionally kept for future use
- Document reasoning when keeping dead code

#### Refactoring Best Practices
1. **Analyze first**: Use `grep`/`rg` to understand code usage before removing
2. **Start with safe removals**: Remove obviously unused code first
3. **Test incrementally**: Run tests after each major removal phase
4. **Avoid high-risk changes**: Skip complex state migrations unless necessary
5. **Document decisions**: Explain why code was kept or removed

#### Component Trait Pattern (Spec 017)
- All TUI components should implement either `Component` or `ContextualComponent<T>` traits
- Components encapsulate: state, rendering logic, and input handling
- Use event-driven communication between components and AppState
- Avoid exposing internal state via `state()`/`state_mut()` methods
- Legacy patterns from pre-017 should be removed during cleanup

### Git Workflow

#### Commit Message Format
Follow Conventional Commits specification:
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code restructuring (no behavior change)
- `chore`: Maintenance tasks (dependencies, cleanup)
- `docs`: Documentation changes
- `test`: Test additions or modifications
- `perf`: Performance improvements
- `style`: Code style changes (formatting, missing semicolons)
- `build`: Build system changes
- `ci`: CI/CD configuration changes

**Examples:**
```
feat(keycode-picker): add language keycodes with selector and persistence
fix(ui): apply theme background color globally
refactor(tui): remove legacy code from component trait migration
chore: bump version to 0.6.0
```

**Guidelines:**
- Use present tense, lowercase
- Keep description under 72 characters
- Add body for context when needed
- Focus on "why" rather than "what"
- Be specific, avoid generic messages like "Update files" or "Fix bug"

#### Branch Naming
- `feat/feature-name` - New features
- `fix/bug-description` - Bug fixes
- `refactor/refactor-name` - Code restructuring
- `chore/task-name` - Maintenance tasks
- `docs/doc-name` - Documentation updates

#### Release Process

**IMPORTANT:** Releases are automated via GitHub Actions. **Do NOT manually create CHANGELOG.md files.**

**How Releases Work:**
1. **Version Bump**: Update version in `Cargo.toml` (e.g., `0.10.0` → `0.11.0`)
2. **Commit Version Bump**: Create commit with message `chore: bump version to X.Y.Z`
3. **Create Git Tag**: Create annotated tag with `git tag -a vX.Y.Z -m "Release vX.Y.Z: Brief Description"`
4. **Push Tag**: Push tag to GitHub with `git push origin vX.Y.Z`
5. **GitHub Actions Handles Everything**:
   - Runs tests on all platforms (Linux, macOS, Windows)
   - Runs clippy on all platforms
   - Builds release binaries for all targets (x86_64 & ARM64)
   - Generates changelog automatically from git commits between tags
   - Creates GitHub release with binaries and changelog
   - Calculates and includes SHA256 checksums

**Changelog Generation:**
- Changelogs are auto-generated from git commit messages using conventional commits format
- The workflow script (`.github/workflows/release.yml`) parses commit types and formats them:
  - `feat:` → ✨ **Feature**
  - `fix:` → 🐛 **Fix**
  - `refactor:` → ♻️ **Refactor**
  - `chore:` → 🔨 **Chore**
  - `docs:` → 📚 **Docs**
  - `test:` → 🧪 **Test**
  - `ci:` → 🔧 **CI**
  - `perf:` → ⚡ **Performance**
  - `style:` → 💄 **Style**
  - `build:` → 📦 **Build**
- Changelog includes all commits since the previous tag
- Download links and checksums are automatically added

**What NOT To Do:**
- ❌ Don't create or edit `CHANGELOG.md` files manually
- ❌ Don't manually create GitHub releases
- ❌ Don't manually build release binaries
- ❌ Don't skip the version bump commit
- ❌ Don't push tags before committing version changes

**Example Release Flow:**
```bash
# 1. Bump version
sed -i '' 's/version = "0.10.0"/version = "0.11.0"/' Cargo.toml

# 2. Commit version bump
git add Cargo.toml
git commit -m "chore: bump version to 0.11.0"

# 3. Create and push tag
git tag -a v0.11.0 -m "Release v0.11.0: Tap Dance Support"
git push origin main
git push origin v0.11.0

# 4. GitHub Actions will automatically:
#    - Run tests and clippy
#    - Build binaries for all platforms
#    - Generate changelog from commits
#    - Create GitHub release with all assets
```

#### Review Process
- Always validate changes with tests before committing
- Check `git diff` to review all modifications
- Ensure no unintended changes are included
- Exclude unrelated files (submodule changes, temp files, etc.)
- **qmk_firmware submodule:** never delete or replace it. Do not stage/commit its changes unless you are intentionally changing the submodule. If it shows modified or missing, restore with `git submodule update --init --recursive qmk_firmware` and keep it out of commits.

### Architecture Guidelines

#### TUI Component Structure
- **State Management**: Use centralized `AppState` as single source of truth
- **Component Lifecycle**: Initialize → Handle Input → Render
- **Event Communication**: Components emit events, handlers update state
- **Rendering**: Immediate mode - rebuild UI every frame from state

#### Coordinate Systems
The project uses three coordinate systems:
1. **Matrix Coordinates**: Electrical wiring (rows/columns)
2. **LED Index**: Sequential RGB LED numbering
3. **Visual Position**: User's mental model (what appears in UI)

Always use `VisualLayoutMapping` for transformations between systems.

#### File Formats
- **Layouts**: Markdown with YAML frontmatter
- **Configuration**: TOML
- **Keycode Database**: JSON
- **QMK Metadata**: JSON (parsed from `info.json`)

### Performance Considerations
- Target 60fps (16ms/frame), typical event-driven rendering
- Pre-allocate fixed-size vectors where possible
- Use buffered I/O (`BufReader`, `BufWriter`)
- Spawn threads for long operations (firmware compilation)
- Use message passing (`mpsc` channels) for thread communication

### Common Patterns

#### Error Handling
```rust
use anyhow::{Context, Result};

fn operation() -> Result<T> {
    something().context("Failed to do something")?
}
```

#### Component Implementation
```rust
impl Component for MyComponent {
    type Event = MyEvent;
    
    fn handle_input(&mut self, key: KeyEvent) -> Option<Self::Event> {
        // Handle input, return event if action needed
    }
    
    fn render(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        // Render component to frame
    }
}
```

#### State Updates
```rust
// Always set dirty flag when modifying layout data
state.dirty = true;
state.layout.layers[layer_idx].keys[idx].keycode = new_keycode;
```

### Code Review Checklist

Before committing changes:
- [ ] All tests pass (`cargo test`)
- [ ] No compiler warnings (`cargo clippy --all-features -- -D warnings`)
- [ ] Clippy passes with zero warnings (never use allow/ignore flags)
- [ ] Code follows project patterns (Component trait, MVC)
- [ ] No dead code (or justified with comments)
- [ ] Error handling uses `anyhow` with context
- [ ] Public APIs are documented
- [ ] Complex logic has inline comments
- [ ] Commit message follows convention
- [ ] No unrelated changes included
- [ ] No personal information in code, comments, or commit messages
- [ ] git diff reviewed for accuracy

### Troubleshooting

#### Common Issues
1. **Tests failing after refactor**: Check for removed methods still in use
2. **Clippy warnings**: Fix the underlying issue, never use allow/ignore flags
3. **Coordinate system confusion**: Always use `VisualLayoutMapping` methods
4. **Theme not applying**: Ensure all blocks use `bg(theme.background)`
5. **Component not rendering**: Check `ActiveComponent` enum and popup routing

#### Debug Techniques
- Use `cargo test -- --nocapture` to see println! output
- Add `eprintln!` for debugging (goes to stderr, not captured by tests)
- Use `cargo check` for faster iteration than `cargo build`
- Use `cargo clippy -- -W clippy::pedantic` for strict linting

### Resources
- [ARCHITECTURE.md](docs/ARCHITECTURE.md) - Complete technical architecture
- [FEATURES.md](docs/FEATURES.md) - All implemented features
- [QUICKSTART.md](QUICKSTART.md) - User guide
- [specs/archived/](specs/archived/) - Historical specification documents

<!-- MANUAL ADDITIONS END -->
