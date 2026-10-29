(* The two test patterns shared by the source and the sink, as pure
   next-state functions so both sides generate bit-identical beats.

   Counter: beat k carries the little-endian 64-bit words 4k, 4k+1, 4k+2,
   4k+3 — a host can check it as an incrementing uint64 array.
   PRBS-31: x^31 + x^28 + 1, 256 bits per beat (the LFSR advanced 256 steps
   per cycle, unrolled into an XOR network). *)

open Hardcaml

let words_per_beat = Axis.data_width / 64
let prbs_width = 31
let prbs_seed = 0x7FFF_FFFF

(* Over any [Comb.S] so testbenches can compute expected beats in [Bits]. *)
module Make (C : Comb.S) = struct
  open C
  (** Beat for counter value [beat_index] (64 bits). *)
  let counter_beat beat_index =
    let base = sll (uresize beat_index 64) 2 in
    List.init words_per_beat (fun i -> base +:. i) |> concat_lsb
  ;;

  (** One LFSR step: returns (output bit, next state). *)
  let prbs_step state =
    let fb = state.:(30) ^: state.:(27) in
    fb, drop_top state 1 @: fb
  ;;

  (** [prbs_beat state] = (256 data bits, state after 256 steps). Bit 0 of the
      data is the first bit out. *)
  let prbs_beat state =
    let rec go n state acc =
      if n = 0
      then concat_lsb (List.rev acc), state
      else (
        let bit, state = prbs_step state in
        go (n - 1) state (bit :: acc))
    in
    go Axis.data_width state []
  ;;
end

include Make (Signal)
