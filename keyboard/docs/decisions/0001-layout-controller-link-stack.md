# 0001 — Layout, controller, inter-half link and stack budget (KS1, KS2 partial)

**Date:** 2026-10-31
**Status:** accepted 2026-10-31 (user)

## Context

The K-layout worker built three Ergogen candidates with 1:1 templates (`keyboard/layout/`) and an options study
(`keyboard/docs/electronics-options.md`). Its estimate for a hot-swap Choc v1 half was about 13 mm against the
10–12 mm stowed budget.

## Decision

- **Layout B, 42 keys** (`keyboard/layout/b-6col-corne-3fan.yaml`): 6 columns × 3 rows + 3 fanned thumbs per half,
  half ≈ 142 × 102 mm. Fits the tray side by side. **KS1 frozen.**
- **Controller: an RP2040 module per half.** Layout B's 6 × 4 matrix needs 10 GPIO plus the link, so the module is the
  **RP2040-Zero** (20+ GPIO; Claude's call: the XIAO RP2040's 11 GPIO would force a half-duplex link).
- **Inter-half link: USB-C**, with CC and series-resistor protection against the laptop cable going into the link
  port.
- **Stowed budget relaxed to ≈ 13 mm per half**, keeping standard Kailh hot-swap sockets; the tray deepens by ~1 mm.

## Consequences

- Parts order: 42 Choc v1 switches + ~10 % spares, 42 hot-swap sockets, 42 diodes, keycaps, 2× RP2040-Zero, USB-C
  receptacles for the link and the laptop port.
- `keyboard/PLAN.md` acceptance moves from "≤ 12 mm stowed" to "≤ 13 mm"; tray step 3 sizes the insert for 13 mm.
- Still open for KS2: the GPIO map and which half carries the laptop port.
