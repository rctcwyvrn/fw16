# Handoff: lane K-layout (+ K-elec research) — written 2026-10-31

Start `keyboard/PLAN.md` steps 1 and 2 (research only). Read first: `CLAUDE.md`, `PARALLEL.md` (lanes
K-layout, K-elec; sync KS1), `PLAN.md` (track K, the acquire list), `keyboard/PLAN.md`, `keyboard/tray/PLAN.md`,
`keyboard/tray/KEEPOUT.md` v1, and the design doc's "Split keyboard" section
(https://claude.ai/artifact/AnCbHXtriDXSs3vPVNYoFu; if you cannot read it, work from `keyboard/PLAN.md`).

## Decisions already made (don't re-ask)
- Two halves, ≈ 130 × 100 mm each, Kailh Choc v1 (PG1350) in hot-swap sockets, 5–6 columns × 3 rows + 3–4
  thumb keys per half, QMK on an RP2040 per half, wired first, one USB-C cable from one half to the laptop.
- The halves stow in the belly tray (keyboard/tray); the stock deck keyboard stays.
- No hardware in hand. The tray is printed first, machined later.
- KiCad changes only through Konnect in the interactive session: **no KiCad files from this lane.**

## Next steps (this round)
1. **Layout configs** in `keyboard/layout/`: Ergogen 4.2.1 is in nixpkgs (`ergogen`); add it to `flake.nix`
   (tool list only, committed alone). Write 3 candidate layouts that differ in what the user must choose
   (e.g. 5 vs 6 columns; column stagger strength; 3 vs 4 thumb keys / thumb arc), Choc spacing (18 × 17 mm
   unless you find a reason otherwise and say it).
2. **Printable 1:1 templates:** one PDF per candidate, both halves, keycap outlines + column labels + a 50 mm
   scale bar to check the print scale, letter or A4 (say which, and that "actual size" must be selected). If
   Ergogen's outputs don't give a clean PDF, convert SVG with a nix-provided tool (cairosvg, rsvg-convert, or
   inkscape) — never pip.
3. **Tray fit:** for each candidate, the half outline (+ case wall allowance you state) against the tray inner
   area in `keyboard/tray/KEEPOUT.md`; a short table.
4. **K-elec research → `keyboard/docs/electronics-options.md`:** HandyWork RP2040/QMK fork vs an RP2040 module
   per half (e.g. RP2040-Zero / Seeed XIAO RP2040 class) vs a bare RP2040; inter-half link (TRRS vs USB-C,
   including cable nesting in the tray); hot-swap socket and diode choices; stack height vs the 10–12 mm stowed
   budget; a BOM with sources and rough prices for the key counts of each candidate. Options with tradeoffs, no
   choice.

Checkpoint: stop when the PDFs are ready to print and the options doc is written. The user types on the paper
templates and picks (KS1).

## Rules for this lane
- Commit only `keyboard/layout/`, `keyboard/docs/`, and `flake.nix`/`flake.lock` (alone).
- No dune, no Vivado.

## Judgment calls to surface in the report
- Key spacing, paper size, the case-wall allowance used in the fit table, which candidates you chose and why.
