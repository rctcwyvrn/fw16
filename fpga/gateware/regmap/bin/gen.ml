(* gen c|md — print map v0 as a C header or Markdown; fails on a malformed map. *)

open Regmap

let () =
  let blocks = Map_v0.blocks in
  (match Check.errors blocks with
   | [] -> ()
   | errs ->
     List.iter prerr_endline errs;
     exit 1);
  let major = Map_v0.version_major and minor = Map_v0.version_minor in
  match Sys.argv with
  | [| _; "c" |] -> print_string (To_c.render ~major ~minor blocks)
  | [| _; "md" |] -> print_string (To_markdown.render ~major ~minor blocks)
  | _ ->
    prerr_endline "usage: gen c|md";
    exit 2
;;
