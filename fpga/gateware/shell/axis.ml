(* The shell's AXI-Stream beat: 256-bit data to match XDMA's 256-bit AXI at
   250 MHz (Gen4 x4 / Gen3 x8 rate). Source-side fields only; tready travels
   the other way as a plain signal. *)


let data_width = 256

type 'a t =
  { tvalid : 'a
  ; tdata : 'a [@bits data_width]
  ; tkeep : 'a [@bits data_width / 8]
  ; tlast : 'a
  }
[@@deriving hardcaml]
