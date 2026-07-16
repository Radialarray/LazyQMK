# Building Firmware

## Prerequisites

- QMK CLI: `pip3 install qmk`
- ARM GCC toolchain (for RP2040 based boards)
- LazyQMK repo with qmk_firmware submodule initialized:
  ```bash
  git submodule update --init --recursive
  ```

## PaletteFX Module

PaletteFX is a community module required by some layouts:

```bash
cd qmk_firmware
git clone https://github.com/getreuer/qmk-modules.git modules/getreuer
cd ..
```

## Generate & Compile

```bash
# Generate keymap files from layout
cargo run -- generate \
  --layout ~/Library/Application\ Support/LazyQMK/layouts/<your_layout>.json \
  --qmk-path qmk_firmware \
  --out-dir ~/Library/Application\ Support/LazyQMK/builds

# Copy generated files to QMK keymap directory
cp ~/Library/Application\ Support/LazyQMK/builds/* \
  qmk_firmware/keyboards/<keyboard>/keymaps/<keymap_name>/

# Compile firmware
cd qmk_firmware
qmk compile -kb <keyboard> -km <keymap_name>
```

Output will be at `qmk_firmware/.build/<keyboard>_<keymap>.uf2`.

## Quick Flash

Enter bootloader mode on your keyboard, then:

```bash
cp qmk_firmware/.build/<keyboard>_<keymap>.uf2 /Volumes/RPI-RP2/
```
