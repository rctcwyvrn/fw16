(* Source -> ring -> C2H with a stalling consumer, and H2C -> checker with an
   injected bit error: the two paths omni-loopback exercises. The waveform of
   the first beats is part of the expected output. *)

open Hardcaml
open Hardcaml_waveterm
open Omni_shell
module R = Regmap.Map_v0

let reset sim =
  let i = Cyclesim.inputs sim in
  i.Shell_top.I.clear := Bits.vdd;
  Cyclesim.cycle sim;
  i.clear := Bits.gnd
;;

let%expect_test "c2h: counter source through the ring, consumer ready 1 in 3" =
  let module Sim = Cyclesim.With_interface (Shell_top.I) (Shell_top.O) in
  let sim = Sim.create Shell_top.Bram.create in
  let waves, sim = Cyclesim.Waveform.create sim in
  let i = Cyclesim.inputs sim and o = Cyclesim.outputs ~clock_edge:Before sim in
  reset sim;
  Axil_driver.write sim 0x1000 0b01;
  let got = ref [] in
  for n = 0 to 60 do
    i.c2h_tready := Bits.of_bool (n mod 3 = 0);
    if Bits.to_bool !(o.c2h.tvalid) && Bits.to_bool !(i.c2h_tready)
    then got := Bits.to_unsigned_int (Bits.sel_bottom !(o.c2h.tdata) ~width:30) :: !got;
    Cyclesim.cycle sim
  done;
  Printf.printf "first words: %s\n"
    (String.concat " " (List.rev_map string_of_int !got));
  Printf.printf "ring fill: %d\n" (Axil_driver.read sim 0x3008);
  (* Handshake only: 256-bit tdata is too wide to read here; "first words"
     already checks the data. Globs keep the rules independent of how
     ppx_hardcaml mangles nested port names. *)
  let display_rules =
    Display_rule.
      [ port_name_is "clock"
      ; port_name_is "c2h_tready"
      ; port_name_matches (Glob "*c2h*tvalid")
      ]
  in
  Waveform.print ~display_rules ~display_width:90 ~start_cycle:6
    ~signals_width:20 waves;
  [%expect {|
    first words: 0 4 8 12 16 20 24 28 32 36 40 44 48 52 56 60 64 68 72 76
    ring fill: 43
    ┌Signals───────────┐┌Waves───────────────────────────────────────────────────────────────┐
    │clock             ││┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───┐   ┌───│
    │                  ││    └───┘   └───┘   └───┘   └───┘   └───┘   └───┘   └───┘   └───┘   │
    │c2h_tready        ││        ┌───────┐               ┌───────┐               ┌───────┐   │
    │                  ││────────┘       └───────────────┘       └───────────────┘       └───│
    │m_axis_c2h_tvalid ││────────────────────────────────────────────────────────────────────│
    │                  ││                                                                    │
    └──────────────────┘└────────────────────────────────────────────────────────────────────┘
    |}]
;;

let%expect_test "h2c: checker counts one corrupted beat, counter and PRBS" =
  List.iter
    (fun mode ->
      let module Sim = Cyclesim.With_interface (Shell_top.I) (Shell_top.O) in
      let sim = Sim.create Shell_top.Bram.create in
      let i = Cyclesim.inputs sim in
      reset sim;
      Axil_driver.write sim 0x2000 (0b01 lor (mode lsl 1));
      (* Generate the expected beats with the same Pattern functions, in Bits. *)
      let module P = Pattern.Make (Bits) in
      let prbs = ref (Bits.of_unsigned_int ~width:Pattern.prbs_width Pattern.prbs_seed) in
      for k = 0 to 19 do
        let data =
          if mode = 0
          then P.counter_beat (Bits.of_unsigned_int ~width:64 k)
          else (
            let d, next = P.prbs_beat !prbs in
            prbs := next;
            d)
        in
        let data = if k = 7 then Bits.(data ^: of_unsigned_int ~width:Axis.data_width 1) else data in
        i.h2c.tvalid := Bits.vdd;
        i.h2c.tdata := data;
        Cyclesim.cycle sim
      done;
      i.h2c.tvalid := Bits.gnd;
      Printf.printf "mode %d: beats=%d errors=%d\n" mode
        (Axil_driver.read sim 0x2008) (Axil_driver.read sim 0x2004))
    [ 0; 1 ];
  [%expect {|
    mode 0: beats=20 errors=1
    mode 1: beats=20 errors=1
    |}]
;;
