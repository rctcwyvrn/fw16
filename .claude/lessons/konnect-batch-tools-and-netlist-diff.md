# Drive big Konnect sheet builds from a netlist spec with batch tools, and verify by netlist diff

Redrawing omnimeter's core_power (52 parts, 167 pins) in fw16, 2026-10-29:
- `generate_netlist` on another project's root sheet (output to a Windows temp path) gives a component-level spec
  (value, footprint, pin → net) without touching that project. The `.net` is multi-line S-expressions: parse it
  with a tokenizer, not line regexes. Net names contain spaces (`/Core power/VCCINT_SW`) — never split pin=net
  lists on whitespace.
- `batch_place_components` (52 parts) and `batch_connect_to_net` (one call per net, labels at pin ends, stub 0)
  do the build in ~30 calls. Their responses can exceed the tool-result limit; the write still happened — read the
  saved result file's `outcome` instead of retrying.
- `batch_connect_to_net` labels on vertical pins come out at rotation 0 and overlap neighbours; rotate them with
  `batch_rotate_labels` (90 for a pin pointing up, 270 for down).
- GND on many pins: place `power:GND` at each pin endpoint with `batch_place_components` (C/R/L pin 2 =
  (x, y + 3.81)), then `annotate_schematic` numbers the `#PWR?`.
- Proof of equivalence: export both netlists and compare every (ref, pin) → net through the designator map.
