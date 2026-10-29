(* Shell v0: the register file, test source, checker and ring buffer wired
   to the two XDMA AXI-Stream channels.

     source ─┐                      ┌─> C2H (m_axis_c2h)
             ├─ mux ─> ring buffer ─┘
     H2C ────┤  (RING_CTL.LOOPBACK picks H2C)
             └─> checker (always sees H2C)

   The ring's memory is a parameter ([Make]); [Bram] is the BRAM build used in
   simulation and on branch A spin 1. *)

open Hardcaml
open Signal
module Map = Regmap.Map_v0

(* [~rtlmangle:false] drops the field name from nested ports, so they get
   XDMA/Vivado AXI names: s_axil_awvalid, not s_axil_axil$awvalid. It must be
   record-level: a per-field [@rtlmangle] is spliced in as a separator string,
   so it cannot turn mangling off. *)
module I = struct
  type 'a t =
    { clock : 'a
    ; clear : 'a
    ; axil : 'a Axi_lite.M2s.t [@rtlprefix "s_axil_"]
    ; h2c : 'a Axis.t [@rtlprefix "s_axis_h2c_"]
    ; c2h_tready : 'a
    ; link_up : 'a
    ; temp : 'a [@bits 16]  (** SYSMON temperature word *)
    ; pwr_mw : 'a [@bits 32]
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module O = struct
  type 'a t =
    { axil : 'a Axi_lite.S2m.t [@rtlprefix "s_axil_"]
    ; h2c_tready : 'a
    ; c2h : 'a Axis.t [@rtlprefix "m_axis_c2h_"]
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module Make (M : Mem_port.S) = struct
  module Ring = Ring_buffer.Make (M)

  (* Source, sink and ring are hierarchical: separate modules in generated RTL
     unless [scope] flattens the design (simulation). Regs stays inline; its
     [value] lookup is not an interface. *)
  let create scope (i : _ I.t) : _ O.t =
    let ro = ref [] in
    let ro_wire name w =
      let s = wire w in
      ro := (name, s) :: !ro;
      s
    in
    let status = ro_wire "STATUS" 32 in
    let src_beats_lo = ro_wire "SRC_BEATS_LO" 32 in
    let src_beats_hi = ro_wire "SRC_BEATS_HI" 32 in
    let snk_errs = ro_wire "SNK_ERRS" 32 in
    let snk_beats_lo = ro_wire "SNK_BEATS_LO" 32 in
    let snk_beats_hi = ro_wire "SNK_BEATS_HI" 32 in
    let ring_fill = ro_wire "RING_FILL" 32 in
    let ts_ro = List.map (fun n -> ro_wire n 32) [ "TS_NOW_LO"; "TS_NOW_HI"; "TS_PPS_LO"; "TS_PPS_HI"; "TS_PPS_COUNT" ] in
    let regs =
      Regs.create ~clock:i.clock ~clear:i.clear ~blocks:Map.blocks
        ~ro:
          (("TEMP", i.temp)
           :: ("PWR_MW", i.pwr_mw)
           :: ("RING_DEPTH", of_unsigned_int ~width:32 M.depth)
           :: !ro)
        i.axil
    in
    let bit name n = (regs.value name).:(n) in
    let src_ctl = "SRC_CTL" and snk_ctl = "SNK_CTL" in
    let source_ready = wire 1 in
    let source =
      Stream_source.hierarchical scope
        { clock = i.clock
        ; clear = i.clear
        ; en = bit src_ctl 0
        ; mode = bit src_ctl 1
        ; rate = regs.value "SRC_RATE"
        ; sync_clear = bit "SRC_CLEAR" 0
        ; tready = source_ready
        }
    in
    (* The checker counts completed H2C handshakes, not offers: in loopback the
       ring can stall H2C. *)
    let h2c_tready = wire 1 in
    let sink =
      Stream_sink.hierarchical scope
        { clock = i.clock
        ; clear = i.clear
        ; en = bit snk_ctl 0
        ; mode = bit snk_ctl 1
        ; sync_clear = bit "SNK_CLEAR" 0
        ; axis = { i.h2c with tvalid = i.h2c.tvalid &: h2c_tready }
        }
    in
    let loopback = bit "RING_CTL" 0 in
    let ring_in = Axis.Of_signal.mux2 loopback { i.h2c with tvalid = i.h2c.tvalid &: h2c_tready } source.axis in
    let ring =
      Ring.hierarchical scope
        { clock = i.clock
        ; clear = i.clear
        ; sync_clear = bit "RING_CLEAR" 0
        ; s_axis = ring_in
        ; m_tready = i.c2h_tready
        }
    in
    source_ready <-- (~:loopback &: ring.s_tready);
    h2c_tready <-- (sink.tready &: (~:loopback |: ring.s_tready));
    status
    <-- concat_lsb
          [ i.link_up; ring.mem_ready; sink.errors <>:. 0; gnd; zero 28 ];
    src_beats_lo <-- sel_bottom source.beats ~width:32;
    src_beats_hi <-- sel_top source.beats ~width:32;
    snk_errs <-- sink.errors;
    snk_beats_lo <-- sel_bottom sink.beats ~width:32;
    snk_beats_hi <-- sel_top sink.beats ~width:32;
    ring_fill <-- ring.fill;
    (* TS block: step 4; reads 0 until the timestamp unit lands. *)
    List.iter (fun s -> s <-- zero 32) ts_ro;
    { axil = regs.s2m; h2c_tready; c2h = ring.m_axis }
  ;;
end

module Bram = Make (Bram_mem.Make (struct
    let depth = 4096
  end))
