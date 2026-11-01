#!/bin/sh
# Copy the keyboard's two KiCad projects (left/, right/) and their shared lib/ between a Windows working copy and the
# repo. KiCad on Windows cannot save files on \\wsl.localhost (.claude/lessons/kicad-cannot-save-to-wsl-share.md), so
# KiCad, and Konnect while KiCad has a board open, work on the Windows copy. The repo copy is the source of truth:
# `pull` before committing, `seed` after pulling others' changes, `check` before ending a turn.
#
#   keyboard/pcb/sync.sh seed|pull|check
set -eu

WIN="${FW16_KEYBOARD_WIN:-/mnt/c/Users/17786/fw16-keyboard}"
REPO="$(cd "$(dirname "$0")" && pwd)"
files() {
  (cd "$1" && ls */*.kicad_sch */*.kicad_pro */*.kicad_pcb */sym-lib-table */fp-lib-table */outline.dxf \
     lib/*.kicad_sym lib/*.pretty/*.kicad_mod 2>/dev/null)
}

case "${1:-}" in
  seed)
    for f in $(files "$REPO"); do
      mkdir -p "$WIN/$(dirname "$f")"
      if [ -e "$WIN/$f" ] && ! cmp -s "$REPO/$f" "$WIN/$f"; then
        echo "seed: $WIN/$f differs, left alone (pull or delete it first)" >&2
      elif [ ! -e "$WIN/$f" ]; then
        cp "$REPO/$f" "$WIN/$f" && echo "seed: $f"
      fi
    done ;;
  pull)
    [ -d "$WIN" ] || { echo "pull: no working copy at $WIN" >&2; exit 1; }
    for f in $(files "$WIN"); do
      mkdir -p "$REPO/$(dirname "$f")"
      cmp -s "$WIN/$f" "$REPO/$f" 2>/dev/null || { cp "$WIN/$f" "$REPO/$f" && echo "pull: $f"; }
    done ;;
  check)
    [ -d "$WIN" ] || { echo "keyboard-sync: no working copy at $WIN, skipped"; exit 0; }
    rc=0
    for f in $(files "$WIN"); do
      cmp -s "$WIN/$f" "$REPO/$f" 2>/dev/null || { echo "differs: $f (run keyboard/pcb/sync.sh pull)" >&2; rc=1; }
    done
    exit $rc ;;
  *) echo "usage: $0 seed|pull|check" >&2; exit 2 ;;
esac
