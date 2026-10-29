# Phase 2 — Bay bring-up on a custom AU15P card

Goal: a custom Artix UltraScale+ AU15P card sitting directly on the bay's FXBeam interposer, running PCIe
Gen4 x4 (Gen3 accepted for spin 1), with DDR4, a heatsink in the fan airstream and a rear I/O connector,
doing sustained DMA from a Hardcaml design. There is no drop-in stepping stone (decision
[0001](../docs/decisions/0001-tier1-board.md)): the AUBoard 15P on a desktop proves the silicon and gateware
side, a $39 Dual M.2 Adapter with an NVMe proves the bay side, and the card joins the two.

Expect 3–5 months from parts-in-hand to first DMA in the bay, and budget two board spins. Phase 3 (tray) and
the step-0 CPU checks of Phases 4 and 7 run in parallel; Phase 4 gateware develops on the AUBoard until the
card works.

## Steps

### A. Prove the silicon on the AUBoard 15P (desktop)
1. **Bring up the AUBoard** in a desktop PCIe slot: Avnet reference design, `lspci` shows Gen4 x4, DDR4
   passes the vendor memory test. Pin the Vivado version in `gateware/README.md` (confirm AU15P is covered by
   the free Standard edition before buying; if not, budget a Vivado licence or switch to the Ti375 path).
2. **Hardcaml shell v0** in `gateware/shell/`: AXI-Lite register file (ID, version, scratch, status), one
   AXI-Stream source generating counter/PRBS at a programmable rate, one sink that checks it, a DDR4-backed
   ring buffer. Verilog via `Hardcaml.Rtl`, wrapped with the XDMA IP (Gen4 x4, 256-bit AXI at 250 MHz) in a
   Tcl-scripted Vivado project; no GUI state in git.
3. **Host side** in `host/` and `driver/`: udev rules for the XDMA char devices, `omni_transport` C library
   (`open/read_regs/write_regs/dma_read/dma_write`) with a `local_xdma` backend, `omni-loopback` CLI reporting
   GB/s and error count. Phase 9 adds the `aws_f2` backend to this same library.
4. **Sustained desktop test**: 1 h loopback at max rate, zero errors; expect ~6 GB/s at Gen4 x4. Add MSI-X on
   sink-done and the free-running 64-bit timestamp with a host-readable latch and a 1-PPS input stub — the
   first "service every instrument gets".
5. **Optional in-bay silicon test**: AUBoard → M.2-to-PCIe-x4 riser cable in the Dual M.2 Adapter, lid off.
   Ugly, but it shows whether the bay trains Gen4 to a non-GPU AU15P endpoint before any layout exists.

### B. Survey the bay (laptop, no custom hardware)
6. **Prove the bay with a $39 part.** Dual M.2 Adapter + interposer + a cheap NVMe in the stock shell:
   BIOS ≥ 3.05, the SSD enumerates, survives suspend/resume. This is the known-good reference every later
   "why doesn't it enumerate" question is measured against.
7. **Record the keepouts.** Open `PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.{pdf,dxf,stp}` and
   `Mechanical/ExpansionBayShell/FW_UMA_MODULE_REF_0.{pdf,stp}` in FreeCAD/KiCad. Write `bay/KEEPOUT.md`: PCB
   outline, FXBeam pad-array positions (two 74-pin fields, 27.75 mm), threaded-insert positions, fan
   footprints and duct zones, max component height above/below the PCB, the open middle area's x/y/z, the
   rear wall's free area. The community warns the interface area is tight (vias-in-pad on the lanes, ≥ 6
   layers) and that the spec "could change again" — diff the repo against your clone date before fab.
8. **Rail survey.** Inline USB-C PD meter for system draw; confirm from the Electrical README what the card
   may draw: 5V_ALW ≤ 2.5 A, 3V3_ALW ≤ 2.0 A, VADP_GPU only with ≥ 180 W PD (GPIO2_EC), 12V_FAN pass-through.
   Write `bay/POWER.md` with the budget the card must meet: **≤ 19 W, all from 5V_ALW except I/O.**
9. **EC behaviour.** Confirm GPIO0–3_EC and the I²C_ALW bus do nothing that would fight a non-GPU module (the
   Dual M.2 Adapter already implies plain endpoints are fine; note what it connects on those pins if visible).

### C. Design the card (`bay/card/`, KiCad)
10. **FPGA decision record** `docs/decisions/0006-fpga.md`: AU15P (`XCAU15P-2SBVB484E`, 0.8 mm pitch, 12 GTH,
    Gen4 x4 hard block) unless the quote is > $300 or allocated, in which case Efinix Ti375 (free tools, Gen4
    x4 hard block, LPDDR4) and the Hardcaml→Efinity flow is accepted. **Get the AU15P quote in Phase 0.**
