# create_project: the omni_top project for xcau15p-ffvb676-2-e with xdma_0,
# the Hardcaml shell (omni_shell.v, emitted by run.sh) and the wrapper.
# Recreates <build>/omni from scratch every run; nothing in it is source.

source [file join [file dirname [info script]] settings.tcl]

create_project -force $omni_proj $omni_projdir -part $omni_part
set_property target_language Verilog [current_project]
set_property default_lib xil_defaultlib [current_project]

source [file join $omni_src xdma_0.tcl]

add_files -norecurse [list \
  [file join $omni_src omni_shell.v] \
  [file join $omni_src omni_top.v]]
add_files -fileset constrs_1 -norecurse [file join $omni_src omni_top.xdc]
set_property top omni_top [current_fileset]
update_compile_order -fileset sources_1

puts "omni: project at $omni_projdir"
close_project
