# 0001 — Skip the Tier 1 drop-in board; go straight to a custom AU15P card

**Date:** 2026-10-28
**Status:** accepted; the "AUBoard 15P first" condition is superseded by [0007](0007-card-architecture.md)

## Context

The design doc planned a NiteFury (Artix-7 M.2) on Framework's Dual SSD card as a drop-in first step, then a
custom Artix UltraScale+ card later. On 2026-10-28 the NiteFury was found discontinued; the nearest substitute
is the Numato Aller A7 (same XC7A200T, backorder, quote-only). The module side of the bay interface turns out
to be a plain gold pad array that Framework's FXBeam interposer presses onto — no connector to source — and
the community has already built and shipped a custom PCIe module this way (the OCuLink adapter).

## Options

1. **Aller A7 on the Dual M.2 Adapter, then a custom card (the doc's path).** ~$340 and ~3 weeks to first DMA
   in the bay; proves bay/BIOS/driver on known-good hardware; Artix-7 constraints and a Gen2 x4 ceiling that
   would be thrown away.
2. **Straight to a custom AU15P card, with an AUBoard 15P on a desktop to prove silicon and gateware first.**
   3–5 months and ~2 spins to first DMA in the bay; no throwaway work; the card's own regulators sidestep the
   Dual M.2 Adapter's unknown 3.3 V budget; debug unknowns overlap on first power-up.

## Decision

Option 2. The user has the PCB, FPGA and driver background to carry it, and the finished card is what the
doc actually wants. Conditions: AUBoard 15P first; hard block configured for Gen3 fallback with Gen4 accepted
as a spin-2 goal; a $39 Dual M.2 Adapter + NVMe still bought as the bay's known-good reference; AU15P quote
obtained in Phase 0 with Efinix Ti375 as the fallback part.

## Consequences

- Phase 8 folds into Phase 2; the custom card is no longer deferrable, so the budget's only flex is the CPU
  choice and the Instrument 2/3 parts.
- Phase 4 gateware develops on the AUBoard until the card works; Phase 3 and the step-0 CPU checks run in
  parallel with the card's fab and bring-up.
- `bay/KEEPOUT.md` becomes a prerequisite for schematic work, not a nice-to-have.
