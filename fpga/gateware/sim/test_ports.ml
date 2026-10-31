(* Top-level port names: step 3's XDMA wrapper wires to these by name, so
   nested interfaces must flatten to XDMA/Vivado AXI names (s_axil_awvalid,
   m_axis_c2h_tvalid), with no ppx_hardcaml "$" mangling. *)

open Hardcaml
open Fw16_fpga_shell

let%expect_test "shell_top port names" =
  print_endline (String.concat " " (Shell_top.I.Names_and_widths.port_names));
  print_endline (String.concat " " (Shell_top.O.Names_and_widths.port_names));
  [%expect {|
    clock clear s_axil_awvalid s_axil_awaddr s_axil_wvalid s_axil_wdata s_axil_wstrb s_axil_bready s_axil_arvalid s_axil_araddr s_axil_rready s_axis_h2c_tvalid s_axis_h2c_tdata s_axis_h2c_tkeep s_axis_h2c_tlast c2h_tready link_up temp pwr_mw usr_irq_ack pps_in slot_clk s_axib_awid s_axib_awaddr s_axib_awlen s_axib_awsize s_axib_awburst s_axib_awprot s_axib_awlock s_axib_awcache s_axib_awvalid s_axib_wdata s_axib_wstrb s_axib_wlast s_axib_wvalid s_axib_bready s_axib_arid s_axib_araddr s_axib_arlen s_axib_arsize s_axib_arburst s_axib_arprot s_axib_arlock s_axib_arcache s_axib_arvalid s_axib_rready
    s_axil_awready s_axil_wready s_axil_bvalid s_axil_bresp s_axil_arready s_axil_rvalid s_axil_rdata s_axil_rresp h2c_tready m_axis_c2h_tvalid m_axis_c2h_tdata m_axis_c2h_tkeep m_axis_c2h_tlast usr_irq_req s_axib_awready s_axib_wready s_axib_bid s_axib_bresp s_axib_bvalid s_axib_arready s_axib_rid s_axib_rdata s_axib_rresp s_axib_rlast s_axib_rvalid
    |}]
;;

let%expect_test "unflattened scope: source, sink, ring, irq, timestamp and slot are sub-modules" =
  let module C = Circuit.With_interface (Shell_top.I) (Shell_top.O) in
  let scope = Scope.create ~flatten_design:false () in
  let top = C.create_exn ~name:"fw16_fpga_shell" (Shell_top.Bram.create scope) in
  let subs = Circuit_database.get_circuits (Scope.circuit_database scope) in
  print_endline
    (String.concat " " (List.sort compare (List.map Circuit.name subs)));
  let mangled c =
    List.concat_map Signal.names (Circuit.inputs c @ Circuit.outputs c)
    |> List.filter (fun n -> String.contains n '$')
  in
  Printf.printf "ports with $: %d\n"
    (List.length (List.concat_map mangled (top :: subs)));
  [%expect {|
    fw16_fpga_irq fw16_fpga_ring fw16_fpga_sink fw16_fpga_slot fw16_fpga_source fw16_fpga_timestamp
    ports with $: 0
    |}]
;;
