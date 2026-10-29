# Dump every package pin of the card's FPGA as CSV (one row per ball).
# Run with the Windows Vivado, e.g. from PowerShell:
#   vivado.bat -mode batch -nojournal -nolog -source fpga\bay\tools\au15p_pins.tcl -tclargs <out.csv>
# The CSV is the single source for the KiCad symbol (au15p_symbol.py) and the pin plan.
set part xcau15p-ffvb676-2-e
set out [lindex $argv 0]
link_design -part $part
set f [open $out w]
puts $f "ball,pin_func,bank,general_purpose,differential,diff_pair,master,global_clk,bytegroup,nibble"
foreach p [lsort [get_package_pins]] {
  set row {}
  foreach k {NAME PIN_FUNC BANK IS_GENERAL_PURPOSE IS_DIFFERENTIAL DIFF_PAIR_PIN IS_MASTER IS_GLOBAL_CLK PKGPIN_BYTEGROUP_INDEX PKGPIN_NIBBLE_INDEX} {
    lappend row [get_property $k $p]
  }
  puts $f [join $row ","]
}
close $f
puts "wrote [llength [get_package_pins]] pins for $part to $out"
