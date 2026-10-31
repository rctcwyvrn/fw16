# On the AU15P (PCIE4C_X0Y0, GTH Quad 225), XDMA puts PCIe lane n on channel 3−n, and a conflicting PACKAGE_PIN is silently dropped

The IP's `xdma_0_pcie4c_ip_gt.xdc` LOCs gtwizard `inst[k]` to `GTHE4_CHANNEL_X0Y(4+k)`, and PCIe lane 0 is driven by
`inst[3]`, so lane 0 is on X0Y7 = channel 3 (RX P2, TX R5). The `lane_order` option (Bottom/Top) made no difference
here. A top-level PACKAGE_PIN that asks for another order only raises CRITICAL WARNING 12-1411 ("bel is occupied")
and the IP's placement wins, so synthesis, placement and the BFM sim all still pass while the board is lane-reversed.
Found 2026-10-31. Never trust "GT sites are right" alone: after placement, check
`get_property PACKAGE_PIN [get_ports pci_exp_rxp[n]]` against the pin table (`power.tcl` and `pinplan_check.tcl` now do).
Also, `reset_run synth_1` reuses the out-of-context IP netlist: reset `xdma_0_synth_1` too when testing an IP setting.
