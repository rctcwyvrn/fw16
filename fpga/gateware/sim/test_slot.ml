(* The project slot through the whole shell: H2C -> slot -> ring -> C2H with
   RING_CTL.SRC_SEL = 2 (and H2C kept out of the slot otherwise), Slot_echo's
   BAR2 registers over AXI4 with the lane each narrow read lands in, and IRQ
   bit 3 plus SLOT_CTL.RESET with a small test project. *)

open Hardcaml
open Fw16_fpga_shell

module Mem = Bram_mem.Make (struct
    let depth = 64
  end)

let create (module P : Slot.Project) : Axil_driver.sim =
  let module S = Shell_top.Make (Mem) (P) in
  let module Sim = Cyclesim.With_interface (Shell_top.I) (Shell_top.O) in
  let sim = Sim.create (S.create (Scope.create ~flatten_design:true ())) in
  let i = Cyclesim.inputs sim in
  i.clear := Bits.vdd;
  Cyclesim.cycle sim;
  i.clear := Bits.gnd;
  (* checker on, so snk_beats counts every H2C beat taken *)
  Axil_driver.write sim 0x2000 1;
  sim
;;

(* Offer [n] H2C beats (data k = base + k, tlast on the last) with C2H always
   ready; print what came out of C2H and what the checker and ring counted. *)
let stream (sim : Axil_driver.sim) what ~base ~n =
  let i = Cyclesim.inputs sim and o = Cyclesim.outputs ~clock_edge:Before sim in
  i.c2h_tready := Bits.vdd;
  let got = ref [] and sent = ref 0 and cycles = ref 0 in
  while !cycles < 4 * n + 10 do
    let k = !sent in
    i.h2c.tvalid := Bits.of_bool (k < n);
    i.h2c.tdata := Bits.of_unsigned_int ~width:Axis.data_width (base + k);
    i.h2c.tlast := Bits.of_bool (k = n - 1);
    if Bits.to_bool !(o.c2h.tvalid)
    then
      got
      := Printf.sprintf "%d%s"
           (Bits.to_unsigned_int (Bits.sel_bottom !(o.c2h.tdata) ~width:16))
           (if Bits.to_bool !(o.c2h.tlast) then "L" else "")
         :: !got;
    if k < n && Bits.to_bool !(o.h2c_tready) then incr sent;
    Cyclesim.cycle sim;
    incr cycles
  done;
  i.h2c.tvalid := Bits.gnd;
  Printf.printf "%-22s c2h: [%s] snk_beats=%d ring_fill=%d\n" what
    (String.concat " " (List.rev !got))
    (Axil_driver.read sim 0x2008) (Axil_driver.read sim 0x3008)
;;

let%expect_test "slot: SRC_SEL routes H2C through the echo project to C2H" =
  let sim = create (module Slot_echo) in
  Axil_driver.write sim 0x3000 2;
  stream sim "SRC_SEL=2 (slot)" ~base:100 ~n:6;
  Axil_driver.write sim 0x3000 0;
  stream sim "SRC_SEL=0 (source off)" ~base:200 ~n:3;
  Axil_driver.write sim 0x3000 3;
  stream sim "SRC_SEL=3 (none)" ~base:300 ~n:3;
  Axil_driver.write sim 0x3000 1;
  stream sim "SRC_SEL=1 (loopback)" ~base:400 ~n:3;
  [%expect {|
    SRC_SEL=2 (slot)       c2h: [100 101 102 103 104 105] snk_beats=6 ring_fill=0
    SRC_SEL=0 (source off) c2h: [] snk_beats=9 ring_fill=0
    SRC_SEL=3 (none)       c2h: [] snk_beats=12 ring_fill=0
    SRC_SEL=1 (loopback)   c2h: [400 401 402] snk_beats=15 ring_fill=0
    |}]
;;

let%expect_test "slot: echo's BAR2 registers over AXI4, lane by address" =
  let sim = create (module Slot_echo) in
  List.iter
    (fun addr ->
      match Axi4_driver.read ~size:2 ~id:5 sim addr with
      | [ (d, rid, last) ] ->
        Printf.printf "read  0x%02x -> 0x%08x (rid=%d last=%b)\n" addr
          (Axi4_driver.word_at d addr) rid last
      | beats -> Printf.printf "read 0x%02x: %d beats\n" addr (List.length beats))
    [ 0x0; 0x4; 0x8; 0x20 ];
  let bid, bresp = Axi4_driver.write32 ~id:9 sim 0x8 0xC0FFEE in
  Printf.printf "write 0x08 <- 0xc0ffee (bid=%d bresp=%d)\n" bid bresp;
  let _ = Axi4_driver.write32 sim 0x0 0xDEAD in
  let beats = Axi4_driver.read ~len:2 ~id:3 sim 0x0 in
  List.iteri
    (fun k (d, rid, last) ->
      Printf.printf "burst beat %d: id=0x%08x ver=0x%08x scratch=0x%08x (rid=%d last=%b)\n" k
        (Axi4_driver.word_at d 0x0) (Axi4_driver.word_at d 0x4) (Axi4_driver.word_at d 0x8)
        rid last)
    beats;
  [%expect {|
    read  0x00 -> 0x4543484f (rid=5 last=true)
    read  0x04 -> 0x00010000 (rid=5 last=true)
    read  0x08 -> 0x00000000 (rid=5 last=true)
    read  0x20 -> 0x00000000 (rid=5 last=true)
    write 0x08 <- 0xc0ffee (bid=9 bresp=0)
    burst beat 0: id=0x4543484f ver=0x00010000 scratch=0x00c0ffee (rid=3 last=false)
    burst beat 1: id=0x00000000 ver=0x00000000 scratch=0x00000000 (rid=3 last=true)
    |}]
;;

(* Echo, plus slot_irq = "a tlast has gone through since slot_clear". *)
module Irq_project = struct
  open Signal

  let create scope (i : _ Slot.I.t) : _ Slot.O.t =
    let o = Slot_echo.create scope i in
    let spec = Reg_spec.create ~clock:i.clock ~clear:i.clear () in
    let seen = i.s_axis.tvalid &: i.s_axis.tlast &: o.s_tready in
    { o with irq = reg_fb spec ~width:1 ~f:(fun d -> d |: seen) }
  ;;
end

let%expect_test "slot: IRQ bit 3 is a level; SLOT_CTL.RESET drops it" =
  let sim = create (module Irq_project) in
  let show what =
    let o = Cyclesim.outputs ~clock_edge:Before sim in
    Printf.printf "%-26s pending=0x%x req=%d\n" what (Axil_driver.read sim 0x5004)
      (Bits.to_unsigned_int !(o.usr_irq_req))
  in
  (* checker off: these beats are not its pattern, and SNK_ERR would set bit 2 *)
  Axil_driver.write sim 0x2000 0;
  Axil_driver.write sim 0x5000 0b1000;
  Axil_driver.write sim 0x3000 2;
  show "enabled, no packet";
  stream sim "one packet" ~base:0 ~n:2;
  show "after the packet";
  Axil_driver.write sim 0x5004 0b1000;
  show "cleared, level still high";
  Axil_driver.write sim 0x6000 1;
  Axil_driver.write sim 0x6000 0;
  Axil_driver.write sim 0x5004 0b1000;
  show "SLOT_CTL.RESET, cleared";
  [%expect {|
    enabled, no packet         pending=0x0 req=0
    one packet             c2h: [0 1] snk_beats=0 ring_fill=0
    after the packet           pending=0x8 req=1
    cleared, level still high  pending=0x8 req=1
    SLOT_CTL.RESET, cleared    pending=0x0 req=0
    |}]
;;
