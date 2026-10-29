(* AXI-Lite register file generated from a register map (gateware/regmap), so
   the map that produced host/include/omni_regs.h is the one decoded here.

   One transaction at a time: a write is accepted when address and data are
   both valid, a read when no read response is pending. Unmapped reads return
   0 with OKAY (probing is harmless); writes to RO or unmapped offsets are
   dropped. wstrb is honoured per byte on RW registers. *)

open Hardcaml
open Signal
module Reg = Regmap.Reg

type t =
  { s2m : Signal.t Axi_lite.S2m.t
  ; value : string -> Signal.t
      (** current value of an RW register, or this cycle's pulse bits of a
          WO_PULSE register; raises on an unknown or RO name *)
  }

let regs_of blocks = List.concat_map (fun (b : Reg.block) -> b.regs) blocks

(** [ro] gives the value of every RO register except ID and VERSION, which are
    constants from the map. *)
let create ~clock ~clear ~(blocks : Reg.block list) ~(ro : (string * Signal.t) list)
      (m : Signal.t Axi_lite.M2s.t)
  =
  let spec = Reg_spec.create ~clock ~clear () in
  let regs = regs_of blocks in
  let word_addr a = sel_bottom (srl a 2) 12 in
  let is_at a (r : Reg.t) = word_addr a ==:. (r.offset lsr 2) in
  (* write channel *)
  let bvalid = wire 1 in
  let write = m.awvalid &: m.wvalid &: ~:bvalid in
  bvalid <== reg_fb spec ~width:1 ~f:(fun d -> mux2 write vdd (mux2 m.bready gnd d));
  let hit r = write &: is_at m.awaddr r in
  let strobed old =
    List.init 4 (fun b ->
      mux2 m.wstrb.:(b) (select m.wdata ((8 * b) + 7) (8 * b)) (select old ((8 * b) + 7) (8 * b)))
    |> concat_lsb
  in
  let owned =
    List.filter_map
      (fun (r : Reg.t) ->
        match r.access with
        | Reg.RO -> None
        | RW ->
          let v =
            reg_fb (Reg_spec.override spec ~clear_to:(of_int ~width:32 r.reset)) ~width:32
              ~f:(fun d -> mux2 (hit r) (strobed d) d)
          in
          Some (r.name, v)
        | WO_PULSE -> Some (r.name, reg spec (mux2 (hit r) m.wdata (zero 32))))
      regs
  in
  (* read channel *)
  let read_value (r : Reg.t) =
    match r.access, r.name with
    | Reg.RO, ("ID" | "VERSION") -> of_int ~width:32 r.reset
    | Reg.RO, name ->
      (match List.assoc_opt name ro with
       | Some s -> uresize s 32
       | None -> failwith ("Regs.create: no value for RO register " ^ name))
    | RW, name -> List.assoc name owned
    | WO_PULSE, _ -> zero 32
  in
  let rvalid = wire 1 in
  let read = m.arvalid &: ~:rvalid in
  rvalid <== reg_fb spec ~width:1 ~f:(fun d -> mux2 read vdd (mux2 m.rready gnd d));
  let decoded =
    List.fold_left
      (fun acc r -> mux2 (is_at m.araddr r) (read_value r) acc)
      (zero 32) regs
  in
  let rdata = reg spec ~enable:read decoded in
  let value name =
    match List.assoc_opt name owned with
    | Some v -> v
    | None -> failwith ("Regs.value: not an RW or pulse register " ^ name)
  in
  { s2m =
      { awready = write
      ; wready = write
      ; bvalid
      ; bresp = zero 2
      ; arready = read
      ; rvalid
      ; rdata
      ; rresp = zero 2
      }
  ; value
  }
;;
