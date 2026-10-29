# Register map v0.2 — proposal from the software path (Path 4)

**Date:** 2026-10-28 · **For:** Path 3 (`gateware/regmap/map_v0.ml`, XDMA IP config) · **Status:** landed (ab9cd540)

Software drives the card from userspace through VFIO ([decision 0009](decisions/0009-host-driver.md)), so it
programs the XDMA engine itself and needs a few things fixed in the IP configuration as well as in the
register map. Each item says what software assumes until Path 3 answers; the mock backend implements v0.2
as written here.

## 1. Interrupt block — new, `IRQ` at 0x5000

v0 has no interrupt registers, but Path 4 has `wait_irq` and `fw16-fpga-loopback` reports interrupt latency.

| Offset | Name | Access | Description |
| --- | --- | --- | --- |
| 0x5000 | `IRQ_EN` | RW | Per-source enable. `[0]` FORCE · `[1]` PPS · `[2]` SNK_ERR |
| 0x5004 | `IRQ_PENDING` | RW1C | Latched sources, same bits; write 1 to clear. |
| 0x5008 | `IRQ_COUNT` | RO | Interrupts raised since reset (wraps). |
| 0x500C | `IRQ_FORCE` | WO (pulse) | Bit 0: set `IRQ_PENDING.FORCE` now — the latency test source. |

The OR of `IRQ_PENDING & IRQ_EN` drives XDMA `usr_irq_req[0]`; the shell completes the `usr_irq_ack`
handshake. Needs a new access kind `RW1C` in `reg.ml`.

**Open:** the plan says "MSI-X on sink-done", but the sink has no notion of done. Either add
`SNK_TARGET_LO/HI` (raise `SNK_DONE` when `SNK_BEATS` reaches it) or drop sink-done — software is fine either way.

## 2. 64-bit counters — no torn reads

`SRC_BEATS_LO/HI` and `SNK_BEATS_LO/HI` have no latch. Proposal: a read of `*_LO` copies the live high
word into a shadow, and `*_HI` returns the shadow. Read order LO then HI is then always consistent. Only
the descriptions change; the offsets stay. Until then software uses hi–lo–hi retry reads.

## 3. XDMA IP configuration software relies on

- **BARs:** BAR0 = PCIe-to-AXI-Lite master (the shell's register map; 64 KiB covers blocks 0x0000–0xFFFF),
  BAR1 = XDMA DMA/config BAR. No bypass BAR. (AMD's driver finds the config BAR by its 0x1FC identifier and
  takes the BAR before it as the user BAR — same layout.)
- **Channels:** 1 H2C + 1 C2H, AXI-Stream mode, 256-bit at 250 MHz (as in `axis.ml`).
- **Interrupts:** MSI-X, `usr_irq` width 1. The vector assignment is programmed by software through XDMA's
  IRQ block registers, so it is not a gateware decision.
- **IDs:** vendor 0x10EE with whatever device ID the IP config produces, **subsystem vendor/device
  0x10EE / 0x4650 ('FP')** — the NixOS rule binds to `vfio-pci` on the subsystem ID, so the device ID can
  change between Gen3/Gen4 builds without a driver change.

## 4. Loopback dataflow — please confirm

Software's reading of `RING_CTL.LOOPBACK = 1`: H2C beats are teed to the sink (which checks them) and into
the ring; C2H drains the ring; the source is idle. With `LOOPBACK = 0`: source → ring → C2H, and H2C goes
only to the sink. `fw16-fpga-loopback`'s three modes (c2h, h2c, loop) are built on this reading.
