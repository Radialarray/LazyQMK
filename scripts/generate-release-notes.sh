#!/usr/bin/env bash
set -euo pipefail

# Inputs:
#   TAG (required)             e.g., v0.12.2
#   REPO (required)            e.g., Radialarray/LazyQMK
#   PROJECT_LABEL (optional)   e.g., LazyQMK
#   ARTIFACT_DIR (optional)    where artifacts were downloaded (default: ./artifacts)
#
# Output:
#   Writes release-notes.md to cwd based on current git history and available artifacts.
#
# Sections:
#   ## Changes Since <prev>
#   ## Install <project> <tag>
#   ## Downloads
#     - Archives table
#     - Installer Scripts list
#     - Checksums block
#
# Notes:
#   - We derive PREV_TAG from git tags (excluding TAG), newest first.
#   - We parse artifacts from ARTIFACT_DIR; filenames drive the downloads table.
#   - We include archives + installer scripts + checksums.
#   - We keep cargo-dist naming (no hardcoded old LazyQMK names).

TAG=${TAG:?TAG is required}
REPO=${REPO:?REPO is required}
PROJECT_LABEL=${PROJECT_LABEL:-${REPO##*/}}
ARTIFACT_DIR=${ARTIFACT_DIR:-artifacts}

# Find previous tag
PREV_TAG=$(git tag --sort=-v:refname | grep -v "^${TAG}$" | head -n 1 || true)

# Helper to map target-ish strings to human names
map_target() {
  case "$1" in
    *x86_64-unknown-linux-gnu*) echo "Linux | x86_64" ;;
    *aarch64-unknown-linux-gnu*) echo "Linux | ARM64" ;;
    *aarch64-apple-darwin*) echo "macOS | Apple Silicon (ARM64)" ;;
    *x86_64-apple-darwin*) echo "macOS | Intel" ;;
    *x86_64-pc-windows-msvc*) echo "Windows | x86_64" ;;
    *) echo "Unknown | Unknown" ;;
  esac
}

# Collect artifacts
ARCHIVE_ROWS=()
INSTALLERS=()
CHECKSUMS_FILE=$(mktemp)
: > "$CHECKSUMS_FILE"

if [ -d "$ARTIFACT_DIR" ]; then
  while IFS= read -r -d '' f; do
    base=$(basename "$f")
    url="https://github.com/${REPO}/releases/download/${TAG}/${base}"
    case "$base" in
      *.tar.*|*.zip)
        human=$(map_target "$base")
        IFS='|' read -r platform arch <<<"$human"
        ARCHIVE_ROWS+=("| ${platform// /} | ${arch// /} | [${base}](${url}) |")
        ;;
      *installer.sh|*installer.ps1|*.rb)
        INSTALLERS+=("- [${base}](${url})")
        ;;
      *.sha256|*.sha512|*.sha3-256|*.sha3-512|*.blake2*)
        echo "$(cat "$f")" >> "$CHECKSUMS_FILE"
        ;;
      *)
        ;; # ignore other files
    esac
  done < <(find "$ARTIFACT_DIR" -type f -print0)
fi

# Sort archives for stable output
IFS=$'\n' ARCHIVE_ROWS=($(printf '%s\n' "${ARCHIVE_ROWS[@]}" | sort))
IFS=$'\n' INSTALLERS=($(printf '%s\n' "${INSTALLERS[@]}" | sort))

{
  # Changes header
  if [ -n "$PREV_TAG" ]; then
    echo "## Changes Since ${PREV_TAG}"
    echo
    git log ${PREV_TAG}..HEAD --pretty=format:"- %s" | while IFS= read -r line; do
      echo "$line" | sed -E \
        -e 's/^- feat(\([^)]+\))?: /- ✨ **Feature**: /' \
        -e 's/^- fix(\([^)]+\))?: /- 🐛 **Fix**: /' \
        -e 's/^- ci(\([^)]+\))?: /- 🔧 **CI**: /' \
        -e 's/^- docs(\([^)]+\))?: /- 📚 **Docs**: /' \
        -e 's/^- refactor(\([^)]+\))?: /- ♻️  **Refactor**: /' \
        -e 's/^- test(\([^)]+\))?: /- 🧪 **Test**: /' \
        -e 's/^- chore(\([^)]+\))?: /- 🔨 **Chore**: /' \
        -e 's/^- perf(\([^)]+\))?: /- ⚡ **Performance**: /' \
        -e 's/^- style(\([^)]+\))?: /- 💄 **Style**: /' \
        -e 's/^- build(\([^)]+\))?: /- 📦 **Build**: /'
    done
  else
    echo "## Changes"
    echo
    git log --pretty=format:"- %s" | while IFS= read -r line; do
      echo "$line" | sed -E \
        -e 's/^- feat(\([^)]+\))?: /- ✨ **Feature**: /' \
        -e 's/^- fix(\([^)]+\))?: /- 🐛 **Fix**: /' \
        -e 's/^- ci(\([^)]+\))?: /- 🔧 **CI**: /' \
        -e 's/^- docs(\([^)]+\))?: /- 📚 **Docs**: /' \
        -e 's/^- refactor(\([^)]+\))?: /- ♻️  **Refactor**: /' \
        -e 's/^- test(\([^)]+\))?: /- 🧪 **Test**: /' \
        -e 's/^- chore(\([^)]+\))?: /- 🔨 **Chore**: /' \
        -e 's/^- perf(\([^)]+\))?: /- ⚡ **Performance**: /' \
        -e 's/^- style(\([^)]+\))?: /- 💄 **Style**: /' \
        -e 's/^- build(\([^)]+\))?: /- 📦 **Build**: /'
    done
  fi

  echo
  echo "## Install ${PROJECT_LABEL} ${TAG#v}"
  echo
  echo "### Install prebuilt binaries via shell script"
  echo '```sh'
  echo "curl --proto '=https' --tlsv1.2 -LsSf https://github.com/${REPO}/releases/latest/download/${PROJECT_LABEL,,}-installer.sh | sh"
  echo '```'
  echo
  echo "### Install prebuilt binaries via Homebrew"
  echo '```sh'
  echo "brew install radialarray/tap/${PROJECT_LABEL,,}"
  echo '```'

  echo
  echo "## Downloads"

  echo "### Archives"
  echo "| Platform | Architecture | Download |"
  echo "| --- | --- | --- |"
  if [ ${#ARCHIVE_ROWS[@]} -eq 0 ]; then
    echo "| (none) | (none) | (none) |"
  else
    printf '%s\n' "${ARCHIVE_ROWS[@]}"
  fi

  echo
  echo "### Installer Scripts"
  if [ ${#INSTALLERS[@]} -eq 0 ]; then
    echo "(none)"
  else
    printf '%s\n' "${INSTALLERS[@]}"
  fi

  echo
  echo "### Checksums"
  echo '```'
  if [ -s "$CHECKSUMS_FILE" ]; then
    cat "$CHECKSUMS_FILE"
  fi
  echo '```'
} > release-notes.md

rm -f "$CHECKSUMS_FILE"
