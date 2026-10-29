(* AXI4-Lite channels as seen by a 32-bit slave: master-driven signals in
   [M2s], slave-driven in [S2m]. Addresses are byte offsets within BAR0. *)


let addr_width = 32

module M2s = struct
  type 'a t =
    { awvalid : 'a
    ; awaddr : 'a [@bits addr_width]
    ; wvalid : 'a
    ; wdata : 'a [@bits 32]
    ; wstrb : 'a [@bits 4]
    ; bready : 'a
    ; arvalid : 'a
    ; araddr : 'a [@bits addr_width]
    ; rready : 'a
    }
  [@@deriving hardcaml]
end

module S2m = struct
  type 'a t =
    { awready : 'a
    ; wready : 'a
    ; bvalid : 'a
    ; bresp : 'a [@bits 2]
    ; arready : 'a
    ; rvalid : 'a
    ; rdata : 'a [@bits 32]
    ; rresp : 'a [@bits 2]
    }
  [@@deriving hardcaml]
end
