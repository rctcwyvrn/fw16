# 0011 — fpga sheet: decoupling, unused banks/quads, housekeeping pins

**Date:** 2026-10-29
**Status:** accepted 2026-10-29 (user: "Approve, but more VCCINT caps"; RCAL → 100 Ω). Drawn in `bay/card/fpga.kicad_sch` (A1) the same day.

## Context

Decision [0010](0010-card-pin-plan-and-sheets.md) fixes the pins the card uses (Quad 225, HD bank 84, bank 0,
clocks, PPS, LEDs). The `fpga` sheet also needs every other XCAU15P-FFVB676 ball resolved: rail decoupling,
the banks and quads spin 1 leaves unused, and the configuration/SYSMON housekeeping pins. Sources were read on
2026-10-29 (page numbers are the printed footers):

- UG583 v1.29 (2025-12-23) dropped its decoupling tables (p.8, points to Power Design Manager / AR 39033, which
  could not be retrieved). The last table is **UG583 v1.28 (2024-12-19) Appendix A, Table 154, p.323**, read in
  the Japanese edition (the English v1.28 PDF was not found).
- UG576 v1.7.1 (2021-08-18) for GTH power, RCAL and unused transceivers; it predates Artix UltraScale+, so
  its PSG tables do not list the AU15P.
- UG571 v1.16 (2025-01-14), UG570 v1.20.1 (2025-03-04), UG580 v1.10.1 (2021-09-15), UG575 v1.22, DS931 v1.9
  (2026-05-19).

## Decisions

**Decoupling (UG583 v1.28 Table 154, XCAU15P-FFVB676 row)**

