# sim_bfm: AMD's XDMA example design + its PCIe root-port BFM (xsim), with the
# example's endpoint swapped for omni_top. Passes when the link comes up and a
# BAR0 read at offset 0 returns the shell's ID, 0x4F4D4E49 ("OMNI").
#
# How the swap works (the example's own files are regenerated each run):
#   1. open_example_project for xdma_0 into <build>/example.
#   2. Disable the example endpoint (xilinx_dma_pcie_ep.*); add omni_shell.v
#      and a copy of omni_top.v renamed to xilinx_dma_pcie_ep, so the
#      testbench's EP instance and any EP.xdma_0_i references still resolve.
#   3. Patch the testbench's sample_tests.vh so its test chain starts with an
#      unconditional omni test (TESTNAME is then irrelevant).
# The written-blind parts are the BFM task names in omni_test; if
# elaboration fails on them, compare with the tasks in pci_exp_usrapp_tx.v.

source [file join [file dirname [info script]] settings.tcl]

set omni_exdir [file join $omni_build example]
set omni_magic 32'h4F4D4E49

proc slurp {path} { set f [open $path r]; set s [read $f]; close $f; return $s }
proc spit {path s} { set f [open $path w]; puts -nonewline $f $s; close $f }

open_project [file join $omni_projdir $omni_proj.xpr]
open_example_project -force -in_process -dir $omni_exdir [get_ips xdma_0]

# 2. Endpoint swap.
set ep_files [get_files -quiet -of_objects [get_filesets sources_1] *xilinx_dma_pcie_ep.*]
if {![llength $ep_files]} { error "sim_bfm: no xilinx_dma_pcie_ep source in the example project" }
set_property IS_ENABLED false $ep_files
set ep_copy [file join $omni_exdir omni_ep.v]
set n [regsub {module\s+omni_top\s*\(} [slurp [file join $omni_src omni_top.v]] \
         "module xilinx_dma_pcie_ep (" ep_text]
if {$n != 1} { error "sim_bfm: could not rename omni_top in omni_top.v" }
spit $ep_copy $ep_text
add_files -norecurse [list [file join $omni_src omni_shell.v] $ep_copy]

# The testbench's EP port list, logged so a port mismatch is easy to read.
set board [lindex [get_files -quiet -of_objects [get_filesets sim_1] */board.v] 0]
if {$board ne "" && [regexp {xilinx_dma_pcie_ep[^;]*;} [slurp $board] ep_inst]} {
  puts "omni: board.v EP instance:\n$ep_inst"
}

# 3. The omni test.
set tests [lindex [get_files -quiet -of_objects [get_filesets sim_1] *sample_tests*.vh] 0]
if {$tests eq ""} { error "sim_bfm: sample_tests.vh not found in sim_1" }
set omni_test [string map [list @MAGIC@ $omni_magic] {if (1'b1) begin
    board.RP.tx_usrapp.TSK_SIMULATION_TIMEOUT(10050);
    board.RP.tx_usrapp.TSK_SYSTEM_INITIALIZATION;
    $display("[%t] OMNI_BFM: link up", $realtime);
    board.RP.tx_usrapp.TSK_BAR_INIT;
    board.RP.tx_usrapp.TSK_TX_BAR_READ(3'd0, 32'h0, board.RP.tx_usrapp.DEFAULT_TAG, board.RP.tx_usrapp.DEFAULT_TC);
    board.RP.tx_usrapp.TSK_WAIT_FOR_READ_DATA;
    $display("[%t] OMNI_BFM: BAR0[0x0] = %h", $realtime, board.RP.tx_usrapp.P_READ_DATA);
    if (board.RP.tx_usrapp.P_READ_DATA == @MAGIC@)
      $display("[%t] OMNI_BFM: PASS", $realtime);
    else
      $display("[%t] OMNI_BFM: FAIL", $realtime);
    $finish;
  end else }]
set n [regsub {if\s*\(\s*testname\s*==} [slurp $tests] "$omni_test&" patched]
if {$n != 1} { error "sim_bfm: no 'if (testname ==' in $tests to hook the omni test onto" }
spit $tests $patched

# 4. Run to completion.
set_property top board [get_filesets sim_1]
set_property -name xsim.simulate.runtime -value -all -objects [get_filesets sim_1]
# If the chain's first 'if (testname ==' is inline in pci_exp_usrapp_tx.v and the
# hook above lands on an 'else if', a name no real test matches still falls
# through to it. Harmless if the tb doesn't read the plusarg.
set_property -name xsim.simulate.xsim.more_options -value {-testplusarg TESTNAME=omni_bar0_id} \
  -objects [get_filesets sim_1]
update_compile_order -fileset sim_1
launch_simulation -simset sim_1 -mode behavioral
close_sim

set log [lindex [glob -nocomplain [file join $omni_exdir * *.sim sim_1 behav xsim simulate.log]] 0]
if {$log eq ""} { error "sim_bfm: simulate.log not found under $omni_exdir" }
file copy -force $log [file join $omni_reports bfm_simulate.log]
set lines [lsearch -all -inline [split [slurp $log] \n] *OMNI_BFM*]
puts "omni: BFM result\n  [join $lines "\n  "]"
if {[lsearch $lines {*OMNI_BFM: PASS*}] < 0} { error "sim_bfm: no PASS in $log" }
