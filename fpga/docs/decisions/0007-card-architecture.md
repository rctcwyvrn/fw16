# 0007 — No dev board; SOM carrier if it fits the bay, else minimal chip-down spin 1

**Date:** 2026-10-28
**Status:** accepted; the B/A branch resolves in Phase 0 once bay clearance is measured

## Context

Decision 0001 skipped the Artix-7 stepping stone but kept an AUBoard 15P ($699) as a desktop development
target and debug reference. On review, the AUBoard tests none of the card's real unknowns (interposer pad
signal integrity, the power tree from 5V_ALW, thermals in the bay); it only de-risks gateware and driver,
which is the part the user is strongest at. Meanwhile an AU15P system-on-module exists in the right package.

## Options

| | What the card is | First silicon in bay | Hard parts removed | Risk |
| --- | --- | --- | --- | --- |
| **B. SOM carrier** | 4–6 layer carrier: FXBeam pads, bucks, 168-pin SOM connector, rear I/O; Puzhi `PZ-AU15P-SOM` (`XCAU15P-2FFVB676I`, 1 GB DDR4 x16, 75 × 55 mm, PCIe 4.0 x4 brought out; quote-only) | ~6–8 weeks after SOM arrives | BGA assembly, DDR4 layout, core-rail sequencing | Stack height (SOM + connector + heatsink ≈ 10–14 mm) vs bay clearance; vendor docs quality |
| **A. Chip-down, minimal spin 1** | AU15P + power tree + PCIe x4 + QSPI + JTAG + fan pass-through; **no DDR4, no rear I/O** until spin 2 | ~2–3 months | DDR4 deferred | Board and gateware debugged together on first power-up |
| C. Keep AUBoard | as before | — | gateware risk | $699 spent on the least likely failure |

## Decision

**B if the SOM stack clears the bay, else A.** Phase 0 opens `FW_UMA_MODULE_REF_0.stp` and
`FW_EXPANSION_BAY_PCB_0807.stp` in FreeCAD, records the vertical clearance above the module PCB in
`bay/KEEPOUT.md`, and requests the Puzhi SOM quote (with mated connector height). The branch is recorded here
when taken. The AUBoard is dropped in both branches.

## Consequences

- Gateware (shell v0, Phase 4 app) is developed in simulation and against the XDMA example design until
  the card exists; the first real PCIe test is in the bay. Keep a bench 5 V current-limited power-up and a
  JTAG-only bring-up stage so the board is proven alive before PCIe is attempted.
- The $699 moves to the card budget; both branches come in under the old Phase 2 total.
- In branch B, decision 0006's part is whatever the SOM carries (`-2FFVB676I`); 0006 stays correct on package.
- In branch A, spin 2 is a committed cost, not a contingency.
