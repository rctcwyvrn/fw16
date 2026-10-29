# 0008 — Card power input: VSYS_GPU primary, ALW rails for management only

**Date:** 2026-10-28
**Status:** accepted

## Context

The plan budgeted the card at 5V_ALW (2.5 A) + 3V3_ALW (2.0 A) ≈ 19 W, all FPGA rails from 5V_ALW. The
interposer also carries **VSYS_GPU ×14 pads** (7–20 V; 15.2 V min on battery; "up to 10.5 A" on a high-power
adapter), which the plan had missed. The FW16 EC enables VSYS_GPU for a non-GPU card when the card's EEPROM
holds the "PCIe accessory" descriptor (domain S3), and gates all bay rails — ALW included — on valid ID
resistors plus that EEPROM (`bay/README.md` § How the EC powers the bay). Estimated card load: 7–10 W
(AU15P core, GTH Gen4 x4, aux/IO, DDR4 on the SOM). 5V_ALW alone leaves ≈10.5 W after conversion.

## Options

| | Headroom | Cost | Risk |
| --- | --- | --- | --- |
| **VSYS primary** | tens of watts | 20 V-rated eFuse + buck; load switch the card opens itself in suspend | depends on the descriptor turning VSYS on |
| 5V primary, VSYS DNP | ≈10.5 W | simplest | a 10 W FPGA browns out spin 1 |
| Both, ORed | either source | ideal-diode controller, area, sequencing | complexity |

## Decision

**VSYS_GPU primary.** FPGA/SOM rails come from VSYS_GPU through an eFuse, current sense and a wide-input
buck. 5V_ALW / 3V3_ALW feed only management: EEPROM, sensors, sequencer, LEDs.

## Consequences

- The EEPROM descriptor and 330 k ID0/ID1 resistors are spin-1 mandatory (without them nothing is powered).
- VSYS stays live in S3/S0ix: the card must switch its own FPGA rails off on PEX_RST# assertion / D3, with
  the switch controlled from the ALW-powered management domain.
- Bench bring-up powers the card from a 15–20 V current-limited supply on the VSYS pads, plus 5 V/3.3 V.
- The plan's "card ≤ 19 W" acceptance item becomes a measured VSYS + ALW total; Phase 2 text to be updated.
