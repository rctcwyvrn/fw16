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
# A PACKAGE_PIN that conflicts with the XDMA IP's own GT LOCs is dropped with only a CRITICAL WARNING (12-1411), so
# check that every port landed where fw16_fpga_pins.xdc (from pinplan.csv) put it.
set f [open [file join $fw16_fpga_src fw16_fpga_pins.xdc] r]; set bad 0
foreach line [split [read $f] \n] {
  if {[regexp {^set_property PACKAGE_PIN (\S+) \[get_ports \{(.+)\}\]} $line -> ball port]} {
    set got [get_property PACKAGE_PIN [get_ports $port]]
    if {$got ne $ball} { puts "fw16-fpga: MISMATCH $port placed on $got, pin XDC says $ball"; incr bad }
  }
}
close $f
if {$bad} { error "power: $bad port(s) not on their pin-plan ball" }
puts "fw16-fpga: every port is on its pin-plan ball"

set_operating_conditions -ambient_temp 50 -airflow 0 -heatsink none
set_switching_activity -default_toggle_rate 12.5 -default_static_probability 0.5
report_power -file [file join $fw16_fpga_reports place_power.txt]
puts "fw16-fpga: placed DRC, IO and power reports in $fw16_fpga_reports"
close_project
