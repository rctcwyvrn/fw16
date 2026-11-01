# The card's 199 `lib_footprint_mismatch` DRC warnings are only the 3D-model path: `.step` (Windows KiCad) vs `.stpZ` (nix)

The board's footprints come from the Windows KiCad 10 library (placed through live IPC), and DRC compares them with
the nix `kicad-footprints` copy; with CRLF and UUIDs stripped, R_0402 differs only in
`R_0402_1005Metric.step` vs `.stpZ` (2026-11-01). Geometry is identical, so the warning is safe to ignore.
