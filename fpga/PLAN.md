# Phase 2 — Bay bring-up on a custom AU15P card

Goal: a custom card sitting directly on the bay's FXBeam interposer with an `XCAU15P` in the FFVB676 package,
running PCIe Gen4 x4 (Gen3 accepted for spin 1), with a heatsink in the fan airstream and a rear I/O
connector, doing sustained DMA from a Hardcaml design. There is no drop-in board and no dev board (decisions
[0001](../docs/decisions/0001-tier1-board.md), [0007](../docs/decisions/0007-card-architecture.md)): gateware
is developed in simulation, a $39 Dual M.2 Adapter with an NVMe proves the bay side, and the first real PCIe
test is the card itself in the bay.

Two card branches, chosen in Phase 0 from the bay's measured clearance:

- **B — SOM carrier**: Puzhi `PZ-AU15P-SOM` on a 4–6 layer carrier. First silicon in the bay ~6–8 weeks
  after the SOM arrives.
- **A — minimal chip-down**: spin 1 is AU15P + power + PCIe + QSPI + JTAG + fan pass-through, no DDR4, no
  rear I/O; spin 2 adds them. First silicon ~2–3 months.

The work splits into **four parallel paths** — CAD, PCB, FPGA, software — that meet at six sync points.
Each path can be worked independently between sync points; nothing in one path should block on another
except where a sync point says so. Phase 3 (tray) and the step-0 CPU checks of Phases 4 and 7 also run in
parallel with this phase.

## Sync points

T0 = the Phase 0 branch decision (decision 0007 recorded). Weeks are rough, relative to T0, for branch B;
branch A adds ~4–8 weeks around S4.

| Sync | When | What crosses between paths |
| --- | --- | --- |
| **S1 — clearance known** | T0 (Phase 0) | CAD → PCB: vertical clearance, pad-field geometry, insert positions (`bay/KEEPOUT.md` v1). Picks branch B/A. |
| **S2 — outline frozen** | ~wk 2 | CAD → PCB: board outline, mounting holes, heatsink footprint and height, rear-slot position. PCB layout can start. |
| **S3 — pinout frozen** | ~wk 4 | PCB → FPGA: every FPGA pin assignment (XDC). PCB → software: power-telemetry and LED/GPIO map. FPGA → software: register map v0 (ABI). After S3 the schematic's FPGA pins don't move without a 3-way review. |
| **S4 — boards back** | ~wk 8–10 | PCB runs the bench stages. FPGA must have the *alive* bitstream (JTAG-only: LED blink, SYSMON readout, DDR4/BRAM self-test). Software must have the XDMA driver built and `omni-loopback` passing against `mock`. |
| **S5 — PCIe trains in the bay** | ~wk 10–12 | PCB → everyone: `LnkSta` result (Gen3/Gen4, width). FPGA → software: PCIe bitstream in flash. Software captures enumeration, suspend/resume, loopback. |
| **S6 — thermal soak result** | ~wk 12 | Software → CAD/PCB: Tj, surface temps, fan RPM, rail draw under 1 h loopback. CAD iterates the heatsink; PCB scopes spin 2. |

## Path 1 — CAD (mechanical)

