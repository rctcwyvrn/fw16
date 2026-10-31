# create_project: the fw16_fpga_top project for xcau15p-ffvb676-2-e with xdma_0,
# the Hardcaml shell (fw16_fpga_shell.v, emitted by run.sh) and the wrapper.
# Recreates <build>/fw16_fpga from scratch every run; nothing in it is source.

source [file join [file dirname [info script]] settings.tcl]

create_project -force $fw16_fpga_proj $fw16_fpga_projdir -part $fw16_fpga_part
set_property target_language Verilog [current_project]
set_property default_lib xil_defaultlib [current_project]

source [file join $fw16_fpga_src xdma_0.tcl]

add_files -norecurse [list \
  [file join $fw16_fpga_src fw16_fpga_shell.v] \
  [file join $fw16_fpga_src fw16_fpga_top.v]]
add_files -fileset constrs_1 -norecurse [list \
  [file join $fw16_fpga_src fw16_fpga_top.xdc] \
  [file join $fw16_fpga_src fw16_fpga_pins.xdc]]
set_property top fw16_fpga_top [current_fileset]
update_compile_order -fileset sources_1

puts "fw16-fpga: project at $fw16_fpga_projdir"
close_project
