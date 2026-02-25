#!/bin/bash
set -e

# Fix ownership of bind-mounted volumes so the lazyqmk user can read/write.
# This runs as root on container start, then drops privileges via gosu.
echo "[entrypoint] Fixing permissions for bind-mounted volumes..."
chown -R lazyqmk:lazyqmk /app/workspace /app/qmk_firmware 2>/dev/null || true
# Also ensure the config directory is owned correctly (named volume may already be fine)
chown -R lazyqmk:lazyqmk /home/lazyqmk 2>/dev/null || true

# Fix git submodule reference: if .git is a file (submodule pointer), it references
# a path outside the container (../.git/modules/qmk_firmware) that doesn't exist.
# Initialize a minimal standalone git repo so `git describe` works during QMK compilation.
# Also mark as safe.directory for both root and lazyqmk to avoid "dubious ownership" errors.
git config --global --add safe.directory /app/qmk_firmware 2>/dev/null || true
su -s /bin/bash lazyqmk -c "git config --global --add safe.directory /app/qmk_firmware" 2>/dev/null || true

# Find ALL .git files (submodule pointers) and convert to standalone repos
git_files=$(find /app/qmk_firmware -name .git -type f 2>/dev/null)
if [ -n "$git_files" ]; then
    echo "[entrypoint] Found git submodule references, converting to standalone repos..."
    echo "$git_files" | while read -r git_file; do
        dir=$(dirname "$git_file")
        echo "[entrypoint]   Converting: $dir"
        rm "$git_file"
        cd "$dir"
        git init --quiet
        git -c user.email="docker@lazyqmk" -c user.name="LazyQMK Docker" commit --allow-empty -m "Submodule snapshot" --quiet
        git tag "0.0.0" 2>/dev/null || true
    done
    cd /app
    echo "[entrypoint] All git submodule references converted."
fi

# Mark all git directories as safe for both root and lazyqmk
find /app/qmk_firmware -name .git -type d 2>/dev/null | while read -r git_dir; do
    repo_dir=$(dirname "$git_dir")
    git config --global --add safe.directory "$repo_dir" 2>/dev/null || true
    su -s /bin/bash lazyqmk -c "git config --global --add safe.directory \"$repo_dir\"" 2>/dev/null || true
done

# Set up QMK firmware directory if it exists and hasn't been set up yet
# Note: Delete .qmk_setup_done manually if you update the QMK submodule to force reinstallation
if [ -d "/app/qmk_firmware/keyboards" ] && [ ! -f "/app/qmk_firmware/.qmk_setup_done" ]; then
    echo "[entrypoint] Installing QMK Python dependencies..."
    if [ -f "/app/qmk_firmware/requirements.txt" ]; then
        if pip3 install --break-system-packages -r /app/qmk_firmware/requirements.txt; then
            touch /app/qmk_firmware/.qmk_setup_done
            echo "[entrypoint] QMK setup complete."
        else
            echo "[entrypoint] WARNING: QMK Python dependency installation failed!"
            echo "[entrypoint] QMK compilation may not work correctly."
            # Don't create marker file — allow retry on next container start
        fi
    else
        echo "[entrypoint] WARNING: /app/qmk_firmware/requirements.txt not found"
    fi
fi

# Drop to lazyqmk user and exec the main binary
echo "[entrypoint] Starting LazyQMK as user lazyqmk (uid $(id -u lazyqmk))..."
exec gosu lazyqmk lazyqmk "$@"
