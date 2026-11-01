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
- KiCad's IPC refuses a non-90° rotation for a footprint that contains a rounded-rectangle graphic ("use a multiple of
  90°"); Konnect's `create_footprint` gives its outlines rounded corners, so fw16kb's Choc thumb keys (−12°, −24°)
  had to be rotated in the GUI. `flip_component` to B.Cu mirrors the reported rotation (−12 → +12, and the pad line
  tilts the wrong way): set the rotation again after flipping and check a pad position.

## Edge connectors: check the mouth after placing or mirroring (2026-11-01)
- Both keyboard boards had J1 (HRO TYPE-C-31-M-12) backwards: origin on the edge with pads 4 mm outside the board,
  mouth inward. That footprint has pads at local y −4.045 and the shell front at +3.9; KiCad's rotation θ maps
  local (x, y) → board (x cosθ + y sinθ, −x sinθ + y cosθ), so read one pad back with `get_pad_position` and confirm
  it lands inside the outline. The pre-route DRC showed only "J1 shield 0.25 mm from edge"; the pads outside the
  board raised nothing until tracks reached them.
- Konnect refuses a closed-board file edit (`unsafe_file_fallback`, `board_previously_observed_live`) once it has seen
  that board open in KiCad this server session: reopen the board and edit over IPC instead.
- Netclass writes to `.kicad_pro` are lost if KiCad still has the project open; check for KiCad's process
  (`check_kicad_ui`) before writing, not the user's memory of closing it.
