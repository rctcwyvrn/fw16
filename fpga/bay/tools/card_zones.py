"""Height zones over the card, as KiCad-ready DXFs: one rectangle per region of equal clearance,
labelled with the maximum part height there.

Clearance is measured by vertical rays from the reference board's faces (card_frame.PCB_TOP_Y /
PCB_BOT_Y) on a 1 mm grid, to the first solid in the union of:
  - `FW_EXPANSION_BAY_PCB_REF__0807.stp` minus the board itself: shell, floor plate, rear bracket,
    bosses, interposer, standoffs, inserts and the reference fan connectors, all of which a card must
    clear or carry in the same place;
  - `Framework_16_all_UMA_asm_0408.stp` (FW16 repo zip): the chassis and input modules, the only
    source of the ceiling.
Each cell's clearance is floored to 0.05 mm, then cells merge into rectangles. Labels are raw gaps:
no tolerance taken off. Numbers assume a 1.2 mm board.

usage: python3 card_zones.py <PCB_REF__0807.stp> <Framework_16_all_UMA_asm_0408.stp> <out_dir>
  -> card-zones-top.dxf, card-zones-bottom.dxf
"""
import math
import os
import sys

import ezdxf

from card_frame import PCB_BOT_Y, PCB_TOP_Y, to_dxf
from step_rays import first_hit, load_solids

X0, X1, Z0, Z1 = -57.45, 61.20, 4.0, 76.5  # board bounding box
STEP = 1.0
NONE = 99.0  # nothing hit


def grid(solids, y0, sign):
    xs = [X0 + STEP * (i + 0.5) for i in range(int((X1 - X0) / STEP))]
    zs = [Z0 + STEP * (j + 0.5) for j in range(int((Z1 - Z0) / STEP))]
    cells = []
    for z in zs:
        row = []
        for x in xs:
            d = first_hit(solids, x, y0, z, sign)
            row.append(NONE if d is None else math.floor(d / 0.05 + 1e-6) * 0.05)
        cells.append(row)
    return xs, zs, cells


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


def main(pcbref_path, fw16_path, out_dir):
    ref = [s for s in load_solids(pcbref_path)
           if not (abs(s.hi[0] - s.lo[0] - 118.65) < 1.5 and s.hi[1] - s.lo[1] < 2.0)]
    solids = ref + load_solids(fw16_path)
    for side, y0, sign, name in (("top", PCB_TOP_Y, +1, "card-zones-top.dxf"),
                                 ("bot", PCB_BOT_Y, -1, "card-zones-bottom.dxf")):
        rects = rectangles(*grid(solids, y0, sign))
        write(rects, os.path.join(out_dir, name), side)
        area = {}
        for x0, x1, z0, z1, v in rects:
            area[v] = area.get(v, 0) + (x1 - x0) * (z1 - z0)
        print("%s: %d rectangles; mm² by max height:" % (side, len(rects)))
        for v in sorted(area):
            print("  %6s  %7.0f" % ("open" if v >= NONE else "%.2f" % v, area[v]))


if __name__ == "__main__":
    main(*sys.argv[1:4])
