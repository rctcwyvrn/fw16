# gateware

Hardcaml designs. `shell/` is the reusable part (PCIe/DMA wrapper, registers, timestamp, trigger matrix, event packer); `apps/<instrument>/` are the per-instrument cores. Vivado is driven by Tcl scripts only; no GUI project state is committed. Pin the Vivado version here once chosen.
