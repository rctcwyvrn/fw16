# Project-slot interface — proposal from the gateware path (Path 3)

**Date:** 2026-10-31 · **For:** the user (sync point SP), then Path 4 (`fw16-fpga`) · **Status:** proposal, nothing built

[`projects/PLAN.md`](../projects/PLAN.md) "Shell prerequisites" 4 asks for a slot in the shell — AXI-Stream in
and out plus an AXI-Lite window — so a project is a module plugged into the shell, never an edit of it. This
document fixes nothing yet: each section gives the options with their cost, and a recommendation. The user picks;
the chosen options then become a decision record and a regmap change Path 4 reviews (the map is the S3 ABI).

## What exists today

- **Shell v0** (`gateware/shell/shell_top.ml`), one clock: XDMA's `axi_aclk`, 250 MHz; `clear` = `~axi_aresetn`
  (a PCIe link reset clears the whole shell).
- **Dataflow:** test source or H2C (`RING_CTL.LOOPBACK`) → BRAM ring → C2H; the checker always sees H2C.
  Streams are `Axis.t`: 256-bit `tdata`, 32-bit `tkeep`, `tlast`, no `tuser`.
- **BAR0** = XDMA AXI-Lite master, **64 KiB**, `awaddr` = BAR offset (`vivado/xdma_0.tcl`). The register
  decoder (`regs.ml`) decodes the full word address. Blocks in use: `0x0000`–`0x5FFF` (CTL, SRC, SNK, RING, TS,
  IRQ), one 4 KiB block each. BAR1 = XDMA config. **No bypass BAR** — Path 4 relies on that layout
  ([regmap v0.2 §3](regmap-v0.2-proposal.md)).
- **Timestamp** (`shell/timestamp.ml`): live 64-bit counter on `axi_aclk`, PPS edge pulse.
- **IRQ** (`shell/irq.ml`): one MSI-X line, sources FORCE/PPS/SNK_ERR in bits 0–2; bit 3 is free.
- **Free card clock:** `clk100m` (Y1, 100 MHz, bank 84), runs without PCIe; only the heartbeat uses it.

## 1. Address window

| Option | What changes | Gives | Costs |
| --- | --- | --- | --- |
| **A. Upper half of BAR0** — shell keeps `0x0000`–`0x7FFF`, slot owns `0x8000`–`0xFFFF` (32 KiB) | `regs.ml` routes addresses with bit 15 set to the slot's AXI-Lite port instead of decoding them; regmap gains a `SLOT` window entry (additive, no offset moves) | No XDMA or BAR change; F4's BAR0 mmap is unchanged; smallest diff | 32 KiB ceiling. Enough for register files and small mailboxes; not for a memory-mapped framebuffer or a soft CPU's RAM (those go over the streams) |
| **B. Grow BAR0** to e.g. 1 MiB, slot at `0x10000`–`0xFFFFF` (960 KiB) | `axilite_master_size` in `xdma_0.tcl`; same routing in `regs.ml` on a higher bit | Room for MM-heavy projects (LiteX CSR space, small RAMs, lookup tables) | Rebuild of the IP and BFM run; F4 must stop assuming 64 KiB; still AXI-Lite (single beats, ~1 µs per read over PCIe) |
| **C. Separate bypass BAR** (XDMA AXI-MM bypass master on BAR2) | XDMA config; a second, full-AXI4 port into the slot | Burst MM access, slot address space independent of the shell's | Biggest change: new BAR in F4's layout (regmap v0.2 §3 says "no bypass BAR"), AXI4-full on the slot boundary, more IP to verify; under partial reconfiguration a wider boundary to decouple |

**Recommendation: A now**, B later if a project actually needs MM space. A slot that needs bulk data uses the
streams, which is what they are for. With A the shell answers reads it doesn't decode with 0 (as today), and the
slot convention reserves its offset `0x8000` for a read-only `SLOT_ID` and `0x8004` for `SLOT_VERSION`, so the host
can tell which project is loaded. The shell-side controls (`SLOT_CTL.RESET`, later `DECOUPLE`) go in a new
`SLOT` block at `0x6000`, outside the window, so a project cannot shadow them.

