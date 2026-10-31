(* TS block: a 64-bit free-running counter on the shell clock, a host latch
   into TS_NOW, and capture of the counter on each PPS rising edge.

   pps_in is the raw U.FL pin, asynchronous to the shell clock. It goes
   through a two-flop synchroniser (ASYNC_REG, so Vivado keeps the pair
   together; the top XDC false-paths the port) and a rising-edge detect here,
   not in the Verilog top, so the whole path is in cyclesim and the fixed
   capture latency is tested ([pps_latency]).

   Everything resets with [clear] (~axi_aresetn), so a PCIe link reset
   restarts time at 0. TS_PPS_COUNT wraps, like IRQ_COUNT. *)

open Hardcaml
open Signal

(** TS_PPS minus the counter value in the cycle whose closing edge first
    samples pps_in high: the two synchroniser flops. The edge compare is
    combinational on the second flop, so it adds nothing; the physical edge
    was up to one cycle (4 ns at 250 MHz) before that sample. *)
let pps_latency = 2

module I = struct
  type 'a t =
    { clock : 'a
    ; clear : 'a
    ; latch : 'a  (** TS_LATCH bit 0 pulse *)
    ; pps_in : 'a  (** raw, asynchronous *)
    }
  [@@deriving hardcaml]
end

module O = struct
  type 'a t =
    { now : 'a [@bits 64]  (** counter at the last latch *)
    ; pps : 'a [@bits 64]  (** counter at the last PPS edge *)
    ; pps_count : 'a [@bits 32]
    ; pps_edge : 'a  (** one-cycle pulse, to the IRQ block *)
    }
  [@@deriving hardcaml]
end

let create (_ : Scope.t) (i : _ I.t) : _ O.t =
  let spec = Reg_spec.create ~clock:i.clock ~clear:i.clear () in
  let count = reg_fb spec ~width:64 ~f:(fun d -> d +:. 1) in
  let synced =
    pipeline ~attributes:[ Rtl_attribute.Vivado.async_reg true ] spec ~n:2 i.pps_in
  in
  let pps_edge = synced &: ~:(reg spec synced) in
  let now = reg spec ~enable:i.latch count in
  let pps = reg spec ~enable:pps_edge count in
  let pps_count = reg_fb spec ~width:32 ~f:(fun d -> mux2 pps_edge (d +:. 1) d) in
  { O.now; pps; pps_count; pps_edge }
;;

let hierarchical scope i =
  let module H = Hierarchy.In_scope (I) (O) in
  H.hierarchical ~scope ~name:"fw16_fpga_timestamp" create i
;;
