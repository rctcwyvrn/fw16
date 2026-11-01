#!/usr/bin/env bash
# Build the fw16kb QMK firmware out of tree. Run from anywhere inside the dev shell
# (`nix develop`), which provides `qmk` and `arm-none-eabi-gcc`:
#
#   bash keyboard/firmware/build.sh [keymap]      # default keymap: default
#
# 1. Clones qmk_firmware at QMK_REF into _build/qmk_firmware (git-ignored) if missing.
# 2. Symlinks keyboard/firmware/fw16kb into it as keyboards/fw16kb; QMK only finds
#    keyboard definitions under its own keyboards/ tree.
# 3. Runs `qmk compile` with QMK_HOME pointed at that checkout, so `qmk setup` and
#    ~/.config/qmk are never needed, and copies the .uf2 to _build/keyboard/firmware/.
set -euo pipefail

QMK_REF=0.34.7
KEYMAP="${1:-default}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
QMK_HOME="$ROOT/_build/qmk_firmware"
OUT="$ROOT/_build/keyboard/firmware"
export QMK_HOME

if [ ! -d "$QMK_HOME/.git" ]; then
    git clone --depth 1 --branch "$QMK_REF" --recurse-submodules --shallow-submodules \
        https://github.com/qmk/qmk_firmware.git "$QMK_HOME"
fi

LINK="$QMK_HOME/keyboards/fw16kb"
if [ ! -L "$LINK" ]; then
    ln -s "$ROOT/keyboard/firmware/fw16kb" "$LINK"
fi

cd "$QMK_HOME"
qmk compile -kb fw16kb -km "$KEYMAP" --parallel 8

mkdir -p "$OUT"
cp "$QMK_HOME/fw16kb_$KEYMAP.uf2" "$OUT/"
echo "built $OUT/fw16kb_$KEYMAP.uf2 ($(stat -c %s "$OUT/fw16kb_$KEYMAP.uf2") bytes)"
