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

### core_power: XCAU15P-2FFVB676E rails from 5V0_MAIN

Same LCSC snapshot rules as above (2026-10-29, `wmsc.lcsc.com` product detail). Nothing is chosen.

#### Rail requirements (DS931 v1.9, 2026-05-19, § Recommended Operating Conditions)

| Rail | Min / typ / max | DS931 typ. quiescent (XCAU15P, 85 °C, blank) | Spin-1 load estimate | Basis |
| --- | --- | --- | --- | --- |
| VCCINT | 0.825 / 0.850 / 0.876 V | ICCINTQ 424 mA | **0.5–4.7 A; size for 6 A** | See "VCCINT disagreement" below |
| VCCINT_IO + VCCBRAM | 0.825 / 0.850 / 0.876 V | 44 + 5 mA | 0.05–0.3 A | DS931 static plus margin |
| VCCAUX + VCCAUX_IO | 1.746 / 1.800 / 1.854 V | 55 + 24 mA | 0.1–0.5 A | MPS reference: 0.5 A (VCCAUX/ADC) |
| VCCO, HP banks and bank 0 | 0.950–1.900 V | ICCOQ 1 mA | ≈ 0.1 A at 1.8 V | QSPI (MT25QU256), 1.8 V I/O |
| VCCO, HD banks | 1.140–3.400 V | (in ICCOQ) | 0.05–0.2 A at 3.3 V | LEDs, 3.3 V sideband |
| MGTAVCC | 0.873 / 0.900 / 0.927 V | — | **≈ 0.25 A** (one quad) | MPS: 0.75 A for all 12 GTH ÷ 3 |
| MGTAVTT | 1.164 / 1.200 / 1.236 V | — | **≈ 1.0 A** (one quad) | MPS: 3 A for all 12 GTH ÷ 3 |
| MGTVCCAUX | 1.746 / 1.800 / 1.854 V | — | ≈ 0.05 A | MPS: 0.15 A ÷ 3 |

- **None of these is an XPE number.** XPE and `report_power` were not run; the "÷ 3" scaling assumes MGT current
  goes with lane count. Both are to be replaced by an XPE estimate, then by S6 measurements.
- **VCCINT disagreement.** The sources differ by 2–4×:
  - DS931 static alone is 424 + 44 + 5 ≈ 0.47 A (typical, 85 °C; max static is higher, and only XPE gives it).
  - MPS's AU10P/15P reference designs list VCCINT at 1–1.2 A.
  - This file's load table above (2–4 W) implies 2.4–4.7 A.
  - Candidates below are sized for the upper bound (6 A).
- VCCINT's window is −2.9 / +3.1 %. That window has to hold DC setpoint, ripple and load-step together, so use
  a ±1 % reference and 0.1 % feedback resistors.
- MGT supply noise: UG576 (GTH user guide, § Power Supply and Filtering) carries the GTH noise limit. **It was
  not fetched this session.** The often-quoted "< 10 mVpp, 10 kHz–80 MHz" was seen only in AM017 (Versal GTM).
  Check UG576 before layout.
- Do not reuse the AUBoard-15P "up to 6 A / 14 A" figures as loads. They are regulator ratings on a dev kit.
  That guide also lists **MGTVCCAUX at 1.5 V, which contradicts DS931 (1.746 V min)**.

#### Power-on sequence (DS931 v1.9, § Power-On/Off Power Supply Sequencing)

These fragments are quoted from the docs.amd.com HTML edition, which has no page numbers. Page refs are still
to be added from the PDF. Text outside quotation marks is paraphrase.

- I/O and core: the recommended power-on sequence is "VCCINT, VCCINT_IO/VCCBRAM, VCCAUX/VCCAUX_IO, and VCCO",
  to "achieve minimum current draw and ensure that the I/Os are 3-stated at power-on". Power-off is the reverse.
- "If VCCINT and VCCINT_IO/VCCBRAM have the same recommended voltage levels, they can be powered by the same
  supply and ramped simultaneously." "VCCINT_IO must be connected to VCCBRAM."
