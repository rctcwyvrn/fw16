# Shared settings for the Vivado flow. run.sh copies gateware/vivado/ plus the
# generated omni_shell.v into <build>/src on a Windows-local disk; every path
# here is relative to that copy, so Vivado never reads or writes the WSL share.
#
#   <build>/src/      these scripts, omni_top.v, omni_top.xdc, omni_shell.v
#   <build>/omni/     the project (create_project.tcl)
#   <build>/reports/  synth reports, xdma_0 property dump
#   <build>/example/  XDMA example design + root-port BFM sim (sim_bfm.tcl)

set omni_src     [file normalize [file dirname [info script]]]
set omni_build   [file dirname $omni_src]
set omni_part    xcau15p-ffvb676-2-e
set omni_proj    omni
set omni_projdir [file join $omni_build $omni_proj]
set omni_reports [file join $omni_build reports]
file mkdir $omni_reports
