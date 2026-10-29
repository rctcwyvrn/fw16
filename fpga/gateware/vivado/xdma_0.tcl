# xdma_0: the XDMA IP as docs/regmap-v0.2-proposal.md §3 fixes it.
#   PCIe Gen4 x4 endpoint (the link trains down to Gen3 by itself), 256-bit
#   AXI-ST at 250 MHz, 1 H2C + 1 C2H, BAR0 = 64 KiB AXI-Lite master (the shell's
#   register map, PCIe-to-AXI translation 0, so awaddr = BAR offset),
#   BAR1 = DMA config, MSI-X with usr_irq width 1, subsystem 0x10EE/0x4F4D.
#
# Sourced inside an open project. The CONFIG names were written without a
# Vivado to check them against; xdma_set therefore resolves each name
# case-insensitively, stops with the near matches if one doesn't exist, and
# reads every value back. The full property list lands in
# reports/xdma_0.properties.txt.

set xdma_config {
  mode_selection              Advanced
  pl_link_cap_max_link_width  X4
  pl_link_cap_max_link_speed  16.0_GT/s
  ref_clk_freq                100_MHz
  axi_data_width              256_bit
  axisten_freq                250
  xdma_axi_intf_mm            AXI_Stream
  xdma_rnum_chnl              1
  xdma_wnum_chnl              1
  axilite_master_en           true
  axilite_master_size         64
  axilite_master_scale        Kilobytes
  pf0_msi_enabled             false
  pf0_msix_enabled            true
  xdma_num_usr_irq            1
  pf0_subsystem_vendor_id     10EE
  pf0_subsystem_id            4F4D
}

proc xdma_set {ip config} {
  set props [list_property $ip]
  set dict {}
  set missing {}
  foreach {k v} $config {
    set i [lsearch -exact -nocase $props CONFIG.$k]
    if {$i < 0} {
      set near [lsearch -all -inline -nocase $props "CONFIG.*[lindex [split $k _] end]*"]
      lappend missing "$k (near: [lrange $near 0 9])"
    } else {
      lappend dict [lindex $props $i] $v
    }
  }
  if {[llength $missing]} {
    error "xdma_0: no such CONFIG property:\n  [join $missing "\n  "]"
  }
  # One at a time, in list order: later options are only enabled once
  # mode_selection / the interface choice above them are set.
  foreach {k v} $dict { set_property $k $v $ip }
  foreach {k v} $dict {
    set got [get_property $k $ip]
    if {![string equal -nocase $got $v]} {
      error "xdma_0: $k is '$got' after setting '$v' (dependent on another option?)"
    }
  }
}

create_ip -name xdma -vendor xilinx.com -library ip -version 4.2 -module_name xdma_0
set xdma_ip [get_ips xdma_0]
xdma_set $xdma_ip $xdma_config
report_property -file [file join $omni_reports xdma_0.properties.txt] $xdma_ip
generate_target all [get_files xdma_0.xci]
