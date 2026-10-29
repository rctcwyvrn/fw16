# Phase 2 — Bay bring-up on a custom AU15P card

Goal: a custom card sitting directly on the bay's FXBeam interposer with an `XCAU15P` in the FFVB676 package,
running PCIe Gen4 x4 (Gen3 accepted for spin 1), with a heatsink in the fan airstream and a rear I/O
connector, doing sustained DMA from a Hardcaml design. There is no drop-in board and no dev board (decisions
[0001](../docs/decisions/0001-tier1-board.md), [0007](../docs/decisions/0007-card-architecture.md)): gateware
is developed in simulation, a $39 Dual M.2 Adapter with an NVMe proves the bay side, and the first real PCIe
test is the card itself in the bay.

Two branches, chosen in Phase 0 from the bay's measured clearance:

- **B — SOM carrier**: Puzhi `PZ-AU15P-SOM` on a 4–6 layer carrier. First silicon in the bay ~6–8 weeks
  after the SOM arrives.
- **A — minimal chip-down**: spin 1 is AU15P + power + PCIe + QSPI + JTAG + fan pass-through, no DDR4, no
  rear I/O; spin 2 adds them. First silicon ~2–3 months.

Phase 3 (tray) and the step-0 CPU checks of Phases 4 and 7 run in parallel with fab and bring-up.

## Steps

### A. Gateware and host, in simulation (no hardware)
1. **Hardcaml shell v0** in `gateware/shell/`: AXI-Lite register file (ID, version, scratch, status), one
   AXI-Stream source generating counter/PRBS at a programmable rate, one sink that checks it; a DDR4-backed
   ring buffer behind a memory interface abstraction so branch A spin 1 can substitute BRAM. Verilog via
   `Hardcaml.Rtl`; an OCaml testbench drives the AXI side with `hardcaml_waveterm` traces checked in.
2. **XDMA integration**, Tcl-scripted Vivado project (`gateware/vivado/*.tcl`, no GUI state in git): XDMA
   IP at Gen4 x4 / 256-bit AXI at 250 MHz with Gen3 fallback enabled, wrapping the shell. Run the XDMA example
   design's simulation (PCIe root-port BFM) against the shell to prove the AXI interfaces before any board.
   Target `xcau15p-ffvb676-2-e` (or `-i` in branch B).
3. **Host side** in `host/` and `driver/`: udev rules for the XDMA char devices, `omni_transport` C library
   (`open/read_regs/write_regs/dma_read/dma_write`) with a `local_xdma` backend and a `mock` backend for CI;
   `omni-loopback` CLI reporting GB/s and error count. Phase 9 adds the `aws_f2` backend to this same library.
4. **Timestamps and interrupts**: MSI-X on sink-done; a free-running 64-bit timestamp with a host-readable
   latch and a 1-PPS input — the first "service every instrument gets". Simulated with a PPS stimulus.

### B. Survey the bay (laptop, no custom hardware)
5. **Prove the bay with a $39 part.** Dual M.2 Adapter + interposer + a cheap NVMe in the stock shell:
   BIOS ≥ 3.05, the SSD enumerates, survives suspend/resume. This is the known-good reference every later
   "why doesn't it enumerate" question is measured against.
6. **Finish `bay/KEEPOUT.md`** (started in Phase 0 from the STEP files) against the physical shell: PCB
   outline, FXBeam pad fields (two 74-pin fields, 27.75 mm), threaded-insert positions, fan footprints and
   duct zones, max component height above and below the PCB, the rear wall's free area. The community warns
   the interface area is tight (vias-in-pad on the lanes, ≥ 6 layers) and that the spec "could change again"
   — diff the repo against your clone date before fab.
7. **Rail survey.** Inline USB-C PD meter for system draw; confirm from the Electrical README what the card
   may draw: 5V_ALW ≤ 2.5 A, 3V3_ALW ≤ 2.0 A, VADP_GPU only with ≥ 180 W PD (GPIO2_EC), 12V_FAN pass-through.
   Write `bay/POWER.md` with the budget the card must meet: **≤ 19 W, all from 5V_ALW except I/O.**
8. **EC behaviour.** Confirm GPIO0–3_EC and the I²C_ALW bus do nothing that would fight a non-GPU module (the
   Dual M.2 Adapter already implies plain endpoints are fine; note what it connects on those pins if visible).

