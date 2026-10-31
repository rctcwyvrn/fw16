# 0010 — Card pin plan, PCIe quad, and the sideband/config sheet designs

**Date:** 2026-10-29
**Status:** accepted (decisions made in omnimeter's Phase 2 sessions after the fw16 import; carried over as
decisions only — the fw16 KiCad sheets are still to be drawn to this spec)

## Context

The card work continued in omnimeter between the fw16 import (omnimeter `17b8cf9`) and omnimeter being frozen
(user, 2026-10-29: "omnimeter should have no changes from now on"). The user chose not to re-import; this record
carries the decisions so fw16 continues from them. Source commits in omnimeter: `d2d643b` (pcie_sideband),
`0a57159` (config), `989cac3` (AU15P symbol), `0ee72b0` (core_power; spec in `bay/POWER.md`), `10fec6b` (BFM sim,
Quad 225), plus the uncommitted pin plan (now `bay/card/fpga/pinplan.csv`).

## Decisions

**PCIe quad and pins**
- **GTH Quad 225** (XDMA default for Gen4 x4, `PCIE4C_X0Y0`): **PCIe lane n = channel 3−n** (lane 0 on RX P2 / TX R5,
  lane 3 on RX Y2 / TX AA5). *Amended 2026-10-31 (user chose to swap the lanes on the card): the original "lanes 0–3 =
  channels 0–3" is wrong; see below.*
  `MGTREFCLK0_225` (V7/V6) = host `PEX_REFCLK` through 10 nF AC caps; FPGA TX → 220 nF AC caps → `PEX_RX*`.
- **PERST# is a fabric input on HD bank 84** (AF14), not T19/PERSTN0 (no tandem configuration).
- **All 3.3 V sidebands on HD bank 84 at 3.3 V** (VCCO = `+3V3_IO`): PEX_RST#, FPGA_CLKREQ_N, TH_ALERT/OVERT drives,
  sense I²C + ALERT#, PPS, LEDs. No level shifters.
- **Fabric clock: a 100 MHz 3.3 V LVCMOS oscillator** on AC13 (HDGC pin), not STARTUPE3 CFGMCLK alone.
- **GT bench clock on `MGTREFCLK1_225`: DNP footprint** (100 MHz LVDS osc + caps) for IBERT only.
- **PPS on U.FL** (33 Ω series + ESD) and **2 LEDs** (heartbeat, PCIe link).
- `MGTRREF_R`/`MGTAVTTRCAL_R`: **100 Ω** 1 % calibration resistor, MGTAVTTRCAL tied to MGTAVTT (UG576 v1.7.1 Fig 5-1,
  Table 5-9). *Amended 2026-10-29 (user, via [0011](0011-fpga-sheet-decoupling-and-tieoffs.md)): the carried-over
  value was 200 Ω.*
- The pin plan is **PCB-led and Vivado-checked**: `bay/card/fpga/pinplan.csv` is the single pin table; Vivado
  checks it against the synthesized top (I/O DRC + placement), and the XDC is generated from it (S3).
  The check (`bay/tools/pinplan_check.tcl`) **passed 2026-10-29** on fw16's own synth: Vivado placed the four
  lanes on GTHE4_CHANNEL_X0Y4–X0Y7 (Quad 225) with REFCLK on V7 and PERST# on AF14. Placement only, not routed;
  Vivado's vectorless power estimate warned that Vccint current exceeds the package limit (open item, see 0011).
  **That check missed the lane order**: it listed only the GT sites, which were right, not which port landed on which
  ball. Fixed 2026-10-31: it and `gateware/vivado/power.tcl` now fail if any port is not on its pin-plan ball.

**AU15P symbol**: generated from Vivado's package-pin list (`get_package_pins` → CSV → Konnect `create_symbol`),
not hand-built: 14 units (bank 0, GTH 224/225/226, HP 64/65/66, HD 84/85/86, power, GND ×2, NC), verified ball by
ball against the CSV.

**pcie_sideband sheet**
- TH_ALERT# / TH_OVERT#: 2N7002K open-drain FETs (Q1/Q2) driven by FPGA_TH_ALERT / FPGA_TH_OVERT, 100 k gate pulldowns.
- PEX_CLKREQ#: 0 Ω to GND, plus a DNP 0 Ω to FPGA_CLKREQ_N.
- No PERST# self-gating of the rails: superseded by the core_power sequencer's EN = `5V0_PG` only.

**config sheet**
- MT25QU256ABA1EW7 x4 QSPI on +1V8.
- UG570 pull-ups: FCS_B 2.4 k; D02, D03, PROGRAM_B, INIT_B, DONE 4.7 k.
- M[2:0] = 001 via 1 k straps.
- PUDC_B 1 k to +1V8 (pull-ups off during config, so the TH_* FET gates stay low).
- J4 Tag-Connect TC2050-IDC-NL, TC2050-XILINX mapping: 1 VREF, 2 TMS, 4 TCK, 6 TDO, 8 TDI, odd pins GND, 10 NC.
- Expected ERC error until the fpga sheet drives FPGA_CCLK: the flash CLK input is undriven.

**core_power sequencer trigger**: EN = `5V0_PG` only (not PEX_RST#, no hold-up cap). Full core_power spec: `bay/POWER.md`.

## Consequences

- fw16's `pcie_sideband`, `config`, `core_power` and `fpga` sheets are redrawn here to this spec through Konnect.
  The `fw16-fpga-module` library needs the AU15P symbol and the RQF0009A footprint recreated.
- Reproduced in fw16 on 2026-10-31 with the card's pins in place (`fw16_fpga_pins.xdc`, generated from
  `pinplan.csv`): synthesis setup WNS +0.728 ns, and the XDMA BFM simulation passes. It needed the tied-off
  shims from lesson `xdma-example-tb-hierarchical-refs`, now in `sim_bfm.tcl`. Link-up is at 187.5 µs, and
  BAR0[0x0] reads 0x46504741.
- **Lane order corrected 2026-10-31.** The rerun with the timestamp unit gave 8× CRITICAL WARNING 12-1411. The IP's
  `xdma_0_pcie4c_ip_gt.xdc` LOCs gtwizard `inst[k]` to `GTHE4_CHANNEL_X0Y(4+k)`, PCIe lane 0 is driven by `inst[3]`,
  and the pin XDC's conflicting PACKAGE_PINs were dropped, so the card as drawn was lane-reversed. Setting
  `lane_order = Top` (with a fresh out-of-context resynthesis) changed nothing. The user chose to swap the lanes on the
  card: `pinplan.csv` was remapped, the regenerated `fw16_fpga_pins.xdc` follows it, and the `fpga` sheet's 16 lane labels
  on U14 were moved through Konnect. The netlist diff shows exactly those 16 U14 pins changed, and card ERC has 0 errors.
  Vivado rerun with the swapped pins (project, synth, power): **0 critical warnings**, every port on its pin-plan
  ball, setup WNS +0.728 ns, 2.94 W, Tj 68.8 °C. The BFM sim (it does not model the board pins) passed earlier the
  same day with the timestamp unit: link up at 187.5 µs, BAR0 = 0x46504741.
