# 0013 — Project-slot interface (sync point SP)

**Date:** 2026-10-31
**Status:** accepted 2026-10-31 (user, on [project-slot-proposal.md](../project-slot-proposal.md)); not built yet

## Context

`projects/PLAN.md` "Shell prerequisites" 4 needs a slot in the shell so a project plugs in without editing it. F3-gw's
proposal gave options for the address window, clocking, stream width, routing, the timestamp and the reload route.

## Decision

- **Register space: a separate bypass BAR** (XDMA AXI-MM bypass master, BAR2), a full AXI4 port into the slot with
  its own address space. Not the 32 KiB BAR0 window the proposal recommended. This reverses regmap v0.2 §3
  ("no bypass BAR"). The slot convention keeps `SLOT_ID` at bypass offset 0x0 and `SLOT_VERSION` at 0x4.
- **Clocking:** slot interfaces on `axi_aclk`, plus a free `slot_clk`. A project that wants its own clock crosses
  inside the slot with a shell-provided CDC library (async FIFO, AXI-Lite/AXI4 CDC). That library is built and tested
  before the first project that uses it.
- **Streams: 256-bit native `Axis.t`** both ways, with width converters in a slot-side library.
- **Regmap v0.3, one bump reviewed with F4:**
  - `RING_CTL.SRC_SEL` (2 bits; `LOOPBACK` stays bit 0; `SRC_SEL` = 2 makes the slot the ring source, and H2C goes to
    the slot only then).
  - IRQ bit 3 = `SLOT` (from a `slot_irq` level); "sink-done" stays dropped.
  - A `SLOT` block at 0x6000 in BAR0 with `SLOT_CTL.RESET` (later `DECOUPLE`), outside the project's reach.
  - Fixes the stale TS / IRQ-PPS doc strings in `map_v0.ml`.
- **Timestamp:** the slot gets `ts_now[63:0]` and `pps_edge` on `axi_aclk`. The counter keeps resetting with the
  link (`~axi_aresetn`); the user kept that, so time restarts on every link reset and rescan reload.
- **Reload:** the rescan route only, until a decision record says otherwise. The boundary is designed as if frozen:
  fixed widths, `axi_aclk` + one free clock, one hierarchy boundary with `keep_hierarchy`.

## Consequences

- `xdma_0.tcl` gains the bypass interface: rebuild the IP, extend `fw16_fpga_top.v`, rerun the BFM, and check that
  the bypass BAR enumerates. F4's BAR layout and `vfio` BAR mmap gain BAR2.
- Under a later DFX route, the AXI4 bypass port is part of the boundary to decouple, a wider one than an AXI-Lite
  window.
- **Added 2026-11-01 (user, on F4-sw's round-2 questions):**
  - **`SLOT_CTL.RESET` gates the slot's streams in the shell**: `s_axis.tvalid` into the slot and the slot's `tready`
    are ANDed with `~slot_clear`, so any project is silent in reset and RESET is the host's guaranteed way to stop
    slot traffic. F3 adds the two gates and a cyclesim test; F4's mock already stalls this way.
  - **The SLOT interrupt (bit 3) belongs to project code**: the crate's `irq::drain` leaves it alone, and a
    per-project handler services it, so a project holding `slot_irq` cannot block the shell's sources.
  - Claude's calls (status quo): BAR2 stays `vfio::Card::slot_bar()` only until a `sim` transport exists;
    `fw16-fpga-loopback --mode all` excludes `slot`; vfio-pci binding stays the udev `driver_override` rule.
- **Verified 2026-11-01.** Regmap v0.3 (c9949a39) and the slot boundary with `Slot_echo` (2ed87fcc) are from F3-gw.
  The bypass BAR (BAR2: 32-bit, 1 MiB, AXI4 to the slot) is F3-gw's `xdma_0.tcl`/`fw16_fpga_top.v`/`sim_bfm.tcl`,
  finished by the orchestrator after the worker's background Vivado run was killed at its turn end. A Tcl `regsub`
  `&` expansion in the BFM test splice was fixed. Vivado `all`: 0 critical warnings, every port on its pin-plan ball,
  setup WNS +0.728 ns, LUT 32.7 %, FF 17.0 %, BRAM 57.3 %, 2.97 W, Tj 69.0 °C. BFM: link up 187.5 µs,
  BAR0[0x0] = 0x46504741, BAR2[0x0] SLOT_ID = 0x4543484F ("ECHO"), BAR2[0x4] SLOT_VERSION = 0x00010000, PASS.
- **RESET stream gating verified 2026-11-01** (F3-gw, 04621656). `shell_top.ml` gates all four slot stream
  handshake signals with `~slot_clear`, outside the `fw16_fpga_slot` boundary: `s_axis.tvalid` and `m_tready` into
  the slot, `s_tready` and `m_axis.tvalid` out of it. The decision named two gates; the outgoing pair is added so a
  project that ignores `clear` cannot push a beat into the ring either. Cyclesim (`test_slot.ml`, `Slot_echo` and a
  clear-deaf one-beat pipe): with RESET held no H2C beat is taken and the slot emits nothing; after release all 8
  beats arrive in order. Vivado `all`: 0 critical warnings outside placement's expected RTSTAT-13, every port on
  its pin-plan ball, setup WNS +0.728 ns, LUT 32.7 %, FF 17.0 %, BRAM 57.3 %, 2.95 W, Tj 68.9 °C. BFM: link up
  187.5 µs, BAR0[0x0] = 0x46504741, BAR2[0x0] SLOT_ID = 0x4543484F, BAR2[0x4] SLOT_VERSION = 0x00010000, PASS.
- **Added 2026-11-01 (user, round 3):** SLOT keeps the shared vector with the crate's 1 ms polling fallback
  (`irq::HELD_POLL`); its own MSI-X vector (regmap v0.4) only if a project needs low-latency SLOT servicing. The
  RESET gating covers all four handshake signals (F3-gw's call, kept): `s_axis.tvalid`/`m_tready` into the slot and
  `s_tready`/`m_axis.tvalid` out, so even a project that ignores `clear` is silent in reset.