### C. Design the card (`bay/card/`, KiCad)
9. **Common to both branches.** Land pattern for the two FXBeam fields from the community footprints, checked
   pin by pin against the Electrical README (power field: 5V_ALW ×4, 3V3_ALW ×4, VADP_GPU ×6, 12V_FAN, Fan0/1
   PWM and SPEED, I²C_ALW, GPIO0–3_EC, ID0; signal field: 4 PCIe lanes of 8, REFCLK, PERST#, CLKREQ#, USB 2.0,
   DP pass-through untouched). **Fan pass-through is mandatory** — the shell's two fans are the laptop's
   cooling. Bucks from 5V_ALW; 3V3_ALW for 3.3 V I/O only; VADP_GPU input stage footprint-only. XADC/SYSMON
   temperature readable from the host. Rear: SMA for PPS, status LEDs, and (spin 2 in branch A) the Phase 3
   LVDS link (HDMI-type) plus a SYZYGY-style or dual-SFP+ footprint for multi-gigabit tray links.
10. **Branch B — carrier.** 168-pin B2B connector per the SOM pinout; route 4 GTH lanes + REFCLK from the
    connector to the pads as short as the geometry allows; SOM power input per its spec; QSPI/JTAG per the
    SOM (usually on-module). 4–6 layers, controlled impedance. Heatsink on the SOM's FPGA, between the fan
    outlets; verify the stack against `bay/KEEPOUT.md` with a printed dummy before fab. Rear I/O lands on
    spin 1 since the hard parts are on the SOM.
11. **Branch A — chip-down spin 1.** `XCAU15P-2FFVB676E`, power tree (VCCINT 0.85 V, VCCAUX/VCCO 1.8 V, GTH
    0.9/1.2/1.8 V, sequenced per the AU+ datasheet), QSPI, JTAG header, 4 GTH lanes to the pads, fan
    pass-through, SMA, LEDs. **No DDR4, no rear data connectors.** 8–10 layers, Gen4-capable lanes
    (< 5 cm, via-in-pad/back-drill) with the hard block configured for Gen3 fallback. Fab at a
    controlled-impedance house with BGA assembly. Spin 2 adds 2 GB DDR4 x16 and the rear connectors once
    PCIe and power are proven.
12. **Bring-up checklist** `bay/card/BRINGUP.md`, staged so the board is proven alive before PCIe is tried:
    bench supply, current-limited 5 V → rail sequencing and ripple → JTAG ID → flash → *then* in the bay:
    PCIe training (`LnkSta` — record Gen3 vs Gen4), memory calibration (B, or A spin 2), fans still
    EC-controlled, thermal soak, suspend/resume, then shell v0 and `omni-loopback`. Decide the next spin's
    scope from this list.
13. **Cut the rear I/O slot** in the extra shell to match the card; print a bezel that doubles as strain relief.

## Parts

| Part | Price | Note |
| --- | --- | --- |
| Dual M.2 Adapter + cheap 2230/2280 NVMe | $39 + ~$30 | Bay reference; buy with the laptop |
| Graphics Module Interposer ×2 | $29 ea | One for the Dual M.2 reference, one for the card |
| Threaded inserts ×4, SMT standoffs | free (Framework forms) | |
| **B:** Puzhi PZ-AU15P-SOM | ~$300–450 (quote) | AU15P FFVB676 + 1 GB DDR4 on-module |
| **B:** 2 carrier spins, 4–6 layer, impedance-controlled | ~$150 | |
| **B:** B2B connector, bucks, SMA, LEDs, heatsink stock | ~$120 | |
| **A:** XCAU15P-2FFVB676E ×2 | TBD (quote) | |
| **A:** 2 PCB spins, 8–10 layer, BGA assembly | ~$400 | |
| **A:** DDR4, regulators, connectors, heatsink stock | ~$300 | |
| Inline USB-C PD meter, thermocouple logger | ~$55 | |

## Acceptance (gate to Phase 4 hardware)

- [ ] Shell v0 passes its OCaml testbench and the XDMA BFM simulation; `omni-loopback` builds from a clean
      checkout and passes against the `mock` backend.
- [ ] `bay/KEEPOUT.md` and `bay/POWER.md` written with measured numbers; NVMe-in-bay reference documented.
- [ ] Card enumerates in the FW16 bay at ≥ Gen3 x4; memory (B, or A spin 2) passes memtest; fans still
      EC-controlled.
- [ ] 1 h loopback in the bay, zero errors, FPGA Tj < 85 °C, card ≤ 19 W measured, no CPU throttling beyond
      the no-card baseline.
- [ ] Suspend/resume works or the limitation is documented with a workaround.
- [ ] Rear slot cut; the LVDS link cable passes through with the shell closed (A: after spin 2).

## Open questions

- Branch B: SOM stack height, connector part number and pinout quality — all from the Phase 0 quote.
- Does the bay's root port train Gen4 to a non-GPU endpoint with the stock BIOS? Spin 1 answers it.
- Is the bay's USB 2.0 pair worth routing to the rear as a free low-speed tray channel? Cheap to add.
