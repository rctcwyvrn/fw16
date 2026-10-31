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
