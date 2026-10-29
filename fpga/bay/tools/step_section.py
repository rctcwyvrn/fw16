"""Horizontal section (plane Y = const) of one or more STEP files, as a DXF in STEP X/Z: the outline of
every solid cut by the plane, one layer per input file. Used to read fan housings, duct walls and
openings that vertical rays can't see.

Coordinates stay in the STEP frame (KEEPOUT.md), not the card frame, because the fans and ducts lie
outside the card. DXF x = STEP X, DXF y = -STEP Z, so the rear face (Z 0) is at the top of the sheet,
the same way up as the card DXFs.

usage: bay-python step_section.py <y> <out.dxf> <x0> <x1> <z0> <z1> <file.stp> [<file.stp> ...]
  only solids whose bounding box meets the X/Z window are cut.
"""
import os
import sys

import ezdxf
from OCC.Core.BRepAdaptor import BRepAdaptor_Curve
from OCC.Core.BRepAlgoAPI import BRepAlgoAPI_Section
from OCC.Core.GCPnts import GCPnts_TangentialDeflection
from OCC.Core.TopAbs import TopAbs_EDGE
from OCC.Core.TopExp import TopExp_Explorer
from OCC.Core.gp import gp_Dir, gp_Pln, gp_Pnt

from step_rays import load_solids


def section_polylines(solids, y, window):
    x0, x1, z0, z1 = window
    plane = gp_Pln(gp_Pnt(0, y, 0), gp_Dir(0, 1, 0))
    out = []
    for s in solids:
        if not (s.lo[1] < y < s.hi[1]):
            continue
        if s.hi[0] < x0 or s.lo[0] > x1 or s.hi[2] < z0 or s.lo[2] > z1:
            continue
        sec = BRepAlgoAPI_Section(s.shape, plane, False)
        sec.Approximation(True)
        sec.Build()
        if not sec.IsDone():
            continue
        exp = TopExp_Explorer(sec.Shape(), TopAbs_EDGE)
        while exp.More():
            curve = BRepAdaptor_Curve(exp.Current())
            disc = GCPnts_TangentialDeflection(curve, 0.1, 0.05)
            pts = [disc.Value(i) for i in range(1, disc.NbPoints() + 1)]
            if len(pts) >= 2:
                out.append([(p.X(), p.Z()) for p in pts])
            exp.Next()
    return out


def main(y, out_path, x0, x1, z0, z1, *paths):
    y, window = float(y), tuple(float(v) for v in (x0, x1, z0, z1))
    doc = ezdxf.new(units=4)
    msp = doc.modelspace()
    for path in paths:
        layer = os.path.splitext(os.path.basename(path))[0][:31]
        doc.layers.add(layer)
        lines = section_polylines(load_solids(path), y, window)
        for pl in lines:
            msp.add_lwpolyline([(x, -z) for x, z in pl], dxfattribs={"layer": layer})
        print("%s: %d section edges at Y %.2f" % (layer, len(lines), y))
    doc.saveas(out_path)


if __name__ == "__main__":
    main(*sys.argv[1:])
