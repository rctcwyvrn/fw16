AMD's XDMA example root-port testbench reaches into the example endpoint by hierarchical name, so swapping your own top in as `xilinx_dma_pcie_ep` fails elaboration unless those names exist.

- Referenced from `usp_pci_exp_usrapp_tx.v` and friends (Vivado 2025.2, xdma 4.2): `board.EP.m_axi_wdata/wstrb/wvalid/wready`,
  `board.EP.user_clk`, `board.EP.C_NUM_USR_IRQ`, `board.EP.usr_irq_req/ack`, `board.EP.xdma_0_i.*` (instance must be named `xdma_0_i`).
- Error seen: `VRFC 10-2991 'm_axi_wstrb' is not declared under prefix 'EP'`.
- Fix: in the sim-only EP copy, declare tied-off wires / a localparam with those names; the tasks that use them are never called by a custom test.
- The full flow (open_example_project → xsim to a BAR read) takes ~35 min wall on this machine; link-up is at ~187 µs sim time.
- `board.v` instantiates the EP with only the PCIe ports, so every other card input (e.g. `pps_in`) floats: Z → X in
  xsim, and X spreads into sampled registers such as `IRQ_PENDING`. Give each sampled card input a sim-only
  `pulldown`/`pullup` in the same shims (F3-gw, 2026-10-31).