- "If VCCAUX/VCCAUX_IO and VCCO have the same recommended voltage levels, they can be powered by the same supply
  and ramped simultaneously." "VCCAUX and VCCAUX_IO must be connected together."
- "VCCADC and VREF can be powered at any time and have no power-up sequencing requirements."
- GTH: "The recommended power-on sequence to achieve minimum current draw for the GTH or GTY transceivers is
  VCCINT, VMGTAVCC, VMGTAVTT OR VMGTAVCC, VCCINT, VMGTAVTT." "Both VMGTAVCC and VCCINT can be ramped
  simultaneously." "There is no recommended sequencing for VMGTVCCAUX." Power-off is the reverse.
- "If these recommended sequences are not met, current drawn from VMGTAVTT can be higher than specifications
  during power-up and power-down."

What this means for the card:
- VCCINT, VCCINT_IO and VCCBRAM can share **one 0.85 V rail**, as the tree above already draws.
- VCCAUX, VCCAUX_IO and the 1.8 V VCCO banks can share **one 1.8 V rail**.
- Breaking the GTH order raises MGTAVTT current during the ramp. DS931 does not call it a damage condition.
- **POR timing** (DS931 § Configuration Switching Characteristics):
  - T_POR is 65 ms max (15 ms max with POR_OVERRIDE).
  - It is counted from when the last of VCCINT, VCCBRAM, VCCAUX and VCCO_0 reaches 95 % of nominal.
  - T_PL is 7.5 ms max.
  - Together these set how long before PEX_RST# deassertion the rails must be up.
- **Ramp-time limits:** no T_VCC* row was found in the DS931 HTML sections that were read. DS923 (Virtex US+) gives
  0.2–40 ms. Confirm against the DS931 PDF before setting soft-start capacitors.
- A plain PG→EN daisy chain gives the power-on order but **not the reverse power-off order** when the eFuse
  drops. See the questions in the Path 2 report.
- The MPS reference report is self-contradictory on the sequence (its diagram and its ripple table disagree).
  DS931 governs.

Possible PG→EN order, for discussion:

```
5V0_MAIN PG → 0.85 V (VCCINT + VCCINT_IO/VCCBRAM) PG → 1.8 V (VCCAUX/AUX_IO/VCCO 1.8 V) PG ─┬→ MGTAVCC LDO PG → MGTAVTT LDO
                                                                                         └→ VCCO 3.3 V (HD banks)
MGTVCCAUX: any time (ferrite off 1.8 V, or its own LDO)
```

#### LDO input: where the MGT LDOs get their headroom (decide first)

Dissipation = (Vin − Vout) × I, using the estimates above:

| LDO input | MGTAVCC 0.9 V @ 0.25 A | MGTAVTT 1.2 V @ 1.0 A | Cost |
| --- | --- | --- | --- |
| 5V0_MAIN direct | 1.0 W | **3.8 W — not viable** | none |
| 1.8 V rail (shared with VCCAUX) | 0.23 W | 0.6 W | 1.8 V buck grows to ≈ 2 A, so the 2 A TLV62569 is out and a 3–6 A part is needed |
| Dedicated ≈ 1.5 V pre-regulator buck | 0.15 W | 0.3 W | one more buck and inductor; MGT switching noise is kept off VCCAUX |

TPS7A84A and TPS7A53 accept Vin down to 1.1 V. They need their BIAS pin only at the low end of Vin, so a 1.5 V or
1.8 V input should not need a bias rail. The exact threshold is from memory and must be checked against the
datasheet. TPS74801 always needs a BIAS rail (from 5V0_MAIN).

#### Candidates: VCCINT + VCCINT_IO/VCCBRAM, 0.85 V, 6 A class buck