## 2. Clock and reset

| Option | Gives | Costs |
| --- | --- | --- |
| **A. Slot on `axi_aclk`** (250 MHz), `slot_clear` = shell `clear` OR `SLOT_CTL.RESET` | No clock crossings: streams, AXI-Lite and the timestamp are plain wires. Simplest to simulate | Every project must close timing at 250 MHz on a -2 AU15P; a slow project must pipeline or run a clock enable. Clock stops when the link drops (fine: no host, nothing to serve) |
| **B. Slot on its own clock** (from `clk100m` through an MMCM, frequency chosen per project), async FIFOs for both streams and an AXI-Lite clock converter in the shell | Projects pick their clock (a soft CPU at 100–150 MHz, DSP at a sample-rate multiple); slot keeps running across link resets | Three CDC structures in the shell; the timestamp needs a crossing too (§4); more to verify; one MMCM fewer for projects |
| **C. Both**: the slot gets `axi_aclk` *and* a free `slot_clk`; interfaces are on `axi_aclk`, and a project that wants its own clock does the crossing inside the slot with a shell-provided library (async FIFO, AXI-Lite CDC) | Shell stays single-clock; the CDC cost is paid only by projects that want it | The CDC library has to exist and be tested before the first such project; projects can get CDC wrong inside the slot |

**Recommendation: C.** It keeps the shell as simple as A while not forcing 250 MHz on every project. Reset: the
slot gets `slot_clear` (synchronous, active-high, like the rest of the shell), asserted by the shell `clear` or by
writing `SLOT_CTL.RESET`, so the host can restart a project without a link reset.

## 3. Streams

| Option | Gives | Costs |
| --- | --- | --- |
| **A. 256-bit native `Axis.t`** at the boundary, both directions | Full XDMA bandwidth (8 GB/s at 250 MHz, at or above what Gen4 x4 carries after 128b/130b and TLP overhead); no shell-side converter; width fixed for partial reconfiguration | A small project carries 256-bit wires it doesn't use; it instantiates a width converter itself |
| **B. Narrow (32 or 64-bit)** at the boundary, converters in the shell | Easy for simple projects | Caps throughput at 1/8 or 1/4 of the link for everyone; converters in the static shell |
| **C. Parameterised width per project** | Each project gets exactly its width | Only works with the rescan route (whole design recompiled); impossible under PR, where the boundary is fixed |

**Recommendation: A**, plus `Axis_width` converters in a slot-side library. Add a 64-bit `tuser` sideband? Not in
v0: projects that want to timestamp data put the timestamp in the data (§4).

**Routing.** The ring stays the only path to C2H (it absorbs host back-pressure). Proposal: `RING_CTL` gains a
field `SRC_SEL` (2 bits, additive; `LOOPBACK` stays as bit 0 for compatibility, `SRC_SEL` = 2 selects the slot):

```
H2C ──┬─> checker
      ├─> slot s_axis   (slot m_axis ─┐)
      │                               ├─ mux (SRC_SEL) ─> ring ─> C2H
source ───────────────────────────────┤
H2C (LOOPBACK) ───────────────────────┘
```

H2C goes to the slot only while `SRC_SEL` = slot, so the existing loopback modes are untouched. Path 4's three
loopback modes keep working; a fourth "slot" mode is added.

## 4. Timestamp

The slot gets, as inputs: `ts_now[63:0]` (the live counter) and `pps_edge` (the one-cycle pulse the IRQ block
also sees). On `axi_aclk` (clock options A and C) these are direct wires and a project can stamp events with the
same time base as `TS_NOW`/`TS_PPS`. Under clock option B, or a project running its own clock inside the slot
(option C), it needs either:
- **a Gray-coded copy** of the counter crossed into the slot clock (the shell provides it; ±1 count of jitter), or
- **its own counter** in the slot clock, reset on the synchronised `pps_edge` (simpler, loses the shared epoch).

Projects that want both stamped data and PPS alignment use the first. The slot's IRQ: IRQ bit 3 (free today)
becomes `SLOT`, driven by a `slot_irq` level from the slot — an additive regmap change.

