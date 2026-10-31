(* Testbench helper: AXI4 bursts against a Shell_top sim's bypass port
   (the s_axib_ ports), the way XDMA's bypass master issues them: INCR, 32-byte beats
   unless [size] says otherwise, one burst at a time. *)

open Hardcaml
open Fw16_fpga_shell

type sim = Axil_driver.sim

let ports (sim : sim) = (Cyclesim.inputs sim).axib, (Cyclesim.outputs ~clock_edge:Before sim).axib

let idle (i : Bits.t ref Axi4.M2s.t) =
  Axi4.M2s.iter i ~f:(fun r -> r := Bits.zero (Bits.width !r))
;;

(* Bounded, so a stuck handshake fails the test instead of hanging it. *)
let wait_for ?(what = "handshake") sim (r : Bits.t ref) =
  let n = ref 0 in
  while not (Bits.to_bool !r) do
    incr n;
    if !n > 1000 then failwith ("Axi4_driver: no " ^ what);
    Cyclesim.cycle sim
  done
;;

(** Read [len] beats from [addr]; returns (rdata, rid, rlast) per beat. *)
let read ?(id = 0) ?(size = 5) ?(len = 1) sim addr =
  let i, o = ports sim in
  i.arvalid := Bits.vdd;
  i.araddr := Bits.of_unsigned_int ~width:Axi4.addr_width addr;
  i.arlen := Bits.of_unsigned_int ~width:8 (len - 1);
  i.arsize := Bits.of_unsigned_int ~width:3 size;
  i.arburst := Bits.of_unsigned_int ~width:2 1;
  i.arid := Bits.of_unsigned_int ~width:Axi4.id_width id;
  wait_for sim o.arready;
  Cyclesim.cycle sim;
  i.arvalid := Bits.gnd;
  i.rready := Bits.vdd;
  let beats = ref [] in
  let fin = ref false and n = ref 0 in
  while not !fin do
    incr n;
    if !n > 1000 then failwith "Axi4_driver: no rlast";
    if Bits.to_bool !(o.rvalid)
    then (
      beats := (!(o.rdata), Bits.to_unsigned_int !(o.rid), Bits.to_bool !(o.rlast)) :: !beats;
      fin := Bits.to_bool !(o.rlast));
    Cyclesim.cycle sim
  done;
  idle i;
  List.rev !beats
;;

(** The 32-bit word a [size]-2 read at [addr] returns: XDMA's lane select. *)
let word_at rdata addr =
  let lane = (addr lsr 2) land 7 in
  Bits.to_unsigned_int (Bits.select rdata ~high:((32 * lane) + 31) ~low:(32 * lane))
;;

(** Write one 32-bit word at [addr] (a single beat with only its lane strobed);
    returns (bid, bresp). *)
let write32 ?(id = 0) sim addr data =
  let i, o = ports sim in
  let lane = (addr lsr 2) land 7 in
  i.awvalid := Bits.vdd;
  i.awaddr := Bits.of_unsigned_int ~width:Axi4.addr_width addr;
  i.awlen := Bits.zero 8;
  i.awsize := Bits.of_unsigned_int ~width:3 2;
  i.awburst := Bits.of_unsigned_int ~width:2 1;
  i.awid := Bits.of_unsigned_int ~width:Axi4.id_width id;
  wait_for sim o.awready;
  Cyclesim.cycle sim;
  i.awvalid := Bits.gnd;
  i.wvalid := Bits.vdd;
  i.wdata
  := Bits.(sll (uresize (of_unsigned_int ~width:32 data) ~width:Axi4.data_width) ~by:(32 * lane));
  i.wstrb := Bits.of_unsigned_int ~width:(Axi4.data_width / 8) (0xF lsl (4 * lane));
  i.wlast := Bits.vdd;
  wait_for sim o.wready;
  Cyclesim.cycle sim;
  idle i;
  i.bready := Bits.vdd;
  wait_for sim o.bvalid;
  let b = Bits.to_unsigned_int !(o.bid), Bits.to_unsigned_int !(o.bresp) in
  Cyclesim.cycle sim;
  idle i;
  b
;;
