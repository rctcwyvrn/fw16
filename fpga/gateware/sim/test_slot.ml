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

(* A project that ignores clear: a one-beat register stage with no reset. In
   SLOT_CTL.RESET it would still take H2C and offer its beat to C2H, so only
   the shell's gates keep it silent. *)
module Deaf_pipe = struct
  open Signal

  let create scope (i : _ Slot.I.t) : _ Slot.O.t =
    let o = Slot_echo.create scope i in
    let spec = Reg_spec.create ~clock:i.clock () in
    let full = wire 1 in
    let take = i.s_axis.tvalid &: ~:full in
    let give = full &: i.m_tready in
    full <-- reg spec (mux2 take vdd (mux2 give gnd full));
    let beat = Axis.Of_signal.reg ~enable:take spec i.s_axis in
    { o with s_tready = ~:full; m_axis = { beat with tvalid = full } }
  ;;
end

(* H2C beats base .. base + n - 1 offered back to back, C2H always ready, and
   the handshakes seen each cycle; [step] can drive the AXI-Lite writes too, so
   the stream never pauses while SLOT_CTL changes. *)
type feed =
  { sim : Axil_driver.sim
  ; base : int
  ; n : int
  ; mutable sent : int
  ; mutable got : string list
  ; mutable taken : int  (** H2C handshakes since the last [report] *)
  }

let step f () =
  let i = Cyclesim.inputs f.sim and o = Cyclesim.outputs ~clock_edge:Before f.sim in
  i.c2h_tready := Bits.vdd;
  i.h2c.tvalid := Bits.of_bool (f.sent < f.n);
  i.h2c.tdata := Bits.of_unsigned_int ~width:Axis.data_width (f.base + f.sent);
  i.h2c.tlast := Bits.of_bool (f.sent = f.n - 1);
  Cyclesim.cycle f.sim;
  if f.sent < f.n && Bits.to_bool !(o.h2c_tready)
  then (
    f.sent <- f.sent + 1;
    f.taken <- f.taken + 1);
  if Bits.to_bool !(o.c2h.tvalid)
  then
    f.got
    <- Printf.sprintf "%d%s"
         (Bits.to_unsigned_int (Bits.sel_bottom !(o.c2h.tdata) ~width:16))
         (if Bits.to_bool !(o.c2h.tlast) then "L" else "")
       :: f.got
;;

let run f cycles = for _ = 1 to cycles do step f () done

let report f what =
  Printf.printf "  %-12s h2c taken=%d c2h: [%s]\n" what f.taken
    (String.concat " " (List.rev f.got));
  f.got <- [];
  f.taken <- 0
;;

let slot_ctl f v = Axil_driver.write_on ~cycle:(step f) (Axil_driver.shell_port f.sim) 0x6000 v

let%expect_test "slot: SLOT_CTL.RESET stops both streams and resumes without loss" =
  List.iter
    (fun (name, (module P : Slot.Project)) ->
      Printf.printf "%s\n" name;
      let sim = create (module P) in
      Axil_driver.write sim 0x3000 2;
      let f = { sim; base = 500; n = 8; sent = 0; got = []; taken = 0 } in
      (* in reset before the stream starts, then released *)
      Axil_driver.write sim 0x6000 1;
      run f 20;
      report f "RESET held";
      slot_ctl f 0;
      run f 20;
      report f "released";
      (* into reset mid-stream, then released. C2H under "RESET held" is the
         ring draining beats the slot passed before RESET landed; Deaf_pipe
         keeps 602 inside through the reset and gives it up after. *)
      let f = { f with base = 600; sent = 0 } in
      run f 3;
      slot_ctl f 1;
      report f "until RESET";
      run f 20;
      report f "RESET held";
      slot_ctl f 0;
      run f 20;
      report f "released";
      Printf.printf "  snk_beats=%d ring_fill=%d\n" (Axil_driver.read sim 0x2008)
        (Axil_driver.read sim 0x3008))
    [ "Slot_echo", (module Slot_echo : Slot.Project); "Deaf_pipe", (module Deaf_pipe) ];
  [%expect {|
    Slot_echo
      RESET held   h2c taken=0 c2h: []
      released     h2c taken=8 c2h: [500 501 502 503 504 505 506 507]
      until RESET  h2c taken=5 c2h: [600 601 602]
      RESET held   h2c taken=0 c2h: [603 604]
      released     h2c taken=3 c2h: [605 606 607]
      snk_beats=16 ring_fill=0
    Deaf_pipe
      RESET held   h2c taken=0 c2h: []
      released     h2c taken=8 c2h: [500 501 502 503 504 505 506 507]
      until RESET  h2c taken=3 c2h: [600]
      RESET held   h2c taken=0 c2h: [601]
      released     h2c taken=5 c2h: [602 603 604 605 606 607]
      snk_beats=16 ring_fill=0
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
