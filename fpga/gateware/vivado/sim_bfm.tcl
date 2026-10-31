# sim_bfm: AMD's XDMA example design + its PCIe root-port BFM (xsim), with the
# example's endpoint swapped for fw16_fpga_top. Passes when the link comes up and a
# BAR0 read at offset 0 returns the shell's ID, 0x46504741 ("FPGA").
#
# How the swap works (the example's own files are regenerated each run):
#   1. open_example_project for xdma_0 into <build>/example.
#   2. Disable the example endpoint (xilinx_dma_pcie_ep.*); add fw16_fpga_shell.v
#      and a copy of fw16_fpga_top.v renamed to xilinx_dma_pcie_ep, so the
#      testbench's EP instance and any EP.xdma_0_i references still resolve.
#   3. Patch the testbench's sample_tests.vh so its test chain starts with an
#      unconditional fw16-fpga test (TESTNAME is then irrelevant).
# The written-blind parts are the BFM task names in fw16_fpga_test; if
# elaboration fails on them, compare with the tasks in pci_exp_usrapp_tx.v.

source [file join [file dirname [info script]] settings.tcl]

set fw16_fpga_exdir [file join $fw16_fpga_build example]
set fw16_fpga_magic 32'h46504741

proc slurp {path} { set f [open $path r]; set s [read $f]; close $f; return $s }
proc spit {path s} { set f [open $path w]; puts -nonewline $f $s; close $f }

open_project [file join $fw16_fpga_projdir $fw16_fpga_proj.xpr]
open_example_project -force -in_process -dir $fw16_fpga_exdir [get_ips xdma_0]

# 2. Endpoint swap.
set ep_files [get_files -quiet -of_objects [get_filesets sources_1] *xilinx_dma_pcie_ep.*]
if {![llength $ep_files]} { error "sim_bfm: no xilinx_dma_pcie_ep source in the example project" }
set_property IS_ENABLED false $ep_files
set ep_copy [file join $fw16_fpga_exdir fw16_fpga_ep.v]
set n [regsub {module\s+fw16_fpga_top\s*\(} [slurp [file join $fw16_fpga_src fw16_fpga_top.v]] \
         "module xilinx_dma_pcie_ep (" ep_text]
if {$n != 1} { error "sim_bfm: could not rename fw16_fpga_top in fw16_fpga_top.v" }
# AMD's usrapp tasks reference example-design internals hierarchically (board.EP.m_axi_w*, user_clk,
# C_NUM_USR_IRQ). Our test never calls those tasks, but elaboration needs the names to exist, so the
# EP copy gets tied-off shims (lesson xdma-example-tb-hierarchical-refs; VRFC 10-2991 without them),
# plus a pulldown on pps_in, the one card input the shell now samples.
set shims {
    // --- BFM shims (sim only): names the example testbench references; tasks using them are not called.
    localparam C_NUM_USR_IRQ = 1;
    wire         user_clk     = axi_aclk;
    wire [511:0] m_axi_wdata  = 512'd0;
    wire [63:0]  m_axi_wstrb  = 64'd0;
    wire         m_axi_wvalid = 1'b0;
    wire         m_axi_wready = 1'b0;
    // board.v leaves the card ports unconnected: a floating pps_in would put X into TS and IRQ_PENDING.
    pulldown pps_in_pd (pps_in);
endmodule}
set n [regsub {endmodule\s*$} $ep_text $shims ep_text]
if {$n != 1} { error "sim_bfm: could not find the closing endmodule in fw16_fpga_top.v" }
spit $ep_copy $ep_text
add_files -norecurse [list [file join $fw16_fpga_src fw16_fpga_shell.v] $ep_copy]

# The testbench's EP port list, logged so a port mismatch is easy to read.
set board [lindex [get_files -quiet -of_objects [get_filesets sim_1] */board.v] 0]
if {$board ne "" && [regexp {xilinx_dma_pcie_ep[^;]*;} [slurp $board] ep_inst]} {
  puts "fw16-fpga: board.v EP instance:\n$ep_inst"
}

# 3. The fw16-fpga test.
set tests [lindex [get_files -quiet -of_objects [get_filesets sim_1] *sample_tests*.vh] 0]
if {$tests eq ""} { error "sim_bfm: sample_tests.vh not found in sim_1" }
set fw16_fpga_test [string map [list @MAGIC@ $fw16_fpga_magic] {if (1'b1) begin
    board.RP.tx_usrapp.TSK_SIMULATION_TIMEOUT(10050);
    board.RP.tx_usrapp.TSK_SYSTEM_INITIALIZATION;
    $display("[%t] FW16_FPGA_BFM: link up", $realtime);
    board.RP.tx_usrapp.TSK_BAR_INIT;
    board.RP.tx_usrapp.TSK_TX_BAR_READ(3'd0, 32'h0, board.RP.tx_usrapp.DEFAULT_TAG, board.RP.tx_usrapp.DEFAULT_TC);
    board.RP.tx_usrapp.TSK_WAIT_FOR_READ_DATA;
    $display("[%t] FW16_FPGA_BFM: BAR0[0x0] = %h", $realtime, board.RP.tx_usrapp.P_READ_DATA);
    if (board.RP.tx_usrapp.P_READ_DATA == @MAGIC@)
      $display("[%t] FW16_FPGA_BFM: PASS", $realtime);
    else
      $display("[%t] FW16_FPGA_BFM: FAIL", $realtime);
    $finish;
  end else }]
set n [regsub {if\s*\(\s*testname\s*==} [slurp $tests] "$fw16_fpga_test&" patched]
if {$n != 1} { error "sim_bfm: no 'if (testname ==' in $tests to hook the fw16-fpga test onto" }
spit $tests $patched

# 4. Run to completion.
set_property top board [get_filesets sim_1]
set_property -name xsim.simulate.runtime -value -all -objects [get_filesets sim_1]
# If the chain's first 'if (testname ==' is inline in pci_exp_usrapp_tx.v and the
# hook above lands on an 'else if', a name no real test matches still falls
# through to it. Harmless if the tb doesn't read the plusarg.
set_property -name xsim.simulate.xsim.more_options -value {-testplusarg TESTNAME=fw16_fpga_bar0_id} \
  -objects [get_filesets sim_1]
update_compile_order -fileset sim_1
launch_simulation -simset sim_1 -mode behavioral
close_sim

set log [lindex [glob -nocomplain [file join $fw16_fpga_exdir * *.sim sim_1 behav xsim simulate.log]] 0]
if {$log eq ""} { error "sim_bfm: simulate.log not found under $fw16_fpga_exdir" }
file copy -force $log [file join $fw16_fpga_reports bfm_simulate.log]
set lines [lsearch -all -inline [split [slurp $log] \n] *FW16_FPGA_BFM*]
puts "fw16-fpga: BFM result\n  [join $lines "\n  "]"
if {[lsearch $lines {*FW16_FPGA_BFM: PASS*}] < 0} { error "sim_bfm: no PASS in $log" }
