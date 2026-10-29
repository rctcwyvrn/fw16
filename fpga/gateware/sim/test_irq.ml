(* IRQ block through the whole shell: pending latches without enable, enabling
   a pending source raises usr_irq_req, a still-pending source cannot fire
   twice, RW1C clears and re-arms, and SNK_ERR fires once per error run. *)

open Hardcaml
open Fw16_fpga_shell

let create () =
  let module Sim = Cyclesim.With_interface (Shell_top.I) (Shell_top.O) in
  let sim = Sim.create (Shell_top.Bram.create (Scope.create ~flatten_design:true ())) in
  let i = Cyclesim.inputs sim in
  i.clear := Bits.vdd;
  Cyclesim.cycle sim;
  i.clear := Bits.gnd;
  sim
;;

let show sim what =
  let o = Cyclesim.outputs ~clock_edge:Before sim in
  Printf.printf "%-22s pending=%d count=%d req=%d\n" what (Axil_driver.read sim 0x5004)
    (Axil_driver.read sim 0x5008) (Bits.to_unsigned_int !(o.usr_irq_req))
;;

let%expect_test "irq: forced source, enable, RW1C" =
  let sim = create () in
  show sim "reset";
  Axil_driver.write sim 0x500C 1;
  show sim "force, not enabled";
  Axil_driver.write sim 0x5000 0b001;
  show sim "enable FORCE";
  Axil_driver.write sim 0x500C 1;
  show sim "force while pending";
  Axil_driver.write sim 0x5004 0b001;
  show sim "clear (RW1C)";
  Axil_driver.write sim 0x500C 1;
  show sim "force again";
  [%expect {|
    reset                  pending=0 count=0 req=0
    force, not enabled     pending=1 count=0 req=0
    enable FORCE           pending=1 count=1 req=1
    force while pending    pending=1 count=1 req=1
    clear (RW1C)           pending=0 count=1 req=0
    force again            pending=1 count=2 req=1
    |}]
;;

let%expect_test "irq: SNK_ERR fires on the first bad beat only" =
  let sim = create () in
  let i = Cyclesim.inputs sim in
  Axil_driver.write sim 0x5000 0b100;
  Axil_driver.write sim 0x2000 0b01;
  (* counter mode expects beat 0 = 0; send 1 three times: three errors *)
  i.h2c.tvalid := Bits.vdd;
  i.h2c.tdata := Bits.of_unsigned_int ~width:Axis.data_width 1;
  for _ = 1 to 3 do Cyclesim.cycle sim done;
  i.h2c.tvalid := Bits.gnd;
  Printf.printf "SNK_ERRS=%d\n" (Axil_driver.read sim 0x2004);
  show sim "after error run";
  Axil_driver.write sim 0x5004 0b100;
  show sim "cleared, errors remain";
  [%expect {|
    SNK_ERRS=3
    after error run        pending=4 count=1 req=1
    cleared, errors remain pending=0 count=1 req=0
    |}]
;;