| Part | Package | Vin | Iout, accuracy | Extras | LCSC, stock | Tradeoffs |
| --- | --- | --- | --- | --- | --- | --- |
| TPS566238PRQFR | VQFN-HR-9 1.5 × 2 mm | 3–18 V | 6 A, 0.6 V ±1 % (25 °C) | FCCM (low ripple at light load), 600 kHz, **PG** | C2876603, 7 380, $0.41 @100 | Best-stocked part with PG. FCCM costs light-load efficiency, which is irrelevant when the rails are off in suspend. 1.5 × 2 HR package is assembly-only. |
| TPS566231PRQFR | VQFN-HR-9 1.5 × 2 mm | 3–18 V | 6 A, 0.6 V ±1 % | Eco-mode (PFM at light load), PG | C3190178, 827 (non-P C2870135, 3 004), $0.51 @100 | Pin-compatible sibling. Eco-mode ripple at light load eats into the VCCINT window. Its only benefit is idle efficiency. |
| TPS628690ARQYR (TPS62869) | VQFN-HR-9 1.5 × 2.5 mm | 2.4–5.5 V | 6 A, ±1 % | I²C VID, 2.4 MHz | C3190184, **20**; TPS6286900CRQYR C3190180, **15** | Smallest inductor; I²C allows margining. Stock is too thin to rely on. 5.5 V abs-max means it must sit behind 5V0_MAIN only. |

Also considered, LCSC C number not found this session: TPS62867 (6 A, same family), TPS62827 (4 A, 1.5 × 1.5 mm),
TPS568230 (8 A, 3 × 3 mm), MPS MP2147 (4 A).

#### Candidates: 1.8 V (VCCAUX/VCCAUX_IO + 1.8 V VCCO, optionally the LDO inputs)

| Part | Package | Vin | Iout | Extras | LCSC, stock | Tradeoffs |
| --- | --- | --- | --- | --- | --- | --- |
| TLV62569ADRLR | SOT-563 | 2.5–5.5 V | 2 A | no PG (the PDRLR "P" variant has PG: 7 790 in stock, C number not captured) | C398367, 10 955, $0.11 @100 | Cheapest and deep stock. **2 A is not enough if the MGT LDOs also hang off 1.8 V.** |
| TLV62569DBVR | SOT-23-5 | 2.5–5.5 V | 2 A | no PG | C141836, 15 345, $0.06 @100 | Hand-solderable. Same 2 A ceiling. No PG, so the chain needs a supervisor or an RC delay. |
| TPS566238PRQFR | VQFN-HR-9 1.5 × 2 mm | 3–18 V | 6 A | PG, FCCM | C2876603, 7 380, $0.41 @100 | Same part as VCCINT: one BOM line, and enough to carry the LDO inputs. Oversized for VCCAUX alone. |

TLV62585DRLR (3 A, SOT-563) shows up as an LCSC alternate with 895 in stock. Its C number was not captured.

#### Candidates: VCCO 3.3 V (HD banks), ≈ 0.2 A

TLV62569ADRLR (C398367) or TLV62569DBVR (C141836), as for the 1.8 V rail. A 5 V → 3.3 V LDO is the
alternative. At 0.2 A it dissipates 0.34 W, but its C number was not verified this session. If a 3V3_ALW-fed
bank were acceptable, the rail could come from there, but that breaks decision 0008 (ALW feeds management only).

#### Candidates: MGTAVCC 0.9 V and MGTAVTT 1.2 V low-noise LDOs

