# pinplan_check: apply fpga/bay/card/fpga/pinplan.csv to fw16_fpga_top's synthesized netlist and let Vivado judge it
# (I/O DRC + quick placement, which rejects GT/refclk LOCs the XDMA IP can't reach).
# Usage: vivado -mode batch -source pinplan_check.tcl -tclargs <fw16_fpga_top.dcp> <pinplan.csv> <report.txt> <xdma_0.dcp>
# The run's top checkpoint holds the out-of-context XDMA IP as a black box; <xdma_0.dcp> (from
# fw16_fpga.runs/xdma_0_synth_1/) fills it so opt/place can run.
lassign $argv dcp csv rpt ipdcp
open_checkpoint $dcp
read_checkpoint -cell [get_cells xdma_0_i] $ipdcp
set f [open $csv r]; gets $f; set plan {}
while {[gets $f line] >= 0} { lassign [split $line ,] net ball; dict set plan $net $ball }
close $f
# fw16_fpga_top port → pin-plan net (the PCB names); only ports fw16_fpga_top has today.
set map {sys_clk_p GT_REFCLK0_P sys_rst_n PEX_RST#}
for {set i 0} {$i < 4} {incr i} {
  lappend map "pci_exp_rxp[$i]" GT_RX${i}_P "pci_exp_txp[$i]" GT_TX${i}_P
}
foreach {port net} $map {
  set_property PACKAGE_PIN [dict get $plan $net] [get_ports $port]
  puts "fw16: $port -> $net [dict get $plan $net]"
}
set_property IOSTANDARD LVCMOS33 [get_ports sys_rst_n]
report_drc -file $rpt
opt_design
place_design -directive RuntimeOptimized
report_drc -append -file $rpt
foreach c [lsort [get_cells -hier -filter {REF_NAME == GTHE4_CHANNEL}]] { puts "fw16: GT [get_property LOC $c]" }
puts "fw16: pinplan_check PASS"