Owner artefacts: `bay/KEEPOUT.md`, `bay/cad/` (Fusion 360; the `.f3d` lives in Fusion's cloud, and STEP + outline DXF exports are committed at each sync point), `bay/cad/stl/`, `docs/photos/`.

1. **Keepout from the STEP files** (Phase 0 step 5, repeated here as the entry): open
   `Mechanical/ExpansionBayShell/FW_UMA_MODULE_REF_0.stp` and `PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.stp`;
   record outline, FXBeam pad fields (two 74-pin fields, 16.65 mm apart; 27.75 mm is the connector length), insert positions, fan footprints and
   duct zones, max heights above and below the PCB, the rear wall's free area. → **S1**
2. **Verify against the physical shell** once the laptop and extra shell arrive: calipers on the open area,
   check the STEP against reality, note the Dual M.2 Adapter's own component heights as a sanity reference.
   Update `KEEPOUT.md` v2; diff the ExpansionBay repo against the clone date.
3. **Card envelope model**: a Fusion 360 body for the card outline with mounting holes, the SOM stack (B) or the
   BGA + tallest components (A), and a heatsink block sized to the duct gap. Export the outline DXF and the
   hole pattern for KiCad. → **S2**
4. **Dummy card**: 3D-print the envelope (PCB thickness slab + heatsink block + connectors as blocks) and fit
   it in the extra shell with the interposer and inserts. Photograph; fix the model until it drops in with
   ≥ 1 mm clearance everywhere.
5. **Heatsink**: design for the fan outlet gap — fin direction along the airflow, thermal pad to the FPGA, a
   second pad path to the shell floor if the gap allows; mounting via the card's holes, not adhesive. Spec a
   machined copper or aluminium piece (or a cut-down off-the-shelf heatsink) and a thermocouple pocket.
6. **Rear slot and bezel**: template the rear wall from `KEEPOUT.md`; cut the slot for the rear connectors + SMA
   in the extra shell; print a bezel that doubles as strain relief. Done after S3 fixes the connector set.
7. **Standoffs**: print `fw_expansion_bay_standoff_1_long.stl` as a stopgap; swap to Framework's SMT standoffs
   when they arrive.
8. **Thermal fixture**: thermocouple placements (FPGA lid, heatsink fin root, shell exterior over the card,
   fan inlet) and a repeatable mounting so S6 numbers are comparable across spins. After S6, iterate 5.

## Path 2 — PCB (card electronics)

Owner artefacts: `bay/card/` (KiCad), `bay/POWER.md`, `bay/card/BRINGUP.md`, `bay/lib/` (footprints, git-ignored).

1. **Footprint verification**: load the community FXBeam pad footprints
   ([XPModder](https://github.com/XPModder/Framework-ExpansionBay), [jyancat](https://github.com/jyancat/ExpansionBay));
   check both fields pin by pin against the Electrical README — power field: 5V_ALW ×4, 3V3_ALW ×4, VADP_GPU ×6,
   12V_FAN, Fan0/1 PWM and SPEED, I²C_ALW, GPIO0–3_EC, ID0; signal field: PCIe lanes 0–3 (of 8), REFCLK, PERST#,
   CLKREQ#, USB 2.0, DP pass-through. Note the lane crossover the community reports. Record the check in
   `bay/README.md`.
2. **Power budget and tree** (`bay/POWER.md`): 5V_ALW ≤ 2.5 A, 3V3_ALW ≤ 2.0 A, 12V_FAN ≤ 0.75 A pass-through,
   VADP_GPU footprint-only. All FPGA rails from 5V_ALW via bucks; 3V3_ALW for 3.3 V I/O only; current-sense on
   the 5 V input readable by the FPGA (INA2xx on I²C) for software telemetry. Measure the laptop's actual
   draw with the Dual M.2 + NVMe reference once it exists and update.
3. **Schematic — common**: pad fields, bucks, **fan pass-through (mandatory — the shell fans are the laptop's
   cooling)**, PERST#/CLKREQ# handling, QSPI, JTAG header, SMA for PPS, status LEDs, SYSMON/temperature
   readable by the host, USB 2.0 routed to a rear pad (cheap, optional).
4. **Schematic — branch B (carrier)**: the SOM's 168-pin B2B connector per its pinout; 4 GTH lanes + REFCLK
   from the connector to the pads; SOM power input per its spec; the Phase 3 rear connectors (LVDS/HDMI-type,
   SYZYGY-style or dual-SFP+ footprint) on spin 1.
   **Schematic — branch A (chip-down spin 1)**: `XCAU15P-2FFVB676E`, power sequencing per the AU+ datasheet
   (VCCINT 0.85 V, VCCAUX/VCCO 1.8 V, GTH 0.9/1.2/1.8 V), 4 GTH lanes to the pads, no DDR4, no rear data
   connectors. → **S3** (freeze the FPGA pinout; hand the XDC and GPIO map over).
5. **Layout**: outline and holes from CAD (S2). B: 4–6 layers, controlled impedance, lanes as short as the
   geometry allows. A: 8–10 layers, Gen4-capable lanes (< 5 cm, via-in-pad/back-drill), hard block configured
   for Gen3 fallback. Post-layout SI sanity: lane length match, reference-plane continuity, stub lengths.
6. **Fab and assembly**: controlled-impedance house; A needs BGA assembly, B is hand-solderable apart from the
   B2B connector (consider having the fab place it). Order the SOM (B) or the AU15P pair (A) to arrive with
   the boards. → **S4**
7. **Bench bring-up** (`BRINGUP.md`, staged so the board is proven alive before PCIe is attempted): bench
   supply, current-limited 5 V → rail sequence and ripple → JTAG ID → alive bitstream from the FPGA path →
   flash. Only then into the bay. → **S5**
8. **Spin 2 scoping** from the S5/S6 findings: A adds 2 GB DDR4 x16 and the rear connectors; B fixes
   whatever S5 found. Keep a defect log in `BRINGUP.md`.

## Path 3 — FPGA (gateware)

Owner artefacts: `gateware/shell/`, `gateware/vivado/*.tcl`, `gateware/sim/`.

1. **Shell v0 in Hardcaml**: AXI-Lite register file (ID, version, scratch, status, SYSMON temp, power
   telemetry); AXI-Stream source (counter/PRBS at a programmable rate) and sink (checker with error counter);
   a ring buffer behind a memory-interface abstraction so DDR4 (B, A spin 2) and BRAM (A spin 1) are
   interchangeable. OCaml testbench with `hardcaml_waveterm` traces checked in.
2. **Register map v0** as a generated artefact (OCaml → C header + Markdown) so software and gateware share
   one source. → **S3** (ABI frozen with software).
3. **XDMA wrapper**: Tcl-scripted Vivado project, XDMA IP at Gen4 x4 / 256-bit AXI at 250 MHz with Gen3
   fallback, wrapping the shell. Target `xcau15p-ffvb676-2-e` (A) or `-2-i` (B). Run the XDMA example design's
   PCIe root-port BFM simulation against the shell to prove the AXI side before any board exists.
4. **Timestamp and interrupts**: 64-bit free-running counter with host-readable latch and a 1-PPS input,
   simulated with a PPS stimulus; MSI-X on sink-done.
5. **Constraints**: XDC from the PCB pinout (S3); clocking from REFCLK; GTH placement for the 4 lanes;
   SYSMON enable. Keep the XDC generated from one pin table that the PCB path also uses.
6. **Alive bitstream**: JTAG-only image — LED blink, SYSMON temperature over the JTAG-to-AXI bridge, memory
   self-test (BRAM or DDR4). Ready before boards return. → **S4**
7. **PCIe bitstream**: shell v0 + XDMA for flash; verify in the bay with software's loopback. → **S5**
8. **Post-S6 tuning**: if Gen4 fails to train, lock Gen3 and record; if it trains, run the loopback at Gen4
   and record margin (link retrains, correctable errors from `lspci -vv`/AER).

## Path 4 — Software (driver, host, measurement)

Owner artefacts: `driver/`, `host/`, `benchmarks/bay/`, `docs/thermal/`.

1. **Bay reference tests**: scripts that record `lspci -vv` (BDF, `LnkCap`/`LnkSta`), `dmesg` on enumerate,
   and a suspend/resume cycle test — run first on the Dual M.2 + NVMe reference the day the laptop arrives;
   the same scripts later test the card. Store results in `benchmarks/bay/`.
2. **XDMA driver**: build `dma_ip_drivers/XDMA/linux-kernel` against the laptop's kernel (DKMS or Nix
   package), udev rules for the char devices, a `driver/README.md` with the exact steps.
3. **`omni_transport`** (C): `open/read_regs/write_regs/dma_read/dma_write/wait_irq`; backends `local_xdma`
   and `mock` (in-process ring buffer behaving like the shell). Register definitions consumed from the FPGA
   path's generated header. → **S3**
4. **`omni-loopback`** CLI: streams for N seconds at a target rate, reports GB/s, error count, interrupt
   latency; passes against `mock` in CI before S4. → **S4**
5. **Measurement tooling**: logging for the inline USB-C PD meter (or `/sys/class/power_supply` on battery),
   `sensors`/thermocouple logger, fan RPM via `framework_tool`/`ectool`, FPGA SYSMON via `omni_transport`;
   one script produces the S6 report (`docs/thermal/<date>-<spin>.md`).
6. **In-bay validation** at S5: enumerate, suspend/resume, 1 h loopback on mains and battery, AER/correctable
   error counts; write the result into the Phase 2 acceptance list. → **S6**
7. **Hand-off to Phase 4**: `omni_transport` is the library the cytometer app builds on; freeze v0 ABI.

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

- [ ] **FPGA/software:** shell v0 passes its OCaml testbench and the XDMA BFM simulation; `omni-loopback`
      builds from a clean checkout and passes against `mock`.
- [ ] **CAD/PCB:** `bay/KEEPOUT.md` and `bay/POWER.md` written with measured numbers; dummy card fits the
      shell; NVMe-in-bay reference documented.
- [ ] **PCB:** card enumerates in the FW16 bay at ≥ Gen3 x4; memory (B, or A spin 2) passes memtest; fans
      still EC-controlled.
- [ ] **All:** 1 h loopback in the bay, zero errors, FPGA Tj < 85 °C, card ≤ 19 W measured, no CPU throttling
      beyond the no-card baseline (S6 report in `docs/thermal/`).
- [ ] **Software:** suspend/resume works or the limitation is documented with a workaround.
- [ ] **CAD:** rear slot cut; the LVDS link cable passes through with the shell closed (A: after spin 2).

## Open questions

- Branch B: SOM stack height, connector part number and pinout quality — all from the Phase 0 quote.
- Does the bay's root port train Gen4 to a non-GPU endpoint with the stock BIOS? S5 answers it.
- Is the bay's USB 2.0 pair worth routing to the rear as a free low-speed tray channel? Cheap to add.