| Rail (card net) | Table 154 | Proposed on the card |
| --- | --- | --- |
| VCCINT + VCCINT_IO + VCCBRAM (`+0V85_VCCINT`, one plane per DS931 Table 2 notes 3/7 and 0006's -2 grade) | VCCINT/VCCINT_IO: 330 + 100 + 47 + 10 µF; VCCBRAM/VCCINT_IO: 47 + 10 µF | 330 + 100 + 2× 47 + 2× 10 µF (**the sum of both columns — an estimate**; no source gives a merged-plane count), **plus 8× 100 nF 0402 under/around the BGA** (user asked for margin beyond Table 154's 0× 1.0 µF; the count of 8 is Claude's call) |
| VCCAUX + VCCAUX_IO (`+1V8`) | 47 + 10 µF | 47 + 10 µF |
| VCCO bank 0 (`+1V8`) | 47 + 10 µF per bank | 10 µF; shares the VCCAUX 47 µF (Table 154 note: one 47 µF per 4 banks sharing a VCCO) |
| VCCO HD 84 (`+3V3_IO`) | 47 + 10 µF | 47 + 10 µF |
| VCCO unused banks 64/65/66/85/86 (`+1V8`, see below) | 47 + 10 µF per bank | 2× 47 µF (5 banks sharing one rail, per 4) + 1× 10 µF per bank |
| MGTAVCC (`+0V9_MGTAVCC`), MGTAVTT (`+1V2_MGTAVTT`), MGTVCCAUX (`+1V8_MGTVCCAUX`) | UG576 Table 5-8: 1× 4.7 µF ±10 % per rail per power-supply group | 1× 4.7 µF each (the AU15P has a single `_R` group) |

The core_power sheet already carries bulk at the regulators (C24–C27 22 µF on VCCINT, C39/C40 and C44/C45 on
the MGT LDOs); the counts above are at the FPGA.

**Unused banks and quads**

- HP 64/65/66 and HD 85/86: **power their VCCO from `+1V8`** (UG571 p.23 "RECOMMENDED: Unused banks must be
  powered up", for ESD; UG583 v1.29 p.312 allows floating but with reduced ESD). Their I/Os float. 1.8 V
  rather than 3.3 V keeps the small `+3V3_IO` LDO for bank 84 only.
- VREF_64/65/66: **leave floating** (no VREF I/O standard; UG571 p.16 allows floating or 500 Ω–1 kΩ to GND).
- GTH quads 224 and 226: their MGT power pins **are powered** with 225. The AU15P has one power-supply group,
  and a group with any used quad must be powered (UG576 p.322). RX pairs → GND, TX pairs → unconnected, REFCLK
  pairs → unconnected (UG576 Table 5-9, pp.338–339).

**Housekeeping pins**

| Pin | Connection | Source |
| --- | --- | --- |
| VBATT | GND (no volatile key) | UG570 Table 9 p.28; DS931 Table 2 note 11 |
| POR_OVERRIDE | GND (tying it high needs all rails at 95 % within 2 ms, which the LM3880's 2 ms steps don't give) | UG570 Table 9 p.28, pp.39–40 |
| RSVDGND | GND | UG570 p.36; UG575 Table 1-7 p.36 |
| DXP, DXN | GND (SYSMON's internal sensor is enough for Tj) | UG575 Table 1-7 p.32 |
| VREFP, VREFN | GNDADC (on-chip reference) | UG580 Fig 1-3 p.13, Table 1-2 pp.14–15 |
| VP, VN | GNDADC (unused) | UG580 Table 1-2 p.15 |
| VCCADC | `+1V8` through a ferrite bead, 470 nF + 100 nF (100 nF at the ball) | UG580 Fig 5-1 p.97 and notes, p.96 |
| GNDADC | GND through a ferrite bead (on-chip sensors only) | UG580 Table 1-2, p.97 |

## Part choices (user, 2026-10-29)

- VCCINT bulk: **330 µF/2.5 V polymer tantalum, EIA-7343** + **100 µF/4 V MLCC, 1210**. Check the 7343's ~4 mm
  height against the 7.8 mm middle-zone ceiling (`bay/KEEPOUT.md`) at placement.
- Oscillators: **SiTime SiT8008 100 MHz LVCMOS 3.3 V, 2016** for CLK100M (populated); **SiTime SiT9121 100 MHz LVDS,
  3225** footprint for the DNP GT bench clock on MGTREFCLK1_225.
- U.FL input on AB15 (net `PPS_IN`, name kept for 0010/gateware): built for **PPS and a 10 MHz reference / fast
  trigger**, since it is spin 1's only external input. **TPD1E04U04** low-capacitance ESD (~0.5 pF) + 33 Ω series.
  The HD-bank LVCMOS input caps useful bandwidth at tens of MHz.

## As drawn: Claude's judgment calls (review)

- U14 footprint: KiCad `Package_BGA:BGA-676_27.0x27.0mm_Layout26x26_P1.0mm_Ball0.6mm_Pad0.5mm_NSMD` (Xilinx
  BGA-676); its 676 pad names match the Vivado ball list exactly. Pad size vs UG575's FFVB676 land pattern not checked.
- SiT9121: no KiCad symbol, so `fw16-fpga-module:SiT9121` was created (1 OE, 2 NC, 3 GND, 4 OUT, 5 ~OUT, 6 VDD) for
  KiCad's SiT9121 footprint; check the pinout against the datasheet's ordering-code variant before buying.
  Powered from `+3V3_IO`, OE tied high; DNP with its 100 nF and the two 10 nF AC caps.
- SiT8008 (Y1): OE tied to `+3V3_IO`, no series resistor on CLK100M.
- TPD1E04U04 uses KiCad's `TPD1E05U06DPY` symbol/footprint (same DPY0002A two-pin package, I/O + GND).
- SYSMON filter ferrites: BLM18PG221SN1 (0603) on VCCADC and GNDADC; PWR_FLAGs on both nets.
- LEDs: green 0603 with 1 kΩ (≈ 1.3 mA at 3.3 V).
- ERC warnings left: VBATT/POR_OVERRIDE/RSVDGND are "bidirectional" in the generated symbol and meet GND's
  power flag; the interposer's local `GND` label (carried from omnimeter, left as is).

## RCAL resistor (corrects 0010)

`pinplan.csv` carried **200 Ω** from omnimeter. UG576 Fig 5-1 p.320, p.321 and Table 5-9 p.339 specify **100 Ω 1 %**
between MGTRREF_R (AD9) and MGTAVTTRCAL_R (AD8), MGTAVTTRCAL tied to MGTAVTT, equal-length traces each < 0.5 Ω DC,
and the RCAL quad powered. The AU15P's RCAL pins sit in quad 224 (unused, but powered per the rule above).
**User chose 100 Ω (2026-10-29)**; `pinplan.csv` and 0010 are amended.

## Power estimate (2026-10-31, closes the open item)

`gateware/vivado/run.sh power` (power.tcl) placed the current shell + XDMA Gen4 x4 design with the card's pins
(fw16_fpga_pins.xdc) and ran `report_power` with vectorless defaults (12.5 % toggle, 0.5 static probability),
50 °C ambient, no airflow, no heatsink. Placement DRC: clean apart from RTSTAT-13 (not routed, expected). The
earlier "Vccint exceeds the package limit" warning came from the pin-plan check's unconstrained run and did not
recur.

| Rail | Estimate | Card supply |
| --- | --- | --- |
| VCCINT + VCCINT_IO + VCCBRAM (0.85 V) | 1.84 + 0.04 + 0.05 = **1.92 A** | TPS566238P, 6 A |
| VCCAUX + VCCAUX_IO (1.8 V) | 0.09 A | TLV62569P (shared 1.8 V) |
| MGTAVCC (0.9 V) | 0.47 A | TPS7A8400A, 3 A |
| MGTAVTT (1.2 V) | 0.56 A | TPS7A8400A, 3 A |
| MGTVCCAUX (1.8 V) | 0.01 A | TLV75518P |
| VCCO 3.3 V (bank 84) | 0.01 A | TLV75533P |

Total on-chip 2.95 W (2.47 W dynamic), estimated Tj 68.8 °C. Synthesis timing: setup WNS +0.728 ns; hold is
negative before routing, as expected. Re-run with real activity once the project slot carries logic.

## Not resolved by the sources

- AR 39033 (current per-device decoupling) could not be loaded; Power Design Manager was not run.
- No AU15P-specific PSG/RCAL-master table exists in UG576 v1.7.1.
