(* *_LO reads latch *_HI: on a Regs-only circuit whose high word the test
   moves between the two reads, the torn read the latch exists to prevent.
   (Through the shell a 64-bit counter would need 2^32 beats to show it.) *)

open Hardcaml
open Fw16_fpga_shell
module Reg = Regmap.Reg

module I = struct
  type 'a t =
    { clock : 'a
    ; clear : 'a
    ; axil : 'a Axi_lite.M2s.t [@rtlprefix "s_axil_"]
    ; lo : 'a [@bits 32]
    ; hi : 'a [@bits 32]
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module O = struct
  type 'a t = { axil : 'a Axi_lite.S2m.t [@rtlprefix "s_axil_"] }
  [@@deriving hardcaml ~rtlmangle:false]
end

(* X_LO/X_HI are a pair; Y_HI has no Y_LO, so it reads live. *)
let blocks =
  [ { Reg.bname = "T"
    ; base = 0
    ; bdoc = ""
    ; regs =
        [ Reg.reg "X_LO" 0x0 RO ""; Reg.reg "X_HI" 0x4 RO ""; Reg.reg "Y_HI" 0x8 RO "" ]
    }
  ]
;;

let create (i : _ I.t) : _ O.t =
  let regs =
    Regs.create ~clock:i.clock ~clear:i.clear ~blocks
      ~hw:[ "X_LO", i.lo; "X_HI", i.hi; "Y_HI", i.hi ]
      i.axil
  in
  { O.axil = regs.s2m }
;;

let%expect_test "LO read latches HI" =
  let module Sim = Cyclesim.With_interface (I) (O) in
  let sim = Sim.create create in
  let i = Cyclesim.inputs sim and o = Cyclesim.outputs ~clock_edge:Before sim in
  let cycle () = Cyclesim.cycle sim in
  let port = i.axil, o.axil in
  let set r v = r := Bits.of_unsigned_int ~width:32 v in
  i.clear := Bits.vdd;
  cycle ();
  i.clear := Bits.gnd;
  let rd name a = Printf.printf "%s = %d\n" name (Axil_driver.read_on ~cycle port a) in
  set i.lo 1;
  set i.hi 5;
  rd "X_HI before any LO read" 0x4;
  rd "X_LO" 0x0;
  set i.hi 7;
  rd "X_HI (shadow)" 0x4;
  rd "Y_HI (unpaired, live)" 0x8;
  rd "X_LO" 0x0;
  rd "X_HI" 0x4;
  [%expect {|
    X_HI before any LO read = 0
    X_LO = 1
    X_HI (shadow) = 5
    Y_HI (unpaired, live) = 7
    X_LO = 1
    X_HI = 7
    |}]
;;
