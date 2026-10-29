# FPGA projects (project slot)

From the design doc's "Bay bring-up" and "Personal FPGA projects" sections. Each project is a bitstream for
the **project slot** behind the fixed shell (`gateware/shell/`), driven from the host through the `fw16-fpga`
crate. Nothing here starts on hardware before [Phase 2](../PLAN.md) reaches S5 (PCIe trains in the bay);
before that, projects run against the Hardcaml cyclesim and the `mock`/`sim` transports.

## Shell prerequisites (the doc's first milestones)

1. Shell enumerates and passes DMA loopback — Phase 2 S5.
2. Counter pattern streamed at full speed, throughput measured — `fw16-fpga-loopback` c2h at S5/S6.
3. **Reload the project slot without rebooting the laptop.** Not in Phase 2 yet. Two routes (doc risk
   "Reloading the FPGA"): a PCIe remove → reprogram → rescan flow in `fw16-fpga` (simple, drops the link),
   or partial reconfiguration of a slot region (keeps the link, needs a floorplanned shell and PR licensing
   check in Vivado ML Standard). Start with rescan; record a decision before PR work.
4. A project-slot interface in the shell (AXI-Stream in/out + an AXI-Lite window) so projects don't edit the shell.

## Projects, in order

| # | Project | Extra hardware | Starts when |
| --- | --- | --- | --- |
| 1 | Logic analyzer and oscilloscope (ThunderScope-style, laptop as display) | probe front end + input protection at the shell's rear (card spin 2 rear I/O) | logic analyzer in sim now; on hardware after S5 + spin 2 rear I/O |
| 2 | RISC-V soft CPU (LiteX SoC, boot Linux) | none | after S5 |
| 3 | HDL playground (Verilog, VHDL, Hardcaml, Amaranth) | none | after milestone 3 |
| 4 | Hardware accelerator (filters, compression, hashing, hls4ml) | none | after milestone 3 |
| 5 | Audio synth / DSP | audio codec board | rear I/O |
| 6 | Software-defined radio (ADS-B, FM, ham bands) | RF front end or SDR module, antenna | rear I/O |
| 7 | Retro computer/console cores (MiSTer ports from Cyclone V) | controller, audio, video out | rear I/O |
| 8 | Drive the laptop's own screen via the bay's DP lines | none; very ambitious | research only |

Spin 1 has **no rear I/O** (decision 0007), so projects 1 (analog side), 5, 6 and 7 wait for spin 2's rear
connectors; the logic analyzer's digital capture can use a JTAG-header-adjacent test header if spin 1 routes one.

## Per-project template

Each project gets `fpga/projects/<name>/` with its gateware, a host tool in `fpga/host/`, and its own
acceptance list here when it starts.
