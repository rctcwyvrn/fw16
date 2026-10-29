(* Register file through the whole shell: constants, scratch read/write,
   reset values, writes to RO registers dropped, no aliasing between blocks. *)

open Hardcaml
open Omni_shell

let%expect_test "regs: id, version, scratch, reset values" =
  let module Sim = Cyclesim.With_interface (Shell_top.I) (Shell_top.O) in
  let sim = Sim.create (Shell_top.Bram.create (Scope.create ~flatten_design:true ())) in
  let i = Cyclesim.inputs sim in
  i.clear := Bits.vdd;
  Cyclesim.cycle sim;
  i.clear := Bits.gnd;
  let rd a = Printf.printf "0x%04X = 0x%08X\n" a (Axil_driver.read sim a) in
  rd 0x0000;
  rd 0x0004;
  rd 0x1004;
  rd 0x3004;
  Axil_driver.write sim 0x0008 0xDEADBEEF;
  rd 0x0008;
  Axil_driver.write sim 0x0000 0x12345678;
  rd 0x0000;
  rd 0x0FFC;
  (* TS_NOW_LO is tied to 0; a truncated decoder aliased it onto ID. *)
  rd 0x4000;
  [%expect {|
    0x0000 = 0x4F4D4E49
    0x0004 = 0x00000002
    0x1004 = 0x00010000
    0x3004 = 0x00001000
    0x0008 = 0xDEADBEEF
    0x0000 = 0x4F4D4E49
    0x0FFC = 0x00000000
    0x4000 = 0x00000000
    |}]
;;
