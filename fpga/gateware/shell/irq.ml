(* IRQ block: latches interrupt sources into IRQ_PENDING and drives XDMA's
   usr_irq_req[0] with the OR of IRQ_PENDING & IRQ_EN.

   usr_irq_req is a level: XDMA sends one MSI-X message when it rises and
   another only after it falls and rises again, so it stays high until the
   host clears the pending bit (RW1C). A source that is already pending
   therefore cannot fire twice; enabling a pending source fires it. Pending
   bits latch whether or not they are enabled, and a set wins over a clear in
   the same cycle so no event is lost. IRQ_COUNT counts rising edges of the
   request, i.e. messages XDMA was asked to send. *)

open Hardcaml
open Signal

let sources = 3

module I = struct
  type 'a t =
    { clock : 'a
    ; clear : 'a
    ; en : 'a [@bits sources]  (** IRQ_EN *)
    ; ack_clear : 'a [@bits sources]  (** IRQ_PENDING write-1-to-clear mask *)
    ; force : 'a  (** IRQ_FORCE bit 0 pulse *)
    ; pps : 'a  (** PPS rising-edge pulse *)
    ; snk_err : 'a  (** STATUS.SNK_ERR level *)
    }
  [@@deriving hardcaml]
end

module O = struct
  type 'a t =
    { pending : 'a [@bits sources]
    ; count : 'a [@bits 32]
    ; usr_irq_req : 'a
    }
  [@@deriving hardcaml]
end

let create (_ : Scope.t) (i : _ I.t) : _ O.t =
  let spec = Reg_spec.create ~clock:i.clock ~clear:i.clear () in
  let rose s = s &: ~:(reg spec s) in
  let set = concat_lsb [ i.force; i.pps; rose i.snk_err ] in
  let pending = reg_fb spec ~width:sources ~f:(fun d -> d &: ~:(i.ack_clear) |: set) in
  let usr_irq_req = (pending &: i.en) <>:. 0 in
  let count = reg_fb spec ~width:32 ~f:(fun d -> mux2 (rose usr_irq_req) (d +:. 1) d) in
  { O.pending; count; usr_irq_req }
;;

let hierarchical scope i =
  let module H = Hierarchy.In_scope (I) (O) in
  H.hierarchical ~scope ~name:"omni_irq" create i
;;
