# Freerouting has no neck-down, so a netclass width or clearance too big for a fine-pitch pad leaves that pad unrouted

On the right keyboard board (2026-11-01), Freerouting 2.5.0 routed everything except the USB-C (HRO TYPE-C-31-M-12)
/ USBLC6 cluster while the Power class was 0.5 mm and clearance 0.2 mm: J1's pads are 0.3 mm (0.6 mm for VBUS/GND)
at 0.5 mm pitch, and Freerouting uses a class's width for the whole trace with no neck-down at the pad. Dropping
Power to 0.3 mm and clearance to 0.15 mm (still above JLC's 2-layer 0.127) took it from 3 unrouted + 8 violations to
1 unrouted + 0 violations in half the time.

- Size Power and clearance for the finest pad a class touches before the DSN export, or plan to hand-route that area.
- Experiment on a copy of the exported `.dsn` (`(width …)` / `(clearance …)` in its `(structure (rule …))` and
  `(network (class …))` blocks) instead of round-tripping through KiCad; then set the same values in `.kicad_pro`
  before importing the SES, or KiCad's DRC flags every gap the router took below the project's clearance.
- The log's "could not be routed" list can name more connections than the final "(N unrouted …)" score; trust
  KiCad's DRC after the SES import.
- Run it as `freerouting --gui.enabled=false -de in.dsn -do out.ses -mp 100` in the dev shell; about 30 s here.
