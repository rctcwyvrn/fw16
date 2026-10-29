(* emit_verilog: print the BRAM shell as Verilog for the Vivado project
   (gateware/vivado/). Top module [fw16_fpga_shell]; source, sink, ring and irq
   come out as their own modules, as in test_ports. *)

open Hardcaml
open Fw16_fpga_shell

let () =
  let module C = Circuit.With_interface (Shell_top.I) (Shell_top.O) in
  let scope = Scope.create ~flatten_design:false () in
  let circuit = C.create_exn ~name:"fw16_fpga_shell" (Shell_top.Bram.create scope) in
  Rtl.print ~database:(Scope.circuit_database scope) Verilog circuit
;;
