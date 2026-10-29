(* Testbench helper: single AXI-Lite reads and writes against a Cyclesim.
   [write]/[read] drive a Shell_top sim (ports under the [s_axil_] prefix);
   [write_on]/[read_on] take the raw AXI-Lite refs of any other circuit. *)

open Hardcaml
open Fw16_fpga_shell

type sim = (Bits.t ref Shell_top.I.t, Bits.t ref Shell_top.O.t) Cyclesim.t
type port = Bits.t ref Axi_lite.M2s.t * Bits.t ref Axi_lite.S2m.t

let idle (i : Bits.t ref Axi_lite.M2s.t) =
  Axi_lite.M2s.iter i ~f:(fun r -> r := Bits.zero (Bits.width !r))
;;

let write_on ~cycle ((i, o) : port) addr data =
  i.awvalid := Bits.vdd;
  i.awaddr := Bits.of_unsigned_int ~width:32 addr;
  i.wvalid := Bits.vdd;
  i.wdata := Bits.of_unsigned_int ~width:32 data;
  i.wstrb := Bits.of_unsigned_int ~width:4 0xF;
  i.bready := Bits.vdd;
  while not (Bits.to_bool !(o.awready)) do cycle () done;
  cycle ();
  idle i;
  i.bready := Bits.vdd;
  cycle ();
  idle i
;;

let read_on ~cycle ((i, o) : port) addr =
  i.arvalid := Bits.vdd;
  i.araddr := Bits.of_unsigned_int ~width:32 addr;
  i.rready := Bits.vdd;
  while not (Bits.to_bool !(o.arready)) do cycle () done;
  cycle ();
  i.arvalid := Bits.gnd;
  let v = Bits.to_unsigned_int !(o.rdata) in
  cycle ();
  idle i;
  v
;;

let shell_port (sim : sim) : port =
  (Cyclesim.inputs sim).axil, (Cyclesim.outputs ~clock_edge:Before sim).axil
;;

let write (sim : sim) = write_on ~cycle:(fun () -> Cyclesim.cycle sim) (shell_port sim)
let read (sim : sim) = read_on ~cycle:(fun () -> Cyclesim.cycle sim) (shell_port sim)
