(* One register of the shell's AXI-Lite map: the types every generator and the
   gateware register file read. Offsets are byte addresses within BAR0. *)

type access =
  | RO  (** read-only; writes ignored *)
  | RW  (** read/write, value held *)
  | WO_PULSE  (** write-only; each written 1 bit pulses for one cycle, reads 0 *)

type field =
  { fname : string
  ; lsb : int
  ; width : int
  ; fdoc : string
  }

type t =
  { name : string
  ; offset : int
  ; access : access
  ; reset : int
  ; doc : string
  ; fields : field list  (** empty = the whole 32-bit word is one value *)
  }

type block =
  { bname : string
  ; base : int
  ; bdoc : string
  ; regs : t list  (** offsets are absolute, not relative to [base] *)
  }

let block_size = 0x1000

let access_to_string = function
  | RO -> "RO"
  | RW -> "RW"
  | WO_PULSE -> "WO (pulse)"
;;

let field ?(width = 1) fname lsb fdoc = { fname; lsb; width; fdoc }

let reg ?(reset = 0) ?(fields = []) name offset access doc =
  { name; offset; access; reset; doc; fields }
;;
