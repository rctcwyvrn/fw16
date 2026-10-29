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

## Measurements to take

- [ ] Dual M.2 + NVMe reference in the bay: VSYS / 5V / 3V3 present in S0, S3, S5 (with its stock EEPROM).
- [ ] Card in the bay: VSYS voltage on AC and on battery; total draw idle / loopback; Tj (S6).
