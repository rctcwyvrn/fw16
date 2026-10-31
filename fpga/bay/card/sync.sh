#!/bin/sh
# Copy the card's KiCad project between its Windows working copy and the repo.
#
# KiCad on Windows cannot save files that live on \\wsl.localhost: its atomic save fails copying Windows
# permissions onto the temp file (.claude/lessons/kicad-cannot-save-to-wsl-share.md, from momentum). So KiCad, and
# Konnect while KiCad has the project open, work on a Windows copy, and this script carries the files across. The
# repo copy is the source of truth: run `pull` before committing card work.
#
#   fpga/bay/card/sync.sh seed    repo -> working copy (refuses to overwrite a differing file)
#   fpga/bay/card/sync.sh pull    working copy -> repo
#   fpga/bay/card/sync.sh check   exit 1 if any file differs
set -eu

WIN="${FW16_CARD_WIN:-/mnt/c/Users/17786/fw16-card}"
REPO="$(cd "$(dirname "$0")" && pwd)"
# Project truth only: .kicad_prl is per-user UI state; locks, backups and .history are KiCad's own.
files() {
  (cd "$1" && ls *.kicad_sch *.kicad_pro *.kicad_pcb sym-lib-table fp-lib-table \
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
    [ -d "$WIN" ] || { echo "card-sync: no working copy at $WIN, skipped"; exit 0; }
    rc=0
    for f in $(files "$WIN"); do
      cmp -s "$WIN/$f" "$REPO/$f" 2>/dev/null || { echo "differs: $f (run fpga/bay/card/sync.sh pull)" >&2; rc=1; }
    done
    exit $rc ;;
  *) echo "usage: $0 seed|pull|check" >&2; exit 2 ;;
esac
