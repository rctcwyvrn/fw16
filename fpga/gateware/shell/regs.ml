(* AXI-Lite register file generated from a register map (gateware/regmap), so
   the map that produced host/include/omni_regs.h is the one decoded here.

   One transaction at a time: a write is accepted when address and data are
   both valid, a read when no read response is pending. Unmapped reads return
   0 with OKAY (probing is harmless); writes to RO or unmapped offsets are
   dropped. wstrb is honoured per byte on RW and RW1C registers.

   RW1C state lives with whatever sets it (e.g. Irq): this file reads the
   supplied value and turns a write into a one-cycle clear mask.

   Every RO pair [<P>_LO]/[<P>_HI] is tear-free: the read that returns [_LO]
   also captures the live high word into a shadow, and [_HI] reads the shadow. *)

open Hardcaml
open Signal
module Reg = Regmap.Reg

type t =
  { s2m : Signal.t Axi_lite.S2m.t
  ; value : string -> Signal.t
      (** current value of an RW register, this cycle's pulse bits of a
          WO_PULSE register, or this cycle's clear mask of an RW1C register;
          raises on an unknown or RO name *)
  }

let regs_of blocks = List.concat_map (fun (b : Reg.block) -> b.regs) blocks

let suffixed ~suffix name =
  let n = String.length name and k = String.length suffix in
  if n > k && String.sub name (n - k) k = suffix then Some (String.sub name 0 (n - k)) else None
;;

(** [hw] gives the value of every RO register except ID and VERSION (constants
    from the map), and of every RW1C register. *)
let create ~clock ~clear ~(blocks : Reg.block list) ~(hw : (string * Signal.t) list)
      (m : Signal.t Axi_lite.M2s.t)
  =
  let spec = Reg_spec.create ~clock ~clear () in
  let regs = regs_of blocks in
  (* Decode the full word address: truncating it would alias blocks (a 12-bit
     word address folded TS at 0x4000 onto CTL at 0x0000). *)
  let word_addr a = drop_bottom a ~width:2 in
  let is_at a (r : Reg.t) = word_addr a ==:. (r.offset lsr 2) in
  (* write channel *)
  let bvalid = wire 1 in
  let write = m.awvalid &: m.wvalid &: ~:bvalid in
  bvalid <-- reg_fb spec ~width:1 ~f:(fun d -> mux2 write vdd (mux2 m.bready gnd d));
  let hit r = write &: is_at m.awaddr r in
  let strobed old =
    List.init 4 (fun b ->
      let byte s = select s ~high:((8 * b) + 7) ~low:(8 * b) in
      mux2 m.wstrb.:(b) (byte m.wdata) (byte old))
    |> concat_lsb
  in
  let hw_value name =
    match List.assoc_opt name hw with
    | Some s -> uresize s ~width:32
    | None -> failwith ("Regs.create: no value for register " ^ name)
  in
  let owned =
    List.filter_map
      (fun (r : Reg.t) ->
        match r.access with
        | Reg.RO -> None
        | RW ->
          let v =
            reg_fb ~clear_to:(of_unsigned_int ~width:32 r.reset) spec ~width:32
              ~f:(fun d -> mux2 (hit r) (strobed d) d)
          in
          Some (r.name, v)
        | WO_PULSE -> Some (r.name, reg spec (mux2 (hit r) m.wdata (zero 32)))
        | RW1C -> Some (r.name, reg spec (mux2 (hit r) (strobed (zero 32)) (zero 32))))
      regs
  in
  (* read channel *)
  let rvalid = wire 1 in
  let read = m.arvalid &: ~:rvalid in
  let is_ro name = List.exists (fun (r : Reg.t) -> r.name = name && r.access = RO) regs in
  (* [_HI] of an RO pair: the high word captured by the last [_LO] read, in
     the same cycle rdata captured the low word. *)
  let shadowed_hi name =
    match suffixed ~suffix:"_HI" name with
    | Some p when is_ro (p ^ "_LO") ->
      let lo = List.find (fun (r : Reg.t) -> r.name = p ^ "_LO") regs in
      Some (reg spec ~enable:(read &: is_at m.araddr lo) (hw_value name))
    | _ -> None
  in
  let read_value (r : Reg.t) =
    match r.access, r.name with
    | Reg.RO, ("ID" | "VERSION") -> of_unsigned_int ~width:32 r.reset
    | Reg.RO, name ->
      (match shadowed_hi name with
       | Some s -> s
       | None -> hw_value name)
    | RW1C, name -> hw_value name
    | RW, name -> List.assoc name owned
    | WO_PULSE, _ -> zero 32
  in
  rvalid <-- reg_fb spec ~width:1 ~f:(fun d -> mux2 read vdd (mux2 m.rready gnd d));
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
