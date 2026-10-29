# Shared settings for the Vivado flow. run.sh copies gateware/vivado/ plus the
# generated fw16_fpga_shell.v into <build>/src on a Windows-local disk; every path
# here is relative to that copy, so Vivado never reads or writes the WSL share.
#
#   <build>/src/      these scripts, fw16_fpga_top.v, fw16_fpga_top.xdc, fw16_fpga_shell.v
#   <build>/fw16_fpga/     the project (create_project.tcl)
#   <build>/reports/  synth reports, xdma_0 property dump
#   <build>/example/  XDMA example design + root-port BFM sim (sim_bfm.tcl)

set fw16_fpga_src     [file normalize [file dirname [info script]]]
set fw16_fpga_build   [file dirname $fw16_fpga_src]
set fw16_fpga_part    xcau15p-ffvb676-2-e
set fw16_fpga_proj    fw16_fpga
set fw16_fpga_projdir [file join $fw16_fpga_build $fw16_fpga_proj]
set fw16_fpga_reports [file join $fw16_fpga_build reports]
file mkdir $fw16_fpga_reports
