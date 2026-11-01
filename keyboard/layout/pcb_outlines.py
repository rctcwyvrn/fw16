"""Board outlines for KiCad: Ergogen's `pcb` outline as Edge.Cuts DXFs for both keyboard boards.

The `pcb` outline (b-6col-corne-3fan.yaml) is y-up in the CSV frame; KiCad negates y on DXF import, so importing it at
origin 0,0 puts it in positions-b.csv's KiCad frame. The right board is the mirror (pcb-spec section 7): x -> -x.
The mirror is done entity by entity: ezdxf's transform() with a negative scale left ARCs unmirrored (2026-11-01).

The right board's outer edge (x = +10 after mirroring) moves out by RIGHT_OUTER_GROW: the unmirrored Choc hot-swap
footprint puts its socket pad on the outer side there, 0.425 mm from the edge (user, 2026-11-01). Every line endpoint
and arc centre at x >= OUTER_X moves with it; the corner arcs' neighbouring lines keep their far ends.

usage: python3 pcb_outlines.py <ergogen out dir>   (e.g. _build/keyboard/layout/b-6col-corne-3fan)
writes keyboard/pcb/left/outline.dxf and keyboard/pcb/right/outline.dxf
"""
import os
import sys

import ezdxf

RIGHT_OUTER_GROW = 1.5
OUTER_X = 8.99  # the right board's outer edge is x = 10 with R1 corners centred at x = 9

src = os.path.join(sys.argv[1], "outlines", "pcb.dxf")
pcb = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "pcb")


def grow(x):
    return x + RIGHT_OUTER_GROW if x >= OUTER_X else x


def write(side, entities):
    doc = ezdxf.new()
    doc.header["$INSUNITS"] = 4  # mm
    msp = doc.modelspace()
    for kind, a in entities:
        if kind == "line":
            msp.add_line(a[0], a[1])
        else:
            msp.add_arc(a[0], a[1], a[2], a[3])
    out = os.path.join(pcb, side, "outline.dxf")
    doc.saveas(out)
    print("wrote", os.path.relpath(out), len(entities), "entities")


left = []
for e in ezdxf.readfile(src).modelspace():
    if e.dxftype() == "LINE":
        left.append(("line", ((e.dxf.start.x, e.dxf.start.y), (e.dxf.end.x, e.dxf.end.y))))
    elif e.dxftype() == "ARC":
        c = e.dxf.center
        left.append(("arc", ((c.x, c.y), e.dxf.radius, e.dxf.start_angle, e.dxf.end_angle)))
    else:
        raise SystemExit(f"unexpected {e.dxftype()} in {src}")
write("left", left)

right = []
for kind, a in left:
    if kind == "line":
        (x0, y0), (x1, y1) = a
        right.append(("line", ((grow(-x0), y0), (grow(-x1), y1))))
    else:
        (cx, cy), r, s, e = a
        # mirror in x: angle t -> 180 - t, and the sweep reverses, so start/end swap
        right.append(("arc", ((grow(-cx), cy), r, (180 - e) % 360, (180 - s) % 360)))
write("right", right)
