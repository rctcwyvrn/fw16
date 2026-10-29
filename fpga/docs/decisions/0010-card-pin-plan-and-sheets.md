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
- **GTH Quad 225** (XDMA default for Gen4 x4, `PCIE4C_X0Y0`, lane order Bottom): lanes 0–3 = channels 0–3;
  `MGTREFCLK0_225` (V7/V6) = host `PEX_REFCLK` through 10 nF AC caps; FPGA TX → 220 nF AC caps → `PEX_RX*`.
- **PERST# is a fabric input on HD bank 84** (AF14), not T19/PERSTN0 (no tandem configuration).
- **All 3.3 V sidebands on HD bank 84 at 3.3 V** (VCCO = `+3V3_IO`): PEX_RST#, FPGA_CLKREQ_N, TH_ALERT/OVERT drives,
  sense I²C + ALERT#, PPS, LEDs. No level shifters.
- **Fabric clock: a 100 MHz 3.3 V LVCMOS oscillator** on AC13 (HDGC pin), not STARTUPE3 CFGMCLK alone.
- **GT bench clock on `MGTREFCLK1_225`: DNP footprint** (100 MHz LVDS osc + caps) for IBERT only.
- **PPS on U.FL** (33 Ω series + ESD) and **2 LEDs** (heartbeat, PCIe link).
- `MGTRREF_R`/`MGTAVTTRCAL_R`: 200 Ω 1 % calibration resistor (UG576).
- The pin plan is **PCB-led and Vivado-checked**: `bay/card/fpga/pinplan.csv` is the single pin table; Vivado
  checks it against the synthesized top (I/O DRC + placement), and the XDC is generated from it (S3).
  `pinplan.csv` is the omnimeter session's draft and has not been through that check yet.

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
- Vivado results from omnimeter (synth passes: LUT 32.7 %, BRAM 57.3 %, setup WNS +0.728 ns; XDMA BFM sim passes
  once the example EP copy gets tied-off shims, see lesson `xdma-example-tb-hierarchical-refs`) are not reproduced in
  fw16 yet; fw16's `sim_bfm.tcl` lacks the shims.