## 5. Rescan-reload vs partial reconfiguration

The interface above is the same in both routes; what differs is how firmly it is frozen and what extra
hardware the shell needs.

| | **Rescan-reload** (planned first, `projects/PLAN.md` prerequisite 3) | **Partial reconfiguration (DFX)** |
| --- | --- | --- |
| Reload flow | Host removes the PCIe device, loads a full bitstream (JTAG or flash + PROGRAM_B), rescans; link drops | Host loads a partial bitstream through MCAP/ICAP while the link and the shell stay up |
| Boundary is frozen at | Source level: a Hardcaml interface (`Slot.I`/`Slot.O`) or a Verilog module port list. Changing it means editing the shell, then every project recompiles | Netlist level: partition pins in a placed static design. Changing it means a new static shell and re-implementing every project against it |
| Build | Whole design (shell + project) synthesised and implemented together; any width/clock may be parameterised | Static shell implemented once, locked; each project implemented into a pblock against it |
| Shell needs | Nothing beyond the interface | A floorplanned pblock with enough resources; a **decoupler** (holds slot `tvalid`/`tready` low and makes slot-window reads return 0 or SLVERR during reload); `SLOT_CTL.DECOUPLE` and a reload-done status; clock buffers in the static region; MCAP or an ICAP controller + host flow |
| Constraints it imposes | None | Fixed stream width (§3 C is out), fixed clock set (§2 B needs its MMCM in static logic), no parameterised ports |
| Open questions | How the host drives JTAG/flash from the laptop (Path 4) | Whether Vivado ML Standard licenses DFX for the AU15P (check on this install before any design work); MCAP support in XDMA 4.2 for this part; resource cost of the pblock margin |

**Recommendation:** design the boundary now as if it will be frozen (fixed 256-bit streams, `axi_aclk` + one
free clock, the slot behind a single hierarchy boundary with `keep_hierarchy`), so moving to DFX later is a
floorplanning job rather than an interface change. Build only the rescan route until a decision record says
otherwise, as the plan already requires.

## The slot boundary, as it would look

A sketch for review, not code to land:

```ocaml
module Slot = struct
  module I = struct
    type 'a t =
      { clock : 'a            (* axi_aclk, 250 MHz *)
      ; slot_clk : 'a         (* free clock from clk100m, if §2 C *)
      ; clear : 'a            (* shell clear | SLOT_CTL.RESET *)
      ; axil : 'a Axi_lite.M2s.t   (* window 0x8000-0xFFFF, addresses relative to 0x8000 *)
      ; s_axis : 'a Axis.t    (* from H2C *)
      ; m_tready : 'a
      ; ts_now : 'a [@bits 64]
      ; pps_edge : 'a
      }
  end
  module O = struct
    type 'a t =
      { axil : 'a Axi_lite.S2m.t
      ; s_tready : 'a
      ; m_axis : 'a Axis.t    (* to the ring when RING_CTL.SRC_SEL = slot *)
      ; irq : 'a              (* level, IRQ bit 3 *)
      }
  end
end
```

`Shell_top.Make` would take the slot as a functor argument (alongside the memory), so a project build is
`Shell_top.Make (Bram_mem) (My_project)`, and the default build plugs in an empty slot (ID 0, streams idle).

## Questions for the user

1. Address window: A (32 KiB in BAR0), B (grow BAR0) or C (bypass BAR)?
2. Clocking: A (`axi_aclk` only), B (slot clock with shell CDC) or C (`axi_aclk` interfaces + a free clock,
   CDC inside the slot)?
3. Stream width at the boundary: 256-bit native (A) or narrow (B)?
4. Routing: is `RING_CTL.SRC_SEL` with the slot as a third ring source right, or should the slot replace the
   source/checker pair when present?
5. IRQ bit 3: give it to the slot, or keep it for the sink-done source regmap v0.2 left open?
6. Is the regmap change set (SLOT window, `SLOT_CTL.RESET`/`DECOUPLE`, `RING_CTL.SRC_SEL`, IRQ bit 3) acceptable
   as one v0.3 bump, reviewed with Path 4?
