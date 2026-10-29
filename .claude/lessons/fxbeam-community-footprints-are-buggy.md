# Check FXBeam footprints against the Neoconix drawing, by position, reading `drill` not pad `size`

jyancat's footprint names the PJ6/PJ7/PJ8 pads PJ1/PJ2/PJ3 (as shipped: GND shorted to ID0, Fan1_SPEED,
USB+) and lacks the back-side SMT rings; XPModder's is correct apart from leaving mask margin at the board
default. The authority is sheet 2 of `ExpansionBay/Electrical/Dual-channel 74P FXBeam … Rev 2 (Public
version).pdf` ("Recommend DB footprint") plus the README pin table's Column/Row fields — encoded in
`bay/tools/fxbeam_pads.py`, checked by `bay/tools/fxbeam_check.py`.

Two traps hit on the way: both fields share one grid, so a best-fit translation between footprints can align
power onto signal (anchor on the Ø2.70 screw holes); and an NPTH's `(size …)` is its pad/mask opening, not its
diameter — an early script read XPModder's 1.6/3.2 sizes as wrong holes when the drills were right.
