"""Height zones over the card, as KiCad-ready DXFs: one rectangle per region of equal clearance,
labelled with the maximum part height there.

Clearance is measured by vertical rays from the reference board's faces (card_frame.PCB_TOP_Y /
PCB_BOT_Y) on a 1 mm grid, to the first solid in the union of:
  - `FW_EXPANSION_BAY_PCB_REF__0807.stp` minus the board itself: shell, floor plate, rear bracket,
    bosses, interposer, standoffs, inserts and the reference fan connectors, all of which a card must
    clear or carry in the same place;
  - `Framework_16_all_UMA_asm_0408.stp` (FW16 repo zip): the chassis and input modules, the only
    source of the ceiling.
Each cell is one ray at its centre; to cover what falls between rays, each cell takes the smallest gap
of its 3 x 3 neighbourhood (obstacles grow by 1 mm). That gap is floored to a design class (CLASSES),
then cells merge into rectangles. Labels are gaps, no
tolerance taken off. Numbers assume a 1.2 mm board.

The raycast takes ~9 min; it saves the raw gaps as card-zones-grid.json so the classes can be
changed and the DXFs rewritten without re-casting.

usage: python3 card_zones.py <PCB_REF__0807.stp> <Framework_16_all_UMA_asm_0408.stp> <out_dir>
       python3 card_zones.py <card-zones-grid.json> <out_dir>
  -> card-zones-top.dxf, card-zones-bottom.dxf (+ card-zones-grid.json from a raycast)
"""
import json
import os
import sys

import ezdxf

from card_frame import PCB_BOT_Y, PCB_TOP_Y, to_dxf
from step_rays import first_hit, load_solids

X0, X1, Z0, Z1 = -57.45, 61.20, 4.0, 76.5  # board bounding box
STEP = 1.0
NONE = 99.0  # nothing hit
# Height classes a placer cares about (mm above/below the board face). The odd ones are KEEPOUT
# bands: 1.4 floor gap, 1.85 rear bracket, 7.8 front and 8.65 middle ceilings.
CLASSES = (0.0, 0.5, 1.0, 1.4, 1.85, 2.5, 3.5, 4.5, 5.5, 6.5, 7.8, 8.65)


def grid(solids, y0, sign):
    xs = [X0 + STEP * (i + 0.5) for i in range(int((X1 - X0) / STEP))]
    zs = [Z0 + STEP * (j + 0.5) for j in range(int((Z1 - Z0) / STEP))]
    cells = []
    for j, z in enumerate(zs):
        if j % 10 == 0:
            print("  %s row %d/%d" % ("top" if sign > 0 else "bot", j, len(zs)), file=sys.stderr, flush=True)
        row = []
        for x in xs:
            d = first_hit(solids, x, y0, z, sign)
            row.append(NONE if d is None else round(d, 4))
        cells.append(row)
    return xs, zs, cells


def neighbourhood_min(cells):
    """Each cell's gap becomes the smallest of itself and its 8 neighbours. A centre ray misses
    anything between rays (keyboard ribs, grille webs, contacts); this makes every rectangle safe
    for any point inside it, at the cost of growing each obstacle by one cell."""
    n, m = len(cells), len(cells[0])
    return [[min(cells[jj][ii] for jj in range(max(0, j - 1), min(n, j + 2))
                 for ii in range(max(0, i - 1), min(m, i + 2)))
             for i in range(m)] for j in range(n)]


def classify(cells):
    """Floor each raw gap to the highest design class at or below it (0.02 mm slack for ray noise)."""
    def cls(d):
        if d >= NONE:
            return NONE
        return max(c for c in CLASSES if c <= d + 0.02)
    return [[cls(d) for d in row] for row in cells]


def rectangles(xs, zs, cells):
    """Runs of equal value along X, merged down Z when span and value match."""
    open_, done = {}, []
    for j, row in enumerate(cells):
        runs, i = {}, 0
        while i < len(row):
            k = i
            while k + 1 < len(row) and row[k + 1] == row[i]:
                k += 1
            runs[(i, k, row[i])] = True
            i = k + 1
        for key in list(open_):
            if key not in runs:
                done.append(key + (open_.pop(key), j - 1))
        for key in runs:
            open_.setdefault(key, j)
    done += [key + (j0, len(cells) - 1) for key, j0 in open_.items()]
    h = STEP / 2
    return [(xs[i0] - h, xs[i1] + h, zs[j0] - h, zs[j1] + h, v) for i0, i1, v, j0, j1 in done]


def write(rects, path, side):
    doc = ezdxf.new(units=4)
    msp = doc.modelspace()
    for x0, x1, z0, z1, v in rects:
        corners = [to_dxf(x0, z0), to_dxf(x1, z0), to_dxf(x1, z1), to_dxf(x0, z1)]
        msp.add_lwpolyline(corners, close=True)
        w, d = x1 - x0, z1 - z0
        if w * d >= 4:
            label = "%s %s" % (side, "open" if v >= NONE else "%.2f" % v)
            size = min(1.2, 0.8 * d, 1.2 * w / max(len(label), 1))
            cx, cy = to_dxf((x0 + x1) / 2, (z0 + z1) / 2)
            msp.add_text(label, height=size).set_placement((cx, cy), align=ezdxf.enums.TextEntityAlignment.MIDDLE_CENTER)
    doc.saveas(path)


def raycast(pcbref_path, fw16_path):
    ref = [s for s in load_solids(pcbref_path)
           if not (abs(s.hi[0] - s.lo[0] - 118.65) < 1.5 and s.hi[1] - s.lo[1] < 2.0)]
    solids = ref + load_solids(fw16_path)
    return {side: grid(solids, y0, sign) for side, y0, sign in (("top", PCB_TOP_Y, +1),
                                                                ("bot", PCB_BOT_Y, -1))}


def main(*args):
    if len(args) == 2:  # re-quantise a saved grid, no raycasting
        grid_path, out_dir = args
        with open(grid_path) as f:
            grids = json.load(f)
    else:
        pcbref_path, fw16_path, out_dir = args
        grids = raycast(pcbref_path, fw16_path)
        with open(os.path.join(out_dir, "card-zones-grid.json"), "w") as f:
            json.dump(grids, f)
    for side, name in (("top", "card-zones-top.dxf"), ("bot", "card-zones-bottom.dxf")):
        xs, zs, cells = grids[side]
        rects = rectangles(xs, zs, classify(neighbourhood_min(cells)))
        write(rects, os.path.join(out_dir, name), side)
        area = {}
        for x0, x1, z0, z1, v in rects:
            area[v] = area.get(v, 0) + (x1 - x0) * (z1 - z0)
        print("%s: %d rectangles; mm² by max height:" % (side, len(rects)))
        for v in sorted(area):
            print("  %6s  %7.0f" % ("open" if v >= NONE else "%.2f" % v, area[v]))


if __name__ == "__main__":
    main(*sys.argv[1:])
