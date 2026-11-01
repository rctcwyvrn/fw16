# Handoff: lane K-fw (keyboard firmware) + layout outline fix — written 2026-11-01

Read first: `CLAUDE.md`, `PARALLEL.md` (lane K-fw), `keyboard/PLAN.md` step 5, `keyboard/docs/decisions/0001-*.md` and
`0002-*.md`, `keyboard/docs/pcb-spec.md` (§2 has the matrix, GPIO map and a `keyboard.json` fragment),
`keyboard/layout/README.md`, `.claude/lessons/qmk-split-fixed-pins.md`, `ergogen-4-quirks.md`, `rp2040-zero-vbus-direct.md`.

## Decisions already made (don't re-ask)
- Layout B, 42 keys, Corne `LAYOUT_split_3x6_3` matrix 6 × 4 COL2ROW per half; RP2040-Zero per half; the left half is
  master and carries the laptop USB-C; half-duplex serial on GP11 over the USB-C link's D+; `SPLIT_HAND_PIN` GP0 with a
  10 k strap per board, so one `.uf2` serves both halves. Two mirrored boards.

## Next steps
1. **Layout outline fix** (keyboard 0002, Claude's call): Ergogen applies each outline step's `expand` to the union so
   far, so the `case` outlines are 8 mm, not 4 mm, past the pitch boxes. Fix all three configs (`_pitch` expand 0,
   the wall expand once), rebuild the template PDFs, update the README's fit table. The `pcb` outline is already right.
2. **QMK skeleton** in `keyboard/firmware/`: a keyboard definition (`keyboard.json` with the split, serial, hand pin and
   both sides' matrix pins from the spec), a default keymap (Corne-style layers; keep it simple), and a build that runs
   from the dev shell. Add `qmk` and the ARM toolchain QMK needs (`gcc-arm-embedded` or what `qmk doctor` asks for) to
   `flake.nix` (tool list only, committed alone). Put the qmk_firmware checkout under `_build/` (git-ignored), never in
   the tree; build the keyboard out-of-tree or via a symlink/copy step you document in `keyboard/firmware/README.md`.
3. Prove it: `qmk compile` (or the documented equivalent) produces a `.uf2`; record its size and the command.
   Run long builds in the foreground with a timeout, never as background tasks.

Checkpoint: stop after step 3. Commit only `keyboard/layout/`, `keyboard/firmware/`, and `flake.nix`/`flake.lock` alone.
No KiCad files, no dune, no Vivado.

## Judgment calls to surface
- The keymap's layer set, how the out-of-tree build is wired, any spec pin that QMK rejects.
