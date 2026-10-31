(* AXI4 (full) channels of XDMA's bypass master (m_axib_*, BAR2) as the slot
   sees them: master-driven signals in [M2s], slave-driven in [S2m]. Widths
   are XDMA 4.2's, so the top wires each port one to one: 64-bit addresses
   (BAR2 offsets, translation 0), 256-bit data, 4-bit IDs. *)

let addr_width = 64
let data_width = 256
let id_width = 4

module M2s = struct
  type 'a t =
    { awid : 'a [@bits id_width]
    ; awaddr : 'a [@bits addr_width]
    ; awlen : 'a [@bits 8]
    ; awsize : 'a [@bits 3]
    ; awburst : 'a [@bits 2]
    ; awprot : 'a [@bits 3]
    ; awlock : 'a
    ; awcache : 'a [@bits 4]
    ; awvalid : 'a
    ; wdata : 'a [@bits data_width]
    ; wstrb : 'a [@bits data_width / 8]
    ; wlast : 'a
    ; wvalid : 'a
    ; bready : 'a
    ; arid : 'a [@bits id_width]
    ; araddr : 'a [@bits addr_width]
    ; arlen : 'a [@bits 8]
    ; arsize : 'a [@bits 3]
    ; arburst : 'a [@bits 2]
    ; arprot : 'a [@bits 3]
    ; arlock : 'a
    ; arcache : 'a [@bits 4]
    ; arvalid : 'a
    ; rready : 'a
    }
  [@@deriving hardcaml]
end

module S2m = struct
  type 'a t =
    { awready : 'a
    ; wready : 'a
    ; bid : 'a [@bits id_width]
    ; bresp : 'a [@bits 2]
    ; bvalid : 'a
    ; arready : 'a
    ; rid : 'a [@bits id_width]
    ; rdata : 'a [@bits data_width]
    ; rresp : 'a [@bits 2]
    ; rlast : 'a
    ; rvalid : 'a
    }
  [@@deriving hardcaml]
end
