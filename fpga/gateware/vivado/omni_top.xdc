# omni_top timing constraints. No pin LOCs yet: the card's PCIe lane/refclk
# pins and PERST# ball come from the PCB (Path 2); synthesis doesn't need them.
# XDMA's own XDC constrains its GT and user clocks (axi_aclk = 250 MHz).

create_clock -name sys_clk -period 10.000 [get_ports sys_clk_p]
set_false_path -from [get_ports sys_rst_n]
