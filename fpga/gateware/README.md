# gateware

Hardcaml designs. `shell/` is the reusable part (PCIe/DMA wrapper, registers, timestamp, trigger matrix, event packer); `apps/<instrument>/` are the per-instrument cores. Vivado is driven by Tcl scripts only; no GUI project state is committed.

**Vivado 2025.2** (Windows install at `F:\AMDDesignTools\2025.2`, driven from WSL), part `xcau15p-ffvb676-2-e`,
IP `xdma:4.2`. `vivado/run.sh project|synth|sim|all` emits the shell as Verilog (`shell/bin/emit_verilog.exe`),
copies `vivado/` to a Windows-local build dir (default `C:\Users\17786\fw16-fpga-build\vivado`) and runs the
matching Tcl there; reports land in `<build>/reports/`.
