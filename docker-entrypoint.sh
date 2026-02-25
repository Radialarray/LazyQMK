#!/bin/bash
set -e

# Fix ownership of bind-mounted volumes so the lazyqmk user can read/write.
# This runs as root on container start, then drops privileges via gosu.
echo "[entrypoint] Fixing permissions for bind-mounted volumes..."
chown -R lazyqmk:lazyqmk /app/workspace /app/qmk_firmware 2>/dev/null || true
# Also ensure the config directory is owned correctly (named volume may already be fine)
chown -R lazyqmk:lazyqmk /home/lazyqmk 2>/dev/null || true

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
