# power: place the synthesized design with the card's pins (fw16_fpga_pins.xdc) and estimate power.
# Placement doubles as the pin-plan check (I/O DRC, GT/refclk LOCs the XDMA IP can reach). The power
# numbers use vectorless default activity (12.5 % toggle, 0.5 static probability) on top of the
# clocks the XDC declares; a first budget for core_power, not a sign-off. Run after synth.

source [file join [file dirname [info script]] settings.tcl]

open_project [file join $fw16_fpga_projdir $fw16_fpga_proj.xpr]
open_run synth_1
opt_design
place_design -directive RuntimeOptimized
report_drc -file [file join $fw16_fpga_reports place_drc.txt]
report_io -file [file join $fw16_fpga_reports place_io.txt]

set_operating_conditions -ambient_temp 50 -airflow 0 -heatsink none
set_switching_activity -default_toggle_rate 12.5 -default_static_probability 0.5
report_power -file [file join $fw16_fpga_reports place_power.txt]
puts "fw16-fpga: placed DRC, IO and power reports in $fw16_fpga_reports"
close_project