| Part | Package | Vin / BIAS | Iout, dropout | Noise | LCSC, stock | Tradeoffs |
| --- | --- | --- | --- | --- | --- | --- |
| TPS7A8400ARGRR (TPS7A84A) | VQFN-20 3.5 × 3.5 mm | 1.1–6.5 V; BIAS only at low Vin (threshold to verify) | 3 A, 180 mV max | 4.4 µVrms | C524802, 8 278, $1.84 @100 (non-A TPS7A8400RGRR C527696, 7 840, $1.80) | The low-noise reference choice. Vout is pin-strapped 0.5–2.075 V in 25 mV steps, so no feedback divider. 3 A covers MGTAVTT with margin. One part number serves both rails. |
| TPS7A5301RPSR (TPS7A53) | VQFN-HR-12 2.2 × 2.5 mm | 1.1–6.5 V; optional BIAS | 3 A, ultra-low dropout | low-noise class (check datasheet) | C2868223, **289**, $1.93 @100 | Smallest footprint. Thin stock. HR package is assembly-only. |
| TPS74801DRCR | VSON-10 3 × 3 mm | **separate BIAS rail required** (ranges to verify in datasheet) | 1.5 A, low dropout with BIAS | **not a low-noise LDO** (fast transient, programmable soft-start) | C105263, 43 339, $0.33 @100 (onsemi NCP59748 C464052, 5 529) | Cheapest, deepest stock, second source. 1.5 A is tight for MGTAVTT at 1.0 A estimated. It meets the "LDO" decision but not "low-noise". |

TPS7A8300 / TPS7A8300A (2 A, 4.4 µVrms) are out on stock (6–358 units). SGM2049 and 3PEAK TPL920 are listed
as alternates at 0 stock.

#### Candidates: MGTVCCAUX 1.8 V, ≈ 0.05 A

| Option | LCSC, stock | Tradeoffs |
| --- | --- | --- |
| Ferrite bead + caps off the 1.8 V VCCAUX rail | — | No IC. VCCAUX switching noise reaches the QPLL supply unless the filter is designed for it (UG576/UG583 filter guidance not read). |
| TLV75518PDBVR (500 mA LDO, from 5V0_MAIN) | C2877863, 5 445, $0.16 @100 | Isolated, quiet, 0.16 W at 0.05 A. SOT-23-5. Alternates: TLV75718PDBVR, RT9013-18GB (stock listed, C numbers not captured). |
| Ferrite off the MGTAVTT/MGTAVCC LDO input rail | — | Only meaningful if a dedicated 1.8 V pre-reg exists. Otherwise the same as option 1. |

**Trap:** TPS7A20185PDBVR (C2877857) is a 1.85 V part. With its tolerance it can exceed DS931's 1.854 V max.

Sources: DS931 [Recommended Operating Conditions](https://docs.amd.com/r/en-US/ds931-artix-ultrascale-plus/Recommended-Operating-Conditions),
[Quiescent Supply Current](https://docs.amd.com/r/en-US/ds931-artix-ultrascale-plus/Quiescent-Supply-Current),
[Power-On/Off Sequencing](https://docs.amd.com/r/en-US/ds931-artix-ultrascale-plus/Power-On/Off-Power-Supply-Sequencing),
[Configuration Switching Characteristics](https://docs.amd.com/r/en-US/ds931-artix-ultrascale-plus/Configuration-Switching-Characteristics);
MPS test reports [AU10P/15P cost-optimized (MP5416)](https://media.monolithicpower.com/mps_cms_document/a/r/artixus_au10_15p_pmic_mp5416_discrete_solution_tr_01252024_1.pdf)
and [highest power (MP5475)](https://media.monolithicpower.com/mps_cms_document/a/r/artix_us_pmic_mp5475_highest_power_tr_01262024_2.pdf);
Tria AUBoard-15P user guide v1.0 § 5.1.32–5.1.33 (rail ratings, GreenPAK sequencer);
TI [TPS566238](https://www.ti.com/product/TPS566238), [TPS62869](https://www.ti.com/product/TPS62869),
[TLV62569](https://www.ti.com/product/TLV62569), [TPS7A84A](https://www.ti.com/product/TPS7A84A),
[TPS7A53](https://www.ti.com/product/TPS7A53), [TPS74801](https://www.ti.com/product/TPS74801),
[TLV755P](https://www.ti.com/product/TLV755P).

## Measurements to take

- [ ] Dual M.2 + NVMe reference in the bay: VSYS / 5V / 3V3 present in S0, S3, S5 (with its stock EEPROM).
- [ ] Card in the bay: VSYS voltage on AC and on battery; total draw idle / loopback; Tj (S6).
