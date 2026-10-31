(* The project slot (decision 0013): the one boundary a project plugs into,
   so a project is a [Project] module, never an edit of the shell.

   Everything here is on axi_aclk ([clock]) except [slot_clk], a free 100 MHz
   clock (clk100m) that keeps running without PCIe; a project that runs on it
   crosses into it itself, with the slot-side CDC library (not built yet).
   [clear] is synchronous and active high: the shell clear (link reset) or
   SLOT_CTL.RESET.

   The boundary is drawn as if it were frozen for partial reconfiguration:
   fixed widths, one module ([fw16_fpga_slot]) with keep_hierarchy, whatever
   project is inside. Rescan reload is the only route built. *)

open Hardcaml

module I = struct
  type 'a t =
    { clock : 'a
    ; slot_clk : 'a
    ; clear : 'a
    ; s_axis : 'a Axis.t [@rtlprefix "s_axis_"]
        (** H2C, only while RING_CTL.SRC_SEL = 2; a real AXI-Stream handshake *)
    ; m_tready : 'a  (** ring ready, only while SRC_SEL = 2 *)
    ; axi : 'a Axi4.M2s.t [@rtlprefix "s_axib_"]  (** BAR2, addresses = BAR2 offsets *)
    ; ts_now : 'a [@bits 64]  (** live TS counter *)
    ; pps_edge : 'a  (** one-cycle pulse, the one TS_PPS captures on *)
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

module O = struct
  type 'a t =
    { s_tready : 'a
    ; m_axis : 'a Axis.t [@rtlprefix "m_axis_"]  (** to the ring while SRC_SEL = 2 *)
    ; axi : 'a Axi4.S2m.t [@rtlprefix "s_axib_"]
    ; irq : 'a  (** level, IRQ bit 3 SLOT *)
    }
  [@@deriving hardcaml ~rtlmangle:false]
end

(** A project. Convention: SLOT_ID at BAR2 offset 0x0 and SLOT_VERSION at 0x4,
    so the host can tell which project is loaded. [clear] outlasts the shell
    clear by one cycle (it is registered), so a project's AXI4 readys must be
    low while [clear] is high. Plain AXI rules hold at the
    boundary: [m_axis.tvalid] must not depend on [m_tready] (the ring's ready
    does not depend on its tvalid, and [s_axis.tvalid] not on [s_tready]). *)
module type Project = sig
  val create : Scope.t -> Signal.t I.t -> Signal.t O.t
end

let hierarchical (module P : Project) scope i =
  let module H = Hierarchy.In_scope (I) (O) in
  H.hierarchical ~scope ~name:"fw16_fpga_slot"
    ~attributes:[ Rtl_attribute.Vivado.keep_hierarchy true ]
    P.create i
;;
