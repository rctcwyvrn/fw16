# Phase 2 — Bay bring-up

Goal: the Aller A7 running inside the Expansion Bay Shell on the Dual M.2 Adapter, doing sustained DMA to the
host from a Hardcaml design, with the shell's thermal and electrical headroom measured and the rear I/O slot
cut. This is the first thing that proves the whole premise.

## Steps

### Mechanical and electrical survey
1. **Record the keepouts.** Open `PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.pdf`, `.dxf` and `.stp`, and
   `Mechanical/ExpansionBayShell/FW_UMA_MODULE_REF_0.pdf`/`.stp` from the ExpansionBay repo in FreeCAD/KiCad.
   Write `bay/KEEPOUT.md`: PCB outline, interposer positions (two 74-pin FXBeam, 27.75 mm), fan footprints,
   max component height above/below the PCB, the open middle area's x/y/z, and the rear wall's free area for
   I/O. This closes the doc's first open question.
2. **Fit check.** Install the Dual M.2 Adapter on the interposer in the extra shell; seat the Aller with its
   heatsink; confirm clearance to the shell lid and fan ducts. If the heatsink fouls, model a lower-profile one
   in `bay/` and print a test fit before buying copper.
3. **Rail survey.** With a bench supply or inline USB-C PD meter, measure laptop system draw idle vs Aller
   idle vs Aller under a heater bitstream. Separately, break out the adapter's 3.3 V with a current shunt to
   learn whether it comes from 3V3_ALW (≤ 2.0 A shared) or a regulator off 5V_ALW (≤ 2.5 A). Write the numbers
   into `bay/POWER.md`.

### PCIe and DMA
4. **First enumeration.** Boot with Numato's sample bitstream in SPI flash. Expect `lspci` to show the device
   behind the bay's PCIe switch/root at Gen2 x4. Record BDF, link state, and whether the device survives
   suspend/resume (`systemctl suspend`; check `LnkSta` after).
5. **Choose the DMA stack** and write `docs/decisions/0002-dma-stack.md`. Two candidates, prototype both in a
   day each on the desktop adapter if the laptop is late:
   - **AMD XDMA IP + `dma_ip_drivers`**: fastest to first DMA on Artix-7; proprietary IP but free; matches
     the doc's "swap AWS shell for XDMA" plan. Verilog wrapper around Hardcaml logic.
   - **LiteX + LitePCIe**: open, has a 7-series PHY wrapper, a kernel driver and Python host tools; litex-boards
     already has an `rhsresearch_nitefury`-style target to adapt for the Aller. More moving parts.
   Recommendation: XDMA first (de-risks Phase 4 timing), keep the Hardcaml app core transport-agnostic so
   LitePCIe can replace it later.
6. **Hardcaml shell v0** in `gateware/shell/`: AXI-Lite register file (ID, version, scratch, status), one
   AXI-Stream source that generates a counter/PRBS at a programmable rate, one sink that checks it. Export
   Verilog via `Hardcaml.Rtl`, wrap with the XDMA IP in a scripted Vivado project (`gateware/vivado/*.tcl`,
   no GUI state in git). Target `xc7a200tfbg484-2`, Gen2 x4, 128-bit AXI at 125 MHz.
7. **Host side** in `host/` and `driver/`: udev rule for the XDMA char devices, a C library `omni_transport`
   with `open/read_regs/write_regs/dma_read/dma_write` and a `local_xdma` backend; `omni-loopback` CLI that
   streams for N seconds and reports GB/s and error count. This library is the same one Phase 9 gives an AWS
   F2 backend.
8. **Sustained test.** 1 hour of loopback at max rate on battery and on mains; log FPGA temperature (XADC),
   CPU package temp, fan RPM (via `ectool`/`framework_tool`), DMA throughput, error count. Expect ~1.2–1.6 GB/s
   on Gen2 x4.
9. **Interrupts and timestamps.** Add MSI-X interrupt on sink-done; add a free-running 64-bit timestamp
   counter with a host-readable latch, and a 1-PPS input stub (for the Phase 3 GPS module) — this is the
   first "service every instrument gets".

### Rear I/O slot
10. **Cut the slot** in the extra shell's rear wall per `bay/KEEPOUT.md`: sized for the Phase 3 cable choice
    (see that file), plus one SMA hole for 1-PPS. Deburr; print a bezel that also acts as a cable strain
    relief. Photos to `docs/photos/`.

## Parts

| Part | Price | Note |
| --- | --- | --- |
| Inline USB-C PD power meter | ~$25 | Whole-system draw |
| Current shunt / clamp for 3.3 V | ~$20 | Or a bench supply with readout |
| Thermocouple logger | ~$30 | Heatsink and shell surface temps |
| Rotary tool / nibbler, files | have | Slot cutting |

## Acceptance (gate to Phase 3 and 4)

- [ ] `bay/KEEPOUT.md` and `bay/POWER.md` written with measured numbers.
- [ ] Hardcaml shell v0 + XDMA loops back for 1 h, ≥ 1 GB/s, zero errors, FPGA Tj < 85 °C, no CPU throttling
      beyond the no-FPGA baseline.
- [ ] Device enumerates after suspend/resume, or the limitation is documented with a workaround (cold plug).
- [ ] Rear I/O slot cut, bezel printed, cable passes through with the shell fully closed.
- [ ] `omni-loopback` builds and runs from a clean checkout.

## Open questions

- Does the FW16 EC or BIOS do anything with an unknown PCIe device in the bay (e.g. GPU power sequencing via
  GPIO0_EC/GPIO1_EC)? The Dual M.2 Adapter already proves plain endpoints are fine; confirm nothing fights the Aller.
- Does the bay's USB 2.0 HS pair reach the Dual M.2 Adapter? If a pad is accessible, it is a free side channel
  to tray modules without touching PCIe.
- 3.3 V budget: if the Aller alone needs > 1.5 A at load, the Phase 8 card must regulate from 5V_ALW.
