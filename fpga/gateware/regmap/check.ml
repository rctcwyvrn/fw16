(* Structural checks on a register map: every register inside its own block,
   aligned, unique, and every field inside 32 bits without overlap. *)

open Reg

let errors (blocks : block list) =
  let errs = ref [] in
  let err fmt = Printf.ksprintf (fun s -> errs := s :: !errs) fmt in
  let seen = Hashtbl.create 64 in
  List.iter
    (fun b ->
      if b.base mod block_size <> 0 then err "%s: base 0x%x not 4 KiB aligned" b.bname b.base;
      List.iter
        (fun r ->
          if r.offset mod 4 <> 0 then err "%s: offset 0x%x not 4-byte aligned" r.name r.offset;
          if r.offset < b.base || r.offset >= b.base + block_size
          then err "%s: offset 0x%x outside block %s" r.name r.offset b.bname;
          (match Hashtbl.find_opt seen r.offset with
           | Some other -> err "%s and %s share offset 0x%x" r.name other r.offset
           | None -> Hashtbl.add seen r.offset r.name);
          let used = ref 0 in
          List.iter
            (fun f ->
              if f.lsb < 0 || f.width < 1 || f.lsb + f.width > 32
              then err "%s.%s: bits outside [31:0]" r.name f.fname
              else (
                let mask = ((1 lsl f.width) - 1) lsl f.lsb in
                if !used land mask <> 0 then err "%s.%s overlaps another field" r.name f.fname;
                used := !used lor mask))
            r.fields)
        b.regs)
    blocks;
  List.rev !errs
;;
