(* Testbench helper: single AXI-Lite reads and writes against a Cyclesim
   whose inputs/outputs carry [Axi_lite] under the [s_axil_] prefix. *)

open Hardcaml
open Omni_shell

type sim = (Bits.t ref Shell_top.I.t, Bits.t ref Shell_top.O.t) Cyclesim.t

let idle (i : Bits.t ref Shell_top.I.t) =
  Axi_lite.M2s.iter i.axil ~f:(fun r -> r := Bits.zero (Bits.width !r))
;;

let write (sim : sim) addr data =
  let i = Cyclesim.inputs sim and o = Cyclesim.outputs ~clock_edge:Before sim in
  i.axil.awvalid := Bits.vdd;
  i.axil.awaddr := Bits.of_int ~width:32 addr;
  i.axil.wvalid := Bits.vdd;
  i.axil.wdata := Bits.of_int ~width:32 data;
  i.axil.wstrb := Bits.of_int ~width:4 0xF;
  i.axil.bready := Bits.vdd;
  while not (Bits.to_bool !(o.axil.awready)) do Cyclesim.cycle sim done;
  Cyclesim.cycle sim;
  idle i;
  i.axil.bready := Bits.vdd;
  Cyclesim.cycle sim;
  idle i
;;

let read (sim : sim) addr =
  let i = Cyclesim.inputs sim and o = Cyclesim.outputs ~clock_edge:Before sim in
  i.axil.arvalid := Bits.vdd;
  i.axil.araddr := Bits.of_int ~width:32 addr;
  i.axil.rready := Bits.vdd;
  while not (Bits.to_bool !(o.axil.arready)) do Cyclesim.cycle sim done;
  Cyclesim.cycle sim;
  i.axil.arvalid := Bits.gnd;
  let v = Bits.to_int !(o.axil.rdata) in
  Cyclesim.cycle sim;
  idle i;
  v
;;
