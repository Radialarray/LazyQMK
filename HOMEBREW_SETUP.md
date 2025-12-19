# Homebrew Setup Instructions

## ✅ Changes Made to LazyQMK Repository

1. **Updated `.github/workflows/release.yml`** - Added `update-homebrew` job that automatically updates your tap
2. **Updated `README.md`** - Added Homebrew installation instructions

## 🔧 Setup Steps

### Step 1: Create Initial Formula in Your Tap Repository

In your `Radialarray/homebrew-lazyqmk` repository, create the following file:

**File:** `Formula/lazyqmk.rb`

```ruby
class Lazyqmk < Formula
  desc "Interactive terminal workspace for QMK firmware"
  homepage "https://github.com/Radialarray/LazyQMK"
  version "0.12.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/Radialarray/LazyQMK/releases/download/v#{version}/lazyqmk-macos-aarch64.zip"
      sha256 "d7bab8ba24155dcdb4c7a3fd4bdc8885cc52fdabe642a2b3f0a745175eca248f"
    end
  end

  on_linux do
    if Hardware::CPU.arm? && Hardware::CPU.is_64_bit?
      url "https://github.com/Radialarray/LazyQMK/releases/download/v#{version}/lazyqmk-linux-aarch64.zip"
      sha256 "e66222e7f566a72fa637531757edc19a46e2e0d92d367d64e2ebf4be74235cac"
    else
      url "https://github.com/Radialarray/LazyQMK/releases/download/v#{version}/lazyqmk-linux-x86_64.zip"
      sha256 "0fa31c7486f46e83f2d33687d06eca83e904b6c5a7dd2f3d4a98ad7837d19daa"
    end
  end

  def install
    bin.install "lazyqmk"
  end

  test do
    system "#{bin}/lazyqmk", "--version"
  end
end
```

### Step 2: Add README to Your Tap Repository

**File:** `README.md`

```markdown
# Homebrew LazyQMK

Homebrew tap for [LazyQMK](https://github.com/Radialarray/LazyQMK) - Interactive terminal workspace for QMK firmware.

## Installation

```bash
brew install Radialarray/lazyqmk/lazyqmk
```

Or tap first, then install:

```bash
brew tap Radialarray/lazyqmk
brew install lazyqmk
```

## Usage

```bash
# Initialize LazyQMK in your project
lazyqmk

# Get help
lazyqmk --help
```

## About

LazyQMK is a terminal-based keyboard layout editor for QMK firmware. Design keymaps, manage layers, organize with colors and categories, and compile firmware—all without leaving your terminal.

For more information, visit the [main repository](https://github.com/Radialarray/LazyQMK).

## Updating

The formula is automatically updated when new versions are released.

To update to the latest version:

```bash
brew update
brew upgrade lazyqmk
```

## License

MIT License - see the [main repository](https://github.com/Radialarray/LazyQMK) for details.
```

### Step 3: Create Personal Access Token

1. Go to: https://github.com/settings/tokens/new
2. **Token name:** `HOMEBREW_TAP_TOKEN`
3. **Expiration:** Choose your preference (90 days, 1 year, or no expiration)
4. **Scopes:** Check `repo` (full control of private repositories)
5. Click **"Generate token"**
6. **Copy the token** (you won't see it again!)

### Step 4: Add Token to LazyQMK Repository Secrets

1. Go to: https://github.com/Radialarray/LazyQMK/settings/secrets/actions
2. Click **"New repository secret"**
3. **Name:** `HOMEBREW_TAP_TOKEN`
4. **Value:** Paste your token from Step 3
5. Click **"Add secret"**

### Step 5: Commit and Push Your Changes

In the LazyQMK repository:

```bash
# Review changes
git status
git diff

# Commit the workflow and README changes
git add .github/workflows/release.yml README.md
git commit -m "feat(ci): add Homebrew tap support with automated formula updates"

# Push to GitHub
git push origin main
```

### Step 6: Push Initial Formula to Tap Repository

In your `homebrew-lazyqmk` repository:

```bash
# Create the Formula directory and file
mkdir -p Formula
# Copy the lazyqmk.rb content from above

# Add README
# Copy the README.md content from above

# Commit and push
git add Formula/lazyqmk.rb README.md
git commit -m "feat: add lazyqmk formula v0.12.0"
git push origin main
```

## ✅ Testing

### Test Installation Locally

```bash
# Install from your tap
brew install Radialarray/lazyqmk/lazyqmk

# Verify installation
lazyqmk --version

# Should output: lazyqmk 0.12.0 or similar
```

### Test Automatic Updates (Next Release)

When you create your next release (e.g., v0.13.0):

1. Update version in `Cargo.toml`
2. Commit: `git commit -m "chore: bump version to 0.13.0"`
3. Tag: `git tag -a v0.13.0 -m "Release v0.13.0"`
4. Push: `git push origin main && git push origin v0.13.0`
5. GitHub Actions will:
   - Build binaries
   - Create release
   - **Automatically update the formula in homebrew-lazyqmk**
6. Users can update: `brew update && brew upgrade lazyqmk`

## 🎉 Success!

You now have:
- ✅ Automated Homebrew tap updates on every release
- ✅ Users can install with `brew install Radialarray/lazyqmk/lazyqmk`
- ✅ No manual formula maintenance required
- ✅ Professional distribution setup

## 📚 User Installation Instructions

Share these with your users:

### macOS/Linux Installation

```bash
brew install Radialarray/lazyqmk/lazyqmk
```

### Updating

```bash
brew update
brew upgrade lazyqmk
```

### Uninstalling

```bash
brew uninstall lazyqmk
brew untap Radialarray/lazyqmk
```

## 🔍 Troubleshooting

### Formula Not Found

If users get "formula not found":

```bash
brew update
brew tap Radialarray/lazyqmk
brew install lazyqmk
```

### Checksum Mismatch

If you see checksum errors:
1. Ensure the release assets were fully uploaded
2. Wait a few minutes and try again (CDN caching)
3. Check that checksums in formula match release assets

### Automated Update Failed

Check GitHub Actions logs in LazyQMK repository:
1. Go to: https://github.com/Radialarray/LazyQMK/actions
2. Click on the failed release workflow
3. Check the `update-homebrew` job logs

Common issues:
- `HOMEBREW_TAP_TOKEN` not set or expired
- Tap repository URL incorrect
- Network issues downloading checksums

## 🎓 What's Next?

Your Homebrew distribution is now fully automated! Future releases will:
1. Automatically update the formula
2. Users get updates via `brew upgrade`
3. No manual intervention needed

Consider:
- Announcing Homebrew support in your README and release notes
- Adding brew installation instructions to your documentation
- Monitoring formula usage via GitHub repo insights
