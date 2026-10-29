(* AXI-Stream test source (SRC block): offers counter or PRBS-31 beats at a
   programmable rate. Rate is credit-based: each cycle adds SRC_RATE to an
   accumulator, a beat costs 0x10000, so SRC_RATE = 0x8000 offers one beat
   every second cycle. Credit saturates at just under two beats, so a stalled
   consumer cannot bank an unbounded burst. *)

open Hardcaml
open Signal

let one_beat = 0x10000
let credit_width = 18

module I = struct
  type 'a t =
    { clock : 'a
    ; clear : 'a
    ; en : 'a
    ; mode : 'a  (** 0 counter, 1 PRBS-31 *)
    ; rate : 'a [@bits 32]
    ; sync_clear : 'a  (** SRC_CLEAR pulse *)
    ; tready : 'a
    }
  [@@deriving hardcaml]
end

module O = struct
  type 'a t =
    { axis : 'a Axis.t [@rtlprefix "m_axis_"]
    ; beats : 'a [@bits 64]
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

let create (_ : Scope.t) (i : _ I.t) : _ O.t =
  let spec = Reg_spec.create ~clock:i.clock ~clear:i.clear () in
  let unlimited = i.rate >=:. one_beat in
  let credit = wire credit_width in
  let tvalid = i.en &: (unlimited |: (credit >=:. one_beat)) in
  let accept = tvalid &: i.tready in
  let beat_index = reg_fb spec ~width:64 ~f:(fun d ->
    mux2 i.sync_clear (zero 64) (mux2 accept (d +:. 1) d))
  in
  let prbs = wire Pattern.prbs_width in
  let prbs_data, prbs_next = Pattern.prbs_beat prbs in
  let seed = of_unsigned_int ~width:Pattern.prbs_width Pattern.prbs_seed in
  prbs <-- reg ~clear_to:seed spec (mux2 i.sync_clear seed (mux2 accept prbs_next prbs));
  let credit_max = of_unsigned_int ~width:credit_width ((2 * one_beat) - 1) in
  let added =
    uresize credit ~width:(credit_width + 15)
    +: uresize (sel_bottom i.rate ~width:17) ~width:(credit_width + 15)
  in
  let spent = mux2 accept (added -:. one_beat) added in
  let saturated =
    mux2
      (spent >: uresize credit_max ~width:(width spent))
      credit_max
      (uresize spent ~width:credit_width)
  in
  credit
  <-- reg spec
        (mux2 (i.sync_clear |: ~:(i.en) |: unlimited) (zero credit_width) saturated);
  { O.axis =
      { tvalid
      ; tdata = mux2 i.mode prbs_data (Pattern.counter_beat beat_index)
      ; tkeep = ones (Axis.data_width / 8)
      ; tlast = gnd
      }
  ; beats = beat_index
  }
;;

let hierarchical scope i =
  let module H = Hierarchy.In_scope (I) (O) in
  H.hierarchical ~scope ~name:"fw16_fpga_source" create i
;;
