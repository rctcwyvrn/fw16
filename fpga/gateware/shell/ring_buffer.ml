(* Ring buffer (RING block) between an AXI-Stream input and output, stored in
   any [Mem_port.S] memory. Writes backpressure when the ring is full, so no
   data is dropped. Reads are prefetched into a small show-ahead FIFO; the
   number of reads in flight plus beats in that FIFO never exceeds its
   capacity, which makes variable memory latency safe.

   A slot is reused only after its read data has returned, not when the read
   was issued, so a memory that reorders reads against writes (DDR4) cannot
   return overwritten data. RING_CLEAR while reads are in flight is only safe
   when the stream is idle; the host clears before enabling a run. *)

open Hardcaml
open Signal

let prefetch = 16

module Make (M : Mem_port.S) = struct
  let abits = address_bits_for M.depth

  module I = struct
    type 'a t =
      { clock : 'a
      ; clear : 'a
      ; sync_clear : 'a  (** RING_CLEAR pulse *)
      ; s_axis : 'a Axis.t [@rtlprefix "s_axis_"]
      ; m_tready : 'a
      }
    [@@deriving hardcaml ~rtlmangle:false]
  end

  module O = struct
    type 'a t =
      { s_tready : 'a
      ; m_axis : 'a Axis.t [@rtlprefix "m_axis_"]
      ; fill : 'a [@bits 32]
      ; mem_ready : 'a
      }
    [@@deriving hardcaml ~rtlmangle:false]
  end

  let create scope (i : _ I.t) : _ O.t =
    let clear = i.clear |: i.sync_clear in
    let spec = Reg_spec.create ~clock:i.clock ~clear () in
    let pw = abits + 1 in
    let counter inc = reg_fb spec ~width:pw ~f:(fun d -> mux2 inc (d +:. 1) d) in
    let mem_rsp = Mem_port.Rsp.Of_signal.wires () in
    let pop = wire 1 in
    (* write side *)
    let returned = wire pw in
    let wr_ptr = wire pw in
    let full = (wr_ptr -: returned) >=:. M.depth in
    let s_tready = mem_rsp.ready &: mem_rsp.wr_ready &: ~:full in
    let write = i.s_axis.tvalid &: s_tready in
    wr_ptr <-- counter write;
    (* read side *)
    let issued = wire pw in
    let inflight = wire (num_bits_to_represent prefetch) in
    let issue =
      mem_rsp.ready &: mem_rsp.rd_ready &: (issued <>: wr_ptr) &: (inflight <:. prefetch)
    in
    issued <-- counter issue;
    returned <-- counter mem_rsp.rd_data_valid;
    inflight
    <-- reg_fb spec ~width:(width inflight) ~f:(fun d ->
      mux2 (issue &: ~:pop) (d +:. 1) (mux2 (pop &: ~:issue) (d -:. 1) d));
    let fifo =
      Fifo.create
        ~showahead:true
        ~capacity:prefetch
        ~clock:i.clock
        ~clear
        ~wr:mem_rsp.rd_data_valid
        ~d:mem_rsp.rd_data
        ~rd:pop
        ()
        ~scope
    in
    let m_tvalid = ~:(fifo.empty) in
    pop <-- (m_tvalid &: i.m_tready);
    let popped = counter pop in
    let req =
      { Mem_port.Req.wr_valid = write
      ; wr_addr = uresize (sel_bottom wr_ptr ~width:abits) ~width:32
      ; wr_data = i.s_axis.tdata
      ; rd_valid = issue
      ; rd_addr = uresize (sel_bottom issued ~width:abits) ~width:32
      }
    in
    Mem_port.Rsp.Of_signal.assign mem_rsp (M.create ~clock:i.clock ~clear:i.clear req);
    { s_tready
    ; m_axis =
        { tvalid = m_tvalid
        ; tdata = fifo.q
        ; tkeep = ones (Axis.data_width / 8)
        ; tlast = gnd
        }
    ; fill = uresize (wr_ptr -: popped) ~width:32
    ; mem_ready = mem_rsp.ready
    }
  ;;

  let hierarchical scope i =
    let module H = Hierarchy.In_scope (I) (O) in
    H.hierarchical ~scope ~name:"omni_ring" create i
  ;;
end
