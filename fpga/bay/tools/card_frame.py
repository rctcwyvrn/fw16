"""Card frame: maps Framework's STEP frame (KEEPOUT.md: X across, Y up, Z from the rear face) to the
card's KiCad frame.

The KiCad origin is the FXBeam footprint origin used by `fxbeam_pads.py`: midway between the two Ø2.70
interposer screw holes, at STEP (X 7.325, Z 70.55). Seen from the top (+Y), KiCad x = STEP X and KiCad
y (down) = STEP Z, so the rear of the bay is at the top of the KiCad sheet.

DXF is y-up and KiCad negates y on import, so DXF writers use `to_dxf`.
"""

ORIGIN_X, ORIGIN_Z = 7.325, 70.55
PCB_TOP_Y, PCB_BOT_Y = -9.45, -10.65  # reference board, 1.2 mm


def to_kicad(x, z):
    return (x - ORIGIN_X, z - ORIGIN_Z)


def to_dxf(x, z):
    kx, ky = to_kicad(x, z)
    return (kx, -ky)
