(* TS block. The unit test drives Timestamp alone, cycle by cycle, to pin the
   PPS capture latency; the shell test goes through AXI-Lite: TS_LATCH into
   TS_NOW, a PPS stimulus captured into TS_PPS, TS_PPS_COUNT counting edges
   (not high cycles), and the IRQ PPS source. The _LO-latches-_HI tear case
   needs 2^32 cycles here, so test_latch covers it on a Regs-only circuit. *)

open Hardcaml
open Fw16_fpga_shell

let%expect_test "timestamp: latch and PPS capture latency" =
  let module Sim = Cyclesim.With_interface (Timestamp.I) (Timestamp.O) in
  let sim = Sim.create (Timestamp.create (Scope.create ~flatten_design:true ())) in
  let i = Cyclesim.inputs sim and o = Cyclesim.outputs sim in
  (* [n] = cycles since clear, which is the counter value during the next cycle *)
  let n = ref 0 in
  let cycle () =
    Cyclesim.cycle sim;
    incr n
  in
  i.clear := Bits.vdd;
  Cyclesim.cycle sim;
  i.clear := Bits.gnd;
  let u64 r = Bits.to_unsigned_int64 !r in
  for _ = 1 to 10 do cycle () done;
  let at = !n in
  i.latch := Bits.vdd;
  cycle ();
  i.latch := Bits.gnd;
  cycle ();
  Printf.printf "latch in cycle %d -> now=%Ld\n" at (u64 o.now);
  for _ = 1 to 20 do cycle () done;
  let raised = !n in
  i.pps_in := Bits.vdd;
  let edge_seen = ref None in
  for _ = 1 to 10 do
    if Bits.to_bool !(o.pps_edge) && Option.is_none !edge_seen then edge_seen := Some !n;
    cycle ()
  done;
  Printf.printf
    "pps_in high from cycle %d: edge pulse in cycle %s, pps=%Ld (latency %Ld), count=%d\n"
    raised
    (Option.fold ~none:"never" ~some:string_of_int !edge_seen)
    (u64 o.pps)
    (Int64.sub (u64 o.pps) (Int64.of_int raised))
    (Bits.to_unsigned_int !(o.pps_count));
  Printf.printf "matches Timestamp.pps_latency (%d): %b\n" Timestamp.pps_latency
    (Int64.sub (u64 o.pps) (Int64.of_int raised) = Int64.of_int Timestamp.pps_latency);
  [%expect {|
    latch in cycle 10 -> now=10
    pps_in high from cycle 32: edge pulse in cycle 34, pps=34 (latency 2), count=1
    matches Timestamp.pps_latency (2): true
    |}]
;;

let create () =
  let module Sim = Cyclesim.With_interface (Shell_top.I) (Shell_top.O) in
  let sim = Sim.create (Shell_top.Bram.create (Scope.create ~flatten_design:true ())) in
  let i = Cyclesim.inputs sim in
  i.clear := Bits.vdd;
  Cyclesim.cycle sim;
  i.clear := Bits.gnd;
  sim
;;

let ts sim what =
  let rd a = Axil_driver.read sim a in
  let now_lo = rd 0x4004 in
  let now_hi = rd 0x4008 in
  let pps_lo = rd 0x400C in
  let pps_hi = rd 0x4010 in
  let o = Cyclesim.outputs ~clock_edge:Before sim in
  Printf.printf "%-24s now=%d:%d pps=%d:%d count=%d pending=%d req=%d\n" what now_hi now_lo
    pps_hi pps_lo (rd 0x4014) (rd 0x5004) (Bits.to_unsigned_int !(o.usr_irq_req))
;;

let%expect_test "timestamp through the shell: TS registers and the PPS irq" =
  let sim = create () in
  let i = Cyclesim.inputs sim in
  let hold v n =
    i.pps_in := v;
    for _ = 1 to n do Cyclesim.cycle sim done
  in
  ts sim "reset";
  Axil_driver.write sim 0x4000 1;
  ts sim "TS_LATCH";
  Axil_driver.write sim 0x4000 1;
  ts sim "TS_LATCH again";
  Axil_driver.write sim 0x5000 0b010;
  hold Bits.vdd 10;
  ts sim "PPS high, IRQ_EN.PPS";
  hold Bits.vdd 100;
  ts sim "still high: no new edge";
  Axil_driver.write sim 0x5004 0b010;
  ts sim "IRQ_PENDING cleared";
  hold Bits.gnd 10;
  hold Bits.vdd 10;
  ts sim "second edge";
  [%expect {|
    reset                    now=0:0 pps=0:0 count=0 pending=0 req=0
    TS_LATCH                 now=0:19 pps=0:0 count=0 pending=0 req=0
    TS_LATCH again           now=0:40 pps=0:0 count=0 pending=0 req=0
    PPS high, IRQ_EN.PPS     now=0:40 pps=0:65 count=1 pending=2 req=1
    still high: no new edge  now=0:40 pps=0:65 count=1 pending=2 req=1
    IRQ_PENDING cleared      now=0:40 pps=0:65 count=1 pending=0 req=0
    second edge              now=0:40 pps=0:242 count=2 pending=2 req=1
    |}]
;;
