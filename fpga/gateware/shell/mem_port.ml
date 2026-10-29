(* The ring buffer's view of its backing memory: separate write and read
   request channels with valid/ready, and read data returned in order with
   its own valid. Variable read latency is allowed, so the same ring works on
   BRAM (ready always, 1-cycle data) and on DDR4 behind MIG (stalls,
   calibration, tens of cycles). Addresses are in 256-bit beats. *)



module Req = struct
  type 'a t =
    { wr_valid : 'a
    ; wr_addr : 'a [@bits 32]
    ; wr_data : 'a [@bits Axis.data_width]
    ; rd_valid : 'a
    ; rd_addr : 'a [@bits 32]
    }
  [@@deriving hardcaml]
end

module Rsp = struct
  type 'a t =
    { ready : 'a  (** memory usable (BRAM: 1; DDR4: calibration done) *)
    ; wr_ready : 'a
    ; rd_ready : 'a
    ; rd_data_valid : 'a
    ; rd_data : 'a [@bits Axis.data_width]
    }
  [@@deriving hardcaml]
end

(** A backing memory. [depth] is in beats and must be a power of two. *)
module type S = sig
  val depth : int
  val create : clock:Signal.t -> clear:Signal.t -> Signal.t Req.t -> Signal.t Rsp.t
end
