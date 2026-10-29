# Bay power budget and card power tree

Spin-1 draft, 2026-10-28. Decision [0008](../docs/decisions/0008-card-power-input.md): VSYS_GPU primary, ALW
rails for management only. Numbers marked **TBD** are measured in Phase 2 (Path 4 step 5 / S6).

## Inputs from the interposer

| Rail | Pads | Spec | Card use | Enabled by EC (accessory descriptor) |
| --- | --- | --- | --- | --- |
| VSYS_GPU | P1–P14 | 7–20 V; ≥ 15.2 V on battery; ≤ 10.5 A on AC, practical limit lower | FPGA/SOM power tree | S3 and up — **live in suspend** |
| 5V_ALW | P27–P30 | 5 V ± 6 %, ≤ 2.5 A | management 5 V (if needed) | S5 and up |
| 3V3_ALW | P34–P37 | 3.3 V ± 6 %, ≤ 2.0 A | EEPROM, sensors, sequencer, LEDs | S5 and up |
| 12V_FAN | P65 | 12 V, ≤ 0.75 A | fans only, straight through | per FAN block |
| VADP_GPU | P41–P46 | power *into* the laptop | not connected (pads only) | — |

Each interposer pad is rated 0.75 A.

## Load estimate (to be replaced by measurements)

| Load | Estimate | Note |
| --- | --- | --- |
| AU15P VCCINT 0.85 V | 2–4 W | design-dependent; XPE estimate TBD |
| GTH Gen4 x4 (AVCC/AVTT/AVCCAUX) | ~1.5 W | |
| VCCAUX / VCCO / VCCBRAM | 0.5–1.5 W | no DDR4 on spin 1 (branch A) |
| Clocks, flash, sensors, LEDs | < 0.5 W | |
| **Total at the rails** | **≈ 5–8 W** | ≈ 7–10 W from VSYS at ~85 % |

## Tree (spin 1)

```
VSYS_GPU ─ eFuse (EN from mgmt, PG) ─ current sense ─┬─ buck → 5V0_MAIN ─┬─ VCCINT/VCCBRAM 0.85 V
                                                     └─ bulk caps, TVS    ├─ MGTAVCC 0.9 V
                                                                          ├─ MGTAVTT 1.2 V
                                                                          ├─ VCCAUX / MGTVCCAUX / VCCO 1.8 V
                                                                          └─ VCCO 3.3 V (QSPI, LEDs, sideband)
3V3_ALW ─ EEPROM (0x50), mgmt sequencer, current-sense ADC, LEDs, PERST#/PWR_EN logic
12V_FAN ─ straight to the two fan connectors (AFC58-S04FIA-R1)
```

- The management domain (3V3_ALW) controls the eFuse enable. It drops the FPGA rails when PEX_RST# is held
  asserted in suspend, and brings them up before PEX_RST# deasserts. Timing vs PERST#: TBD from the CEM spec
  and measurement.
- Branch A (decision 0007): the core rails follow the AU+ power-on sequence (DS931). Regulator choice is pending.

## Proposal: group-2 part shortlists (2026-10-29, user to pick)

Candidates only. **Nothing here is chosen**; the user picks before the power_in / core_power sheets are drawn.
Stock and prices are an LCSC snapshot from 2026-10-29 (`wmsc.lcsc.com` product API). JLCPCB shares the
C numbers. Basic/Extended tier is "not confirmed" unless stated; expect Extended for every TI IC.
Design constraints used: VSYS 7–20 V, so abs-max must be ≥ 24 V and ideally ≥ 28 V (18 V parts are out).
The card draws ≈ 7–10 W from VSYS, so ≤ 1.5 A at 7 V. The eFuse EN comes from the 3.3 V management domain.

### power_in: eFuse on VSYS

| Part | Package | Vin op / abs-max | Current, Ron | Pins | Reverse blocking | Iq on / shutdown | LCSC, stock | Tradeoffs |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TPS259474LRPWR (TPS25947, adjustable OVLO, latch-off) | VQFN-HR-10 2 × 2 mm | 2.7–23 / 28 V | ILIM 0.5–6 A, 28 mΩ | EN/UVLO, PG, PGTH | yes (back-to-back FETs) | 428 µA / 4.4 µA | C2864845, 3 607, $0.79 @100 | Closest to the plan. Latch-off recovers on an EN toggle, which management owns. Assembly only, not hand-solderable. |
| TPS259461ARPWR (TPS25946) | QFN-10 2 × 2 mm | 2.7–23 / 28 V | ILIM 0.5–6 A, 28 mΩ | EN/UVLO, SPLYGD, FLT (**no PG**) | only when off | 428 µA / not checked | C3662780, 1 312, $0.94 @100 | Pin-compatible fallback. Loses PG and reverse blocking while on. |
| TPS26600PWPR (TPS2660) | HTSSOP-16 + EP | 4.2–60 / 62 V | ILIM 0.1–2.23 A, **150 mΩ** | SHDN# (active-low), FLT, IMON (**no PG**) | yes; reverse polarity to −60 V | 300 µA / 20 µA | C544399, 5 953, $0.96 @100 | Most voltage margin; hand-solderable. 2.23 A ILIM ceiling is near the 1.5 A worst case plus inrush. ≈ 0.34 W loss at 1.5 A. Its EP is RTN, not GND. |

