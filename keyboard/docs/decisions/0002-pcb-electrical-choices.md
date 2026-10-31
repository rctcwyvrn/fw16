# 0002 — Keyboard PCB electrical choices (KS2)

**Date:** 2026-11-01
**Status:** accepted 2026-11-01 (user, on `keyboard/docs/pcb-spec.md` from the K-elec worker)

## Decision

- **The left half is the master** and carries the laptop USB-C; QMK default handedness, `SPLIT_HAND_PIN` GP0 strap so
  one `.uf2` serves both halves.
- **Two mirrored boards**, not one reversible board.
- **Link power: CC-gated source on the left + Ra (1 k) on the right**: the left link port sources 5 V only when it
  sees our right board (Rp 56 k, two P-FETs gated by CC, PTC, Schottky); compliant chargers and laptops never put VBUS
  into either link port. The link cable must be a USB 2.0 C-to-C cable without an e-marker.
- **4× M2 mounting holes per half** in the thumb-to-matrix gap and the controller-zone corners (positions to be added
  to `keyboard/layout/positions-b.csv`).
- Claude's calls (override any): Konnect draws it **schematic-first with `positions-b.csv`** for placement, not by
  importing Ergogen's PCB (ERC and a real netlist; Ergogen's footprints are inline legacy text); the `case` outline's
  cumulative-expand error (8 mm, not 4 mm, per side) is **fixed in the layout configs next round** so the tray fit
  table is right.

## Consequences

- KS2 is frozen except for the hole positions: K-fw can start the QMK skeleton against `pcb-spec.md` §2.
- Before drawing, verify (spec §9): RP2040-Zero pad coordinates, the SOD-123 diode pad map, AO3401A pinout, socket body
  vs diode at +4.7 mm, J1's mouth offset.
