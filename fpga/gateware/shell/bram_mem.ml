(* BRAM backing for the ring buffer (branch A spin 1, and simulation): a
   simple dual-port RAM, always ready, read data one cycle after the request.
   [Make] fixes the depth so it satisfies [Mem_port.S]. *)

open Hardcaml
open Signal

module Make (D : sig
    val depth : int
  end) : Mem_port.S = struct
  let depth = D.depth

  let create ~clock ~clear (req : Signal.t Mem_port.Req.t) : Signal.t Mem_port.Rsp.t =
    let spec = Reg_spec.create ~clock ~clear () in
    let abits = Int.ceil_log2 depth in
    let q =
      Ram.create
        ~collision_mode:Read_before_write
        ~size:depth
        ~write_ports:
          [| { write_clock = clock
             ; write_address = sel_bottom req.wr_addr abits
             ; write_enable = req.wr_valid
             ; write_data = req.wr_data
             }
          |]
        ~read_ports:
          [| { read_clock = clock
             ; read_address = sel_bottom req.rd_addr abits
             ; read_enable = req.rd_valid
             }
          |]
        ()
    in
    { ready = vdd
    ; wr_ready = vdd
    ; rd_ready = vdd
    ; rd_data_valid = reg spec req.rd_valid
    ; rd_data = q.(0)
    }
  ;;
end