11. **Schematic.** Land pattern for the two FXBeam fields from the community footprints, checked pin by pin
    against the Electrical README (power field: 5V_ALW ×4, 3V3_ALW ×4, VADP_GPU ×6, 12V_FAN, Fan0/1 PWM and
    SPEED, I²C_ALW, GPIO0–3_EC, ID0; signal field: 4 PCIe lanes used of 8, REFCLK, PERST#, CLKREQ#, USB 2.0,
    DP pass-through untouched). AU15P; 2 GB DDR4 x16; QSPI flash; JTAG header; power tree from 5V_ALW via
    bucks for VCCINT 0.85 V, VCCAUX/VCCO 1.8 V, GTH 0.9/1.2/1.8 V, DDR4 1.2 V + VTT, sequenced per the AU+
    datasheet; 3V3_ALW for 3.3 V I/O only; VADP_GPU input stage footprint-only on spin 1. **Fan pass-through
    is mandatory** — the shell's two fans are the laptop's cooling. Rear connectors: the Phase 3 v1 LVDS link
    (HDMI-type), a SYZYGY-style or dual-SFP+-cage footprint for multi-gigabit tray links (populate later), SMA
    for PPS, status LEDs. XADC/SYSMON temperature readable from the host.
12. **Mechanical.** Outline from `bay/KEEPOUT.md`; heatsink directly between the fan outlets with a thermal pad
    to the shell floor; standoffs per the reference STL; 3D-print a dummy card with heatsink block and
    connectors to test fit in the extra shell before fab. Cut the rear I/O slot in that shell to match; print a
    bezel that doubles as strain relief.
13. **Layout.** 8–10 layers, controlled impedance, Gen4 lanes < 5 cm with via-in-pad/back-drill, DDR4 matched;
    design the lanes for Gen4 but configure the hard block for Gen3 fallback. Fab at a controlled-impedance
    house with BGA assembly (JLC 8-layer impedance class or similar); hand-solder the rest.
14. **Bring-up checklist** `bay/card/BRINGUP.md`: bench-supply power-up on a current-limited 5 V before the
    bay; rail sequencing and ripple; JTAG ID; flash; then in the bay: PCIe training (`LnkSta` — record Gen3 vs
    Gen4), DDR4 calibration, fans still EC-controlled, thermal soak, suspend/resume, then shell v0 and
    `omni-loopback`. Decide spin 2 scope from this list.

## Parts

| Part | Price | Note |
| --- | --- | --- |
| AUBoard 15P | $699 | Newark, 4-week lead |
| Dual M.2 Adapter + cheap 2230/2280 NVMe | $39 + ~$30 | Bay reference; buy with the laptop |
| Graphics Module Interposer ×2 | $29 ea | One for the Dual M.2 reference, one for the card |
| Threaded inserts ×4, SMT standoffs | free (Framework forms) | |
| 2× PCB spins, 8–10 layer, impedance-controlled, BGA assembly | ~$400 | |
| AU15P ×2, DDR4 ×2, regulators, connectors, passives | ~$500 | AU15P quote first |
| Copper heatsink stock, thermal pads | ~$40 | |
| M.2-to-PCIe x4 riser cable (optional, step 5) | ~$20 | |
| Inline USB-C PD meter, thermocouple logger | ~$55 | |

## Acceptance (gate to Phase 4 hardware)

- [ ] AUBoard: shell v0 loops back 1 h on the desktop at Gen4 x4, zero errors; `omni-loopback` builds from a
      clean checkout.
- [ ] `bay/KEEPOUT.md` and `bay/POWER.md` written with measured numbers; NVMe-in-bay reference documented.
- [ ] Card spin N enumerates in the FW16 bay at ≥ Gen3 x4; DDR4 passes memtest; fans still EC-controlled.
- [ ] 1 h loopback in the bay, zero errors, FPGA Tj < 85 °C, card ≤ 19 W measured, no CPU throttling beyond
      the no-card baseline.
- [ ] Suspend/resume works or the limitation is documented with a workaround.
- [ ] Rear slot cut; the LVDS link cable passes through with the shell closed.

## Open questions

- AU15P unit price and lead time — the one thing that can flip this phase to the Ti375.
- Does the bay's root port train Gen4 to a non-GPU endpoint with the stock BIOS? Step 5 answers it early if
  the riser hack works; otherwise spin 1 answers it.
- Is the bay's USB 2.0 pair worth routing to the rear as a free low-speed tray channel? Cheap to add on spin 1.
