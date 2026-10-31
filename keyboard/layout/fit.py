"""Tray-fit check: two half outlines side by side in the belly tray's inner area.

Tray outer ≈ 356 x 270 mm (keyboard/tray/KEEPOUT.md v1: footprint ±178.37 x
270.07), minus an assumed 3 mm printed wall each side -> 350 x 264 inner. The
halves stow in typing orientation, mirrored, with a 5 mm gap between them for
fingers to lift them out. The case outline already includes the case-wall
allowance (see the candidate configs' `wall` unit).
"""

from dxf_outline import bbox

TRAY_OUTER = (356.0, 270.0)
TRAY_WALL = 3.0
GAP = 5.0
INNER = (TRAY_OUTER[0] - 2 * TRAY_WALL, TRAY_OUTER[1] - 2 * TRAY_WALL)

HEADER = ("| Candidate | Keys/half (total) | Half outline W × D (mm) | Two halves + gap W × D "
          f"| Tray inner {INNER[0]:.0f} × {INNER[1]:.0f}: spare W × D | Fits |\n"
          "| --- | --- | --- | --- | --- | --- |")


def row(name, case, keys_per_half):
    x0, y0, x1, y1 = bbox(case)
    w, d = x1 - x0, y1 - y0
    pair_w = 2 * w + GAP
    spare_w, spare_d = INNER[0] - pair_w, INNER[1] - d
    ok = "yes" if spare_w >= 0 and spare_d >= 0 else "NO"
    return (f"| {name} | {keys_per_half} ({2 * keys_per_half}) | {w:.1f} × {d:.1f} "
            f"| {pair_w:.1f} × {d:.1f} | {spare_w:.1f} × {spare_d:.1f} | {ok} |")
