(* Shell v0: the register file, test source, checker, ring buffer and the
   project slot, wired to XDMA's two AXI-Stream channels and its BAR2 bypass.

     source ────────────────┐
     H2C ──┬─> checker      ├─ mux (RING_CTL.SRC_SEL) ─> ring buffer ─> C2H
           ├────────────────┤   0 source, 1 H2C, 2 slot, 3 none
           └─> slot ────────┘   (H2C reaches the slot only when 2)
     BAR2 (AXI4 bypass) ─> slot;  TS counter and PPS edge ─> slot
     pps_in ─> timestamp ─> TS registers, IRQ PPS;  slot_irq ─> IRQ SLOT

   The ring's memory and the slot's project are parameters ([Make]); [Bram]
   is the BRAM ring with the Slot_echo project, the build used in simulation
   and on branch A spin 1. *)

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
    ; usr_irq_ack : 'a
          (** XDMA usr_irq_ack[0]; unused: usr_irq_req is held until the host
              clears IRQ_PENDING, which is all the MSI-X handshake needs *)
    ; pps_in : 'a  (** raw U.FL PPS pin, asynchronous; Timestamp synchronises it *)
    ; slot_clk : 'a  (** free clock for the slot (clk100m); the shell never uses it *)
    ; axib : 'a Axi4.M2s.t [@rtlprefix "s_axib_"]  (** XDMA bypass master, BAR2 *)
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module O = struct
  type 'a t =
    { axil : 'a Axi_lite.S2m.t [@rtlprefix "s_axil_"]
    ; h2c_tready : 'a
    ; c2h : 'a Axis.t [@rtlprefix "m_axis_c2h_"]
    ; usr_irq_req : 'a  (** XDMA usr_irq_req[0] *)
    ; axib : 'a Axi4.S2m.t [@rtlprefix "s_axib_"]
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module Make (M : Mem_port.S) (P : Slot.Project) = struct
  module Ring = Ring_buffer.Make (M)

  (* Source, sink, ring, irq, timestamp and slot are hierarchical: separate modules in
     generated RTL unless [scope] flattens the design (simulation). Regs stays inline;
     its [value] lookup is not an interface. *)
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
    let irq_pending = ro_wire "IRQ_PENDING" Irq.sources in
    let irq_count = ro_wire "IRQ_COUNT" 32 in
    let ts_now_lo = ro_wire "TS_NOW_LO" 32 in
    let ts_now_hi = ro_wire "TS_NOW_HI" 32 in
    let ts_pps_lo = ro_wire "TS_PPS_LO" 32 in
    let ts_pps_hi = ro_wire "TS_PPS_HI" 32 in
    let ts_pps_count = ro_wire "TS_PPS_COUNT" 32 in
    let regs =
      Regs.create ~clock:i.clock ~clear:i.clear ~blocks:Map.blocks
        ~hw:
          (("TEMP", i.temp)
           :: ("PWR_MW", i.pwr_mw)
           :: ("RING_DEPTH", of_unsigned_int ~width:32 M.depth)
           :: !ro)
        i.axil
    in
    let bit name n = (regs.value name).:(n) in
    let src_ctl = "SRC_CTL" and snk_ctl = "SNK_CTL" in
    let src_sel = sel_bottom (regs.value "RING_CTL") ~width:2 in
    let from_h2c = src_sel ==:. 1 and from_slot = src_sel ==:. 2 in
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
       ring can stall H2C, and with SRC_SEL = 2 the slot can. *)
    let h2c_tready = wire 1 in
    let h2c_taken = { i.h2c with tvalid = i.h2c.tvalid &: h2c_tready } in
    let sink =
      Stream_sink.hierarchical scope
        { clock = i.clock
        ; clear = i.clear
        ; en = bit snk_ctl 0
        ; mode = bit snk_ctl 1
        ; sync_clear = bit "SNK_CLEAR" 0
        ; axis = h2c_taken
        }
    in
    let ts =
      Timestamp.hierarchical scope
        { clock = i.clock; clear = i.clear; latch = bit "TS_LATCH" 0; pps_in = i.pps_in }
    in
    (* The slot gets a real handshake on H2C: its tvalid does not wait for its
       own tready (sink.tready is constant 1), and H2C completes exactly when
       the slot takes the beat. slot_clear comes off a no-clear register so it
       is high during the shell clear as well. *)
    let ring_ready = wire 1 in
    let slot =
      Slot.hierarchical
        (module P)
        scope
        { clock = i.clock
        ; slot_clk = i.slot_clk
        ; clear = reg (Reg_spec.create ~clock:i.clock ()) (i.clear |: bit "SLOT_CTL" 0)
        ; s_axis = { i.h2c with tvalid = i.h2c.tvalid &: from_slot &: sink.tready }
        ; m_tready = from_slot &: ring_ready
        ; axi = i.axib
        ; ts_now = ts.count
        ; pps_edge = ts.pps_edge
        }
    in
    let idle = Axis.map Axis.port_widths ~f:zero in
    let ring_in = Axis.Of_signal.mux src_sel [ source.axis; h2c_taken; slot.m_axis; idle ] in
    let ring =
      Ring.hierarchical scope
        { clock = i.clock
        ; clear = i.clear
        ; sync_clear = bit "RING_CLEAR" 0
        ; s_axis = ring_in
        ; m_tready = i.c2h_tready
        }
    in
    ring_ready <-- ring.s_tready;
    source_ready <-- ((src_sel ==:. 0) &: ring.s_tready);
    h2c_tready
    <-- (sink.tready &: mux2 from_h2c ring.s_tready (mux2 from_slot slot.s_tready vdd));
    status
    <-- concat_lsb
          [ i.link_up; ring.mem_ready; sink.errors <>:. 0; gnd; zero 28 ];
    src_beats_lo <-- sel_bottom source.beats ~width:32;
    src_beats_hi <-- sel_top source.beats ~width:32;
    snk_errs <-- sink.errors;
    snk_beats_lo <-- sel_bottom sink.beats ~width:32;
    snk_beats_hi <-- sel_top sink.beats ~width:32;
    ring_fill <-- ring.fill;
    ts_now_lo <-- sel_bottom ts.now ~width:32;
    ts_now_hi <-- sel_top ts.now ~width:32;
    ts_pps_lo <-- sel_bottom ts.pps ~width:32;
    ts_pps_hi <-- sel_top ts.pps ~width:32;
    ts_pps_count <-- ts.pps_count;
    let irq =
      Irq.hierarchical scope
        { clock = i.clock
        ; clear = i.clear
        ; en = sel_bottom (regs.value "IRQ_EN") ~width:Irq.sources
        ; ack_clear = sel_bottom (regs.value "IRQ_PENDING") ~width:Irq.sources
        ; force = bit "IRQ_FORCE" 0
        ; pps = ts.pps_edge
        ; snk_err = sink.errors <>:. 0
        ; slot = slot.irq
        }
    in
    irq_pending <-- irq.pending;
    irq_count <-- irq.count;
    { axil = regs.s2m
    ; h2c_tready
    ; c2h = ring.m_axis
    ; usr_irq_req = irq.usr_irq_req
    ; axib = slot.axi
    }
  ;;
end

module Bram =
  Make
    (Bram_mem.Make (struct
       let depth = 4096
     end))
    (Slot_echo)
