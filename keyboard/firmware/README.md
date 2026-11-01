# Keyboard firmware (K step 5) — QMK, built out of tree

QMK firmware for the split keyboard: layout B, 42 keys, Corne's `LAYOUT_split_3x6_3` matrix (6 × 4 COL2ROW per
half), a Waveshare RP2040-Zero per half. Pins and the split wiring come from
[`../docs/pcb-spec.md`](../docs/pcb-spec.md) §2 and §4 (sync **KS2**). One `.uf2` serves both halves.

## Files

- `fw16kb/keyboard.json` — the data-driven keyboard definition: matrix pins for both halves, half-duplex vendor
  (PIO) serial on GP11, `SPLIT_HAND_PIN` GP0, bootmagic keys, the layout.
- `fw16kb/keymaps/default/keymap.c` — the default keymap.
- `fw16kb/readme.md` — the readme QMK's lint requires in every keyboard directory.
- `build.sh` — the out-of-tree build.

## Building

From the repo root, in the dev shell (`qmk` and `arm-none-eabi-gcc` come from `flake.nix`):

```
bash keyboard/firmware/build.sh            # or: bash keyboard/firmware/build.sh <keymap>
```

- The first run clones `qmk_firmware` at tag **0.34.7** (shallow, with submodules, ≈ 1.5 GB) into
  `_build/qmk_firmware/`, which is git-ignored. Bump the tag in `build.sh` (`QMK_REF`).
- QMK only finds keyboards under its own `keyboards/` tree, and its external userspace holds keymaps but not keyboard
  definitions. So the script symlinks `_build/qmk_firmware/keyboards/fw16kb` → `keyboard/firmware/fw16kb`. Edits
  here are picked up on the next build; nothing in this repo is a symlink.
- The script sets `QMK_HOME` itself, so `qmk setup` and `~/.config/qmk` are never needed.
- Output: `_build/keyboard/firmware/fw16kb_default.uf2` (also left in `_build/qmk_firmware/` by QMK).

Verified 2026-11-01 with qmk CLI 1.2.0 and Arm GNU Toolchain 15.3.Rel1 from the flake:
- `fw16kb_default.uf2` is 71 168 bytes. `arm-none-eabi-size`: text 35 564, data 0, bss 265 528.
- `qmk lint -kb fw16kb -km default` passes (run inside `_build/qmk_firmware` with `QMK_HOME` set).

## Split behaviour (what the hardware relies on)

- **Master:** QMK's default on ChibiOS without a `USB_VBUS_PIN` is `SPLIT_USB_DETECT`: the half that enumerates
  on USB within 2 s is master. The right half, powered over the link's VBUS, never enumerates and becomes the
  slave. Nothing is set for this in `keyboard.json`.
- **Handedness:** GP0 high = left (QMK's default polarity), matching the spec's straps (left 10 k to 3V3, right
  10 k to GND).
- **Serial:** `driver: vendor` is the RP2040 PIO driver; one pin, half-duplex. The RP2040 needs no pull-up.
- **Right-half pins:** `split.matrix_pins.right`. Columns are listed C0 → C5 on both halves, C0 outermost, so the
  layout's right-half matrix positions run `[4,5] … [4,0]` left to right, as in Corne.

## Flashing

Each half is flashed over its own module USB-C (UF2 drag-and-drop onto the `RPI-RP2` drive). Ways into the
bootloader:
- first flash: hold the module's BOOT button while plugging in;
- bootmagic: hold the outer top key of that half (left `[0,0]`, right `[4,0]`) while plugging in;
- `QK_BOOT` on the ADJUST layer (both outer top keys). It resets the half that is master, so to reflash the right
  half, unplug the link and plug the right module in on its own.

## Default keymap

Corne-style, four layers. LOWER and RAISE are the middle thumbs (`TL_LOWR`, `TL_UPPR`); holding both gives ADJUST
(QMK `tri_layer`).

| Layer | Contents |
| --- | --- |
| BASE | QWERTY; Tab/Ctrl/Shift on the left outer column, Bksp/'/Esc on the right; thumbs GUI, LOWER, Space · Enter, RAISE, AltGr |
| LOWER | number row; arrows on HJKL, Home/PgDn/PgUp/End below |
| RAISE | shifted symbols on the top row; minus, equals, brackets, backslash, grave and their shifted forms on the right |
| ADJUST | F1–F12, media and brightness keys, `QK_BOOT` on both outer top keys |

The on-board WS2812 (GP16) is not enabled.