**Trap:** the best-stocked TPS25947 with PG, **TPS259472ARPWR (C3662789), has a fixed 13.8 V output clamp
and cannot pass 20 V VSYS.** Only the 470x and 474x (adjustable OVLO) variants fit. SGMicro SGM2546 (10 mΩ,
TQFN 2 × 2) was not found at LCSC and has no reverse blocking.

### power_in: current/power monitor (private FPGA I²C bus)

All parts come in VSSOP-10 except INA219. All have 16 pin-strapped I²C addresses and run from a 2.7–5.5 V supply.

| Part | Bus range | ADC | Iq / shutdown | LCSC, stock | Tradeoffs |
| --- | --- | --- | --- | --- | --- |
| INA238AIDGSR | 0–85 V | 16-bit, ±5 µV offset | 640 µA / 5 µA | C2868250, **0 in stock** | The planned part. Needs consignment or global sourcing. |
| INA237AIDGSR | 0–85 V | 16-bit, ±50 µV offset, 1.6 % power accuracy | 640 µA / 5 µA | C2864837, 7 325, $1.20 @100 | Pin- and register-compatible drop-in for INA238 (software unchanged). 10× the offset matters only at small shunt voltages. |
| INA226AIDGSR | 0–36 V | 16-bit, ±10 µV offset | 330 µA / not checked | C49851, 42 029, $0.40 @100 | Cheapest, deep stock, 36 V is enough. Different register map from INA23x, so the driver differs. |

INA228 (20-bit, energy accumulator, C2887910) is also out of stock. INA219 (26 V bus, 12-bit) was rejected
on bus margin.

### power_in: VSYS → 5V0_MAIN buck (3 A class)

| Part | Package | Vin op / abs-max | Iq / shutdown | Light load, extras | LCSC, stock | Tradeoffs |
| --- | --- | --- | --- | --- | --- | --- |
| TPS62933DRLR | SOT-583 1.6 × 2.1 mm | 3.8–30 / 32 V | 12 µA / 2 µA | PFM; 200 k–2.2 MHz via RT; no PG (P/O variants have one) | C3200405, 24 611, $0.27 @100 | Smallest and lowest Iq. Fine-pitch package is hard to rework by hand. |
| TPS54302DDCR | SOT-23-6 | 4.5–28 / 30 V | 45 µA / 2 µA | Eco-mode, fixed 400 kHz, spread spectrum, no PG | C311983, 58 940, $0.16 @500 | Fewest parts, cheapest, hand-solderable. Least abs-max margin. 400 kHz means a bigger inductor. |
| LMR33630ADDAR | HSOIC-8 + EP 5 × 4 mm | 3.8–36 / 38 V | 24 µA / 5 µA | 400 kHz (B 1.4 MHz, C 2.1 MHz), PG pin | C841384, 31 118, $0.43 @1k | Most voltage margin and has PG for the sequencing chain. Physically largest. |

Sources (datasheets): [TPS25947](https://www.ti.com/lit/ds/symlink/tps25947.pdf),
[TPS25946](https://www.ti.com/lit/ds/symlink/tps25946.pdf), [TPS2660](https://www.ti.com/lit/ds/symlink/tps2660.pdf),
[INA238](https://www.ti.com/product/INA238), [INA237](https://www.ti.com/product/INA237),
[INA226](https://www.ti.com/product/INA226), [TPS62933](https://www.ti.com/lit/ds/symlink/tps62933.pdf),
[TPS54302](https://www.ti.com/lit/ds/symlink/tps54302.pdf), [LMR33630](https://www.ti.com/lit/ds/symlink/lmr33630.pdf).

## Measurements to take

- [ ] Dual M.2 + NVMe reference in the bay: VSYS / 5V / 3V3 present in S0, S3, S5 (with its stock EEPROM).
- [ ] Card in the bay: VSYS voltage on AC and on battery; total draw idle / loopback; Tj (S6).
