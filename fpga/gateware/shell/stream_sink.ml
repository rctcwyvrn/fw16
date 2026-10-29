(* AXI-Stream checker (SNK block): always ready; while enabled, compares each
   beat with the expected counter or PRBS-31 beat and counts mismatches. The
   expected sequence advances on every received beat, so one corrupt beat
   costs one error, not a cascade; a dropped or duplicated beat shows up as
   errors on every following beat until SNK_CLEAR. *)

open Hardcaml
open Signal

module I = struct
  type 'a t =
    { clock : 'a
    ; clear : 'a
    ; en : 'a
    ; mode : 'a  (** 0 counter, 1 PRBS-31 *)
    ; sync_clear : 'a  (** SNK_CLEAR pulse *)
    ; axis : 'a Axis.t [@rtlprefix "s_axis_"]
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module O = struct
  type 'a t =
    { tready : 'a
    ; errors : 'a [@bits 32]
    ; beats : 'a [@bits 64]
    }
  [@@deriving hardcaml]
end

let create (_ : Scope.t) (i : _ I.t) : _ O.t =
  let spec = Reg_spec.create ~clock:i.clock ~clear:i.clear () in
  let take = i.en &: i.axis.tvalid in
  let count w = reg_fb spec ~width:w ~f:(fun d ->
    mux2 i.sync_clear (zero w) (mux2 take (d +:. 1) d))
  in
  let beats = count 64 in
  let seed = of_unsigned_int ~width:Pattern.prbs_width Pattern.prbs_seed in
  let prbs = wire Pattern.prbs_width in
  let prbs_expected, prbs_next = Pattern.prbs_beat prbs in
  prbs
  <-- reg ~clear_to:seed spec
        (mux2 i.sync_clear seed (mux2 take prbs_next prbs));
  let expected = mux2 i.mode prbs_expected (Pattern.counter_beat beats) in
  let mismatch = take &: (i.axis.tdata <>: expected) in
  let errors = reg_fb spec ~width:32 ~f:(fun d ->
    mux2 i.sync_clear (zero 32)
      (mux2 (mismatch &: (d <>:. 0xFFFF_FFFF)) (d +:. 1) d))
  in
  { O.tready = vdd; errors; beats }
;;

let hierarchical scope i =
  let module H = Hierarchy.In_scope (I) (O) in
  H.hierarchical ~scope ~name:"fw16_fpga_sink" create i
;;
