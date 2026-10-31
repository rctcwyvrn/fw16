# fw16_fpga_top timing constraints. Pin LOCs and IOSTANDARDs are in fw16_fpga_pins.xdc, generated
# from bay/card/fpga/pinplan.csv. XDMA's own XDC constrains its GT and user clocks (axi_aclk = 250 MHz).

create_clock -name sys_clk -period 10.000 [get_ports sys_clk_p]
set_false_path -from [get_ports sys_rst_n]
create_clock -name clk100m -period 10.000 [get_ports clk100m]
# Slow/static card I/O: LEDs, thermal drives, released I2C/CLKREQ, unused inputs.
set_false_path -to [get_ports {led_hb led_link th_alert th_overt sense_scl sense_sda clkreq_n}]
set_false_path -from [get_ports {sense_alert_n pps_in sense_scl sense_sda clkreq_n}]
