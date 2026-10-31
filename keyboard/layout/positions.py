"""Footprint placement table for the layout-B keyboard PCBs (left and right board).

Run from the repo root inside the dev shell, after build.py has written Ergogen's
debug points:   python3 keyboard/layout/positions.py > keyboard/layout/positions-b.csv

Coordinates are KiCad's (mm, y down, rotation CCW-positive as KiCad displays it),
origin at the centre of the left half's outer-bottom key. The right board is the
mirror image: x -> -x, rotation -> -r, with the *same, unmirrored* footprints
(the hot-swap socket is chiral; only positions mirror). The PCB edge is the
`pcb` outline of the config (key pitch boxes + controller zone, expanded 1 mm).
Rows are what keyboard/docs/pcb-spec.md section 6 places; it explains the refs.
"""

import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from points_yaml import read_points  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(HERE))
POINTS = os.path.join(ROOT, "_build", "keyboard", "layout", "b-6col-corne-3fan", "points", "points.yaml")

KX, KY = 18.0, 17.0           # key pitch (units kx, ky in the config)
MCUX = 22.0                   # controller zone width (unit mcux)
PCB_EXPAND = 1.0              # pcb outline expansion beyond pitch boxes / zone
DIODE_DY = 4.7                # diode centre, key-local, towards the user (KiCad +y)
MODULE_LEN = 23.5             # RP2040-Zero, USB end to far end (Waveshare drawing; verify)
LINK_CY_FROM_ZONE_TOP = 31.5  # link receptacle centre below the zone's top edge

COLS = ["outer", "pinky", "ring", "middle", "index", "inner"]
ROWS = ["top", "home", "bottom"]          # R0, R1, R2
THUMBS = ["tuck", "mid", "reach"]         # R3 x C3, C4, C5


def keys(points):
    """(ref_number, point_name, row, col) in schematic order: row-major, R0..R3."""
    out = []
    for r, row in enumerate(ROWS):
        for c, col in enumerate(COLS):
            out.append((f"matrix_{col}_{row}", r, c))
    for i, t in enumerate(THUMBS):
        out.append((f"thumb_{t}_cluster", 3, 3 + i))
    return [(n + 1, name, r, c) for n, (name, r, c) in enumerate(out) if name in points]


def kicad(x, y, r):
    """Ergogen frame (y up) -> KiCad frame (y down); rotation keeps its sign."""
    return x, -y, r


def rows_for(points, side):
    sx = 1 if side == "L" else -1
    lines = []

    def tidy(v):
        return 0.0 if abs(v) < 5e-4 else v   # no "-0.000"

    def emit(ref, part, layer, x, y, r, note):
        lines.append(f"{side},{ref},{part},{layer},{tidy(sx * x):.3f},{tidy(y):.3f},{tidy(r * sx):g},{note}")

    for n, name, r, c in keys(points):
        px, py, pr = points[name]
        x, y, rot = kicad(px, py, pr)
        emit(f"SW{n}", "choc_v1_hotswap", "F (sockets B)", x, y, rot, f"{name} R{r}xC{c}")
        a = math.radians(pr)
        # key-local (0, -DIODE_DY) in Ergogen's y-up frame, rotated with the key
        dx, dy = DIODE_DY * math.sin(a), -DIODE_DY * math.cos(a)
        dxk, dyk, _ = kicad(px + dx, py + dy, pr)
        emit(f"D{n}", "D_SOD-123", "B", dxk, dyk, rot, f"diode of SW{n}, cathode (pad 1) to R{r}")

    ix, iy, _ = points["matrix_inner_top"]
    zone_left = ix + KX / 2
    zone_top = -(iy + KY / 2)            # KiCad y of the zone's top edge
    zone_cx = zone_left + MCUX / 2
    edge_top = zone_top - PCB_EXPAND
    edge_inner = zone_left + MCUX + PCB_EXPAND
    emit("U1", "RP2040-Zero", "F", zone_cx, edge_top + MODULE_LEN / 2, 0,
         "module centre; USB-C end flush with the top (rear) PCB edge")
    emit("J1", "USB_C_HRO_TYPE-C-31-M-12", "F", edge_inner, zone_top + LINK_CY_FROM_ZONE_TOP,
         -90, "link port; mouth centre on the inner PCB edge, facing the other half")
    return lines


def main():
    points = read_points(POINTS)
    print("side,ref,part,layer,x_mm,y_mm,rot_deg,note")
    for side in ("L", "R"):
        print("\n".join(rows_for(points, side)))


if __name__ == "__main__":
    main()
