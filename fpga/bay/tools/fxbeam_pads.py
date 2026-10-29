"""The FXBeam card-side pad field: Neoconix drawing geometry joined with Framework's pin table.

Geometry is transcribed from `Dual-channel 74P FXBeam, length 27.75mm _Rev 2_ (Public version).pdf`,
sheet 2 "Recommend DB footprint" (DWG FXBM-X148F027A). Signal names come from the pin tables in
`ExpansionBay/Electrical/README.md` (Pin No. -> Column/Row). The footprint frame used here puts the origin
midway between the two Ø2.70 screw holes, x right, y down (KiCad convention).

usage: python3 fxbeam_pads.py <ExpansionBay/Electrical/README.md>   -> JSON on stdout
"""
import json
import re
import sys

# Drawing frame: origin at hole E, y up. Converted to the footprint frame by to_fp().
ORIGIN = (14.665, 4.34)
PAD_W, PAD_H = 0.70, 0.40  # x by y
MASK_MARGIN = 0.05  # drawing note: mask opening >= 0.5 x 0.8 mm
ROWS = "ABCDEFGHJKL"
ROW_PITCH = 0.60
# Column x for columns 8..1 (left to right) in each field.
COL_X = {
    "P": dict(zip(range(8, 0, -1), (0.96, 1.82, 4.26, 5.12, 7.56, 8.42, 10.86, 11.72))),
    "S": dict(zip(range(8, 0, -1), (17.61, 18.47, 20.91, 21.77, 24.21, 25.07, 27.51, 28.37))),
}
# y of row A (top) in each field; signal rows sit 0.25 mm above power rows.
ROW_A_Y = {"P": 7.22, "S": 7.47}
# Non-plated holes: (x, y, diameter). The two Ø2.70 take the interposer screws.
HOLES = [
    (0.00, 0.00, 1.50), (12.68, 0.00, 1.50), (16.65, 8.68, 1.50), (29.33, 8.68, 1.50),
    (6.34, 8.54, 1.40), (22.99, 0.14, 1.40),
    (6.34, 4.34, 2.70), (22.99, 4.34, 2.70),
]
# SMT rings around the screw holes, back side: (x, y, outer diameter, inner diameter).
RINGS = [(6.34, 4.34, 4.50, 3.20), (22.99, 4.34, 4.50, 3.20)]
# Connector keep-out boxes per field: (x0, y0, x1, y1).
KEEPOUT = {"P": (-0.71, -0.71, 13.39, 9.39), "S": (15.94, -0.71, 30.04, 9.39)}


def to_fp(x, y):
    return round(x - ORIGIN[0], 3), round(ORIGIN[1] - y, 3)


def readme_tables(path):
    """{'P': {pin: (signal, col, row_letter)}, 'S': {...}} from the README's two pin tables."""
    out, cur = {}, None
    for line in open(path, encoding="utf-8"):
        if line.startswith("### Power"):
            cur = out.setdefault("P", {})
        elif line.startswith("### Signal"):
            cur = out.setdefault("S", {})
        elif line.startswith("## "):
            cur = None
        elif cur is not None and re.match(r"^\|\s*\d+\s*\|", line):
            f = [c.strip() for c in line.split("|")]
            cur[int(f[1])] = (f[2], int(f[-3]), f[-2][1])
    return out


def pads(readme):
    """One record per contact pad: number (P1..P74, S1..S74), signal, field row/col, footprint x/y."""
    res = []
    for field, table in readme_tables(readme).items():
        for pin, (signal, col, row) in sorted(table.items()):
            x = COL_X[field][col]
            y = ROW_A_Y[field] - ROW_PITCH * ROWS.index(row)
            fx, fy = to_fp(x, y)
            res.append({"number": f"{field}{pin}", "signal": signal, "field": field,
                        "row": row, "col": col, "x": fx, "y": fy})
    return res


def mechanical():
    return {
        "holes": [dict(zip(("x", "y"), to_fp(x, y)), d=d) for x, y, d in HOLES],
        "rings": [dict(zip(("x", "y"), to_fp(x, y)), od=od, id=i) for x, y, od, i in RINGS],
        "keepout": {f: [*to_fp(x0, y1), *to_fp(x1, y0)] for f, (x0, y0, x1, y1) in KEEPOUT.items()},
        "pad": {"w": PAD_W, "h": PAD_H, "mask_margin": MASK_MARGIN},
    }


if __name__ == "__main__":
    print(json.dumps({"pads": pads(sys.argv[1]), **mechanical()}, indent=1))
