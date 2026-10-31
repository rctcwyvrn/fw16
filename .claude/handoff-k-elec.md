# Handoff: lane K-elec (keyboard electronics, research + specs) — written 2026-10-31

Read first: `CLAUDE.md`, `PARALLEL.md` (lanes K-elec, K-fw; sync KS2), `keyboard/PLAN.md`,
**`keyboard/docs/decisions/0001-layout-controller-link-stack.md`**, `keyboard/docs/electronics-options.md`,
`keyboard/layout/` (layout B: `b-6col-corne-3fan.yaml`, its README and fit table), `.claude/lessons/ergogen-4-quirks.md`.

## Decisions already made (don't re-ask)
- Layout **B, 42 keys** (6 × 3 + 3 thumbs per half). **RP2040-Zero** module per half. **USB-C** between the halves,
  protected against the laptop cable going into the link port. Stack budget **≈ 13 mm** with Kailh hot-swap sockets.
- QMK, wired. KiCad files change only through Konnect in the interactive session: **no KiCad files from this lane**.

## Next steps
1. **Matrix and GPIO map** for each half on the RP2040-Zero: rows/cols (6 × 4 incl. thumbs, or a better shape), diode
   direction, pin choices that suit both QMK's split support (full-duplex PIO serial needs 2 pins, half-duplex 1) and
   PCB routing; the reversible-PCB question (one board for both halves?) with tradeoffs.
2. **Which half carries the laptop port**, and how the halves negotiate master (QMK `MASTER_LEFT`/`SPLIT_USB_DETECT`/
   `EE_HANDS`), given both halves have a USB-C receptacle for the link and one for the laptop.
3. **Link connector circuit:** the USB-C link carries only power + the serial line(s), not USB data. Spec what stops
   damage if a laptop/charger cable is plugged in (no CC pull-downs on the link port so a source won't supply VBUS;
   series resistors / ESD on the serial pins; VBUS handling). Cite datasheets/QMK docs.
4. **Netlist-level spec** in `keyboard/docs/pcb-spec.md`, written so the orchestrator can draw it in Konnect without
   further decisions: every part (footprint name in the stock KiCad libs or a named community library: Kailh Choc
   hot-swap, RP2040-Zero, USB-C 16-pin receptacle), every net, and the Ergogen outline/footprint positions to import.
   Say whether Ergogen 4.2.1 can emit a KiCad PCB with footprints placed (and whether its footprints match the chosen
   libs).
5. Optional if time remains: a QMK keyboard skeleton under `keyboard/firmware/` against that GPIO map that
   `qmk compile` builds (add `qmk` and the ARM toolchain to `flake.nix`, tool list only, committed alone).

Checkpoint: stop after step 4 (or 5). Commit only `keyboard/docs/`, `keyboard/layout/`, `keyboard/firmware/`, and
`flake.nix`/`flake.lock` alone. No dune, no Vivado.

## Judgment calls to surface
- The matrix shape, the pin map, reversible vs two boards, the master-detection method.
