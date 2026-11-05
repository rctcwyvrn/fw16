# KiCad IPC rotates rounded-rect footprints only in 90° steps

Konnect's `rotate_component` over IPC rejects any non-multiple-of-90° angle for a footprint that holds a rounded-rect
graphic (KiCad 10.0.7): "cannot rotate a footprint containing a rounded-rectangle graphic by a non-90° angle". Stock
0201/0402 footprints count. Plan diagonal placements (e.g. a 45° cap across a BGA via cell) only if you're willing to do
the rotation in the GUI. Otherwise place square to the grid and bridge to the vias with short stubs, as
`fpga/bay/tools/bga_caps_place.py` does.
