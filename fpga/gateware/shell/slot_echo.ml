(* Slot_echo: the default project. Streams pass straight through (H2C back
   out over the ring and C2H when RING_CTL.SRC_SEL = 2), and BAR2 holds the
   slot convention plus one scratch word:

     0x0  SLOT_ID       RO  0x4543484F ("ECHO")
     0x4  SLOT_VERSION  RO  0x00010000 (1.0, major in [31:16])
     0x8  SLOT_SCRATCH  RW  for host BAR2 write tests

   Everything else reads 0 and drops writes. The AXI4 slave takes one read
   and one write burst at a time (independently), INCR or FIXED, and answers
   each beat with the whole 32-byte line its address falls in: XDMA takes a
   narrow read's bytes from the lane its address selects, so a 4-byte read
   at 0x4 gets SLOT_VERSION from rdata[63:32]. slot_irq stays low. *)

open Hardcaml
open Signal

let slot_id = 0x4543484F
let slot_version = 0x00010000

(* 32-byte line index within the 1 MiB BAR2. *)
let line_of addr = select addr ~high:19 ~low:5
let line_width = 15

let create (_ : Scope.t) (i : _ Slot.I.t) : _ Slot.O.t =
  let spec = Reg_spec.create ~clock:i.clock ~clear:i.clear () in
  let a = i.axi in
  let advance burst line = mux2 (burst ==:. 0) line (line +:. 1) in
  (* write: AW, then W beats until wlast, then B *)
  let w_busy = wire 1 and bvalid = wire 1 in
  (* No address accepted in reset: slot_clear outlasts the shell clear by a
     cycle, and a handshake taken then would be lost. *)
  let awready = ~:w_busy &: ~:bvalid &: ~:(i.clear) in
  let aw = a.awvalid &: awready in
  let w = a.wvalid &: w_busy in
  let bid = reg spec ~enable:aw a.awid in
  let w_burst = reg spec ~enable:aw a.awburst in
  let w_line = wire line_width in
  w_line <-- reg spec (mux2 aw (line_of a.awaddr) (mux2 w (advance w_burst w_line) w_line));
  w_busy <-- reg spec (mux2 aw vdd (mux2 (w &: a.wlast) gnd w_busy));
  bvalid <-- reg spec (mux2 (w &: a.wlast) vdd (mux2 a.bready gnd bvalid));
  let scratch_lane = select a.wdata ~high:95 ~low:64 in
  let scratch_strb = select a.wstrb ~high:11 ~low:8 in
  let scratch =
    reg_fb spec ~width:32 ~f:(fun d ->
      let bytes =
        List.init 4 (fun b ->
          let byte s = select s ~high:((8 * b) + 7) ~low:(8 * b) in
          mux2 (w &: (w_line ==:. 0) &: scratch_strb.:(b)) (byte scratch_lane) (byte d))
      in
      concat_lsb bytes)
  in
  (* read: AR, then arlen + 1 beats *)
  let r_busy = wire 1 in
  let arready = ~:r_busy &: ~:(i.clear) in
  let ar = a.arvalid &: arready in
  let r = r_busy &: a.rready in
  let rid = reg spec ~enable:ar a.arid in
  let r_burst = reg spec ~enable:ar a.arburst in
  let r_left = wire 8 in
  r_left <-- reg spec (mux2 ar a.arlen (mux2 r (r_left -:. 1) r_left));
  let rlast = r_left ==:. 0 in
  let r_line = wire line_width in
  r_line <-- reg spec (mux2 ar (line_of a.araddr) (mux2 r (advance r_burst r_line) r_line));
  r_busy <-- reg spec (mux2 ar vdd (mux2 (r &: rlast) gnd r_busy));
  let line0 =
    concat_lsb
      [ of_unsigned_int ~width:32 slot_id
      ; of_unsigned_int ~width:32 slot_version
      ; scratch
      ; zero (Axi4.data_width - 96)
      ]
  in
  let rdata = mux2 (r_line ==:. 0) line0 (zero Axi4.data_width) in
  { Slot.O.s_tready = i.m_tready
  ; m_axis = i.s_axis
  ; axi =
      { awready
      ; wready = w_busy
      ; bid
      ; bresp = zero 2
      ; bvalid
      ; arready
      ; rid
      ; rdata
      ; rresp = zero 2
      ; rlast
      ; rvalid = r_busy
      }
  ; irq = gnd
  }
;;
