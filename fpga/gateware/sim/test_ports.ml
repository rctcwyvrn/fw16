(* Top-level port names: step 3's XDMA wrapper wires to these by name, so
   nested interfaces must flatten to XDMA/Vivado AXI names (s_axil_awvalid,
   m_axis_c2h_tvalid), with no ppx_hardcaml "$" mangling. *)

open Omni_shell

let%expect_test "shell_top port names" =
  print_endline (String.concat " " (Shell_top.I.Names_and_widths.port_names));
  print_endline (String.concat " " (Shell_top.O.Names_and_widths.port_names));
  [%expect {|
    clock clear s_axil_awvalid s_axil_awaddr s_axil_wvalid s_axil_wdata s_axil_wstrb s_axil_bready s_axil_arvalid s_axil_araddr s_axil_rready s_axis_h2c_tvalid s_axis_h2c_tdata s_axis_h2c_tkeep s_axis_h2c_tlast c2h_tready link_up temp pwr_mw
    s_axil_awready s_axil_wready s_axil_bvalid s_axil_bresp s_axil_arready s_axil_rvalid s_axil_rdata s_axil_rresp h2c_tready m_axis_c2h_tvalid m_axis_c2h_tdata m_axis_c2h_tkeep m_axis_c2h_tlast
    |}]
;;
