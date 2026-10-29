"""Board outline and reference holes, from the bare-PCB STEP's top face, as KiCad-ready DXFs.

Framework's own `fw_expansion_bay_pcb_0*_0807.dxf` are A2 drawing sheets, not outlines, so the outline
comes from `PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.stp`: the solid about 118.65 mm wide, its
planar face at the board top, outer wire -> card-outline.dxf (import as Edge.Cuts), inner wires ->
card-holes.dxf (reference only: KiCad mounting holes should be footprints placed on these centres).
Coordinates are the card frame (card_frame.py).

usage: python3 card_outline.py <FW_EXPANSION_BAY_PCB_0807.stp> <out_dir>
"""
import os
import sys

import ezdxf
from ezdxf.math import ConstructionArc
from OCC.Core.BRepAdaptor import BRepAdaptor_Curve, BRepAdaptor_Surface
from OCC.Core.BRepTools import breptools
from OCC.Core.GeomAbs import GeomAbs_Circle, GeomAbs_Line, GeomAbs_Plane
from OCC.Core.TopAbs import TopAbs_EDGE, TopAbs_FACE, TopAbs_WIRE
from OCC.Core.TopExp import TopExp_Explorer

from card_frame import PCB_TOP_Y, to_dxf
from step_rays import load_solids


def iter_sub(shape, kind):
    exp = TopExp_Explorer(shape, kind)
    while exp.More():
        yield exp.Current()
        exp.Next()


def top_face(solid):
    for f in iter_sub(solid.shape, TopAbs_FACE):
        s = BRepAdaptor_Surface(f)
        if s.GetType() == GeomAbs_Plane and abs(s.Plane().Axis().Direction().Y()) > 0.99 \
                and abs(s.Plane().Location().Y() - PCB_TOP_Y) < 0.05:
            return f
    raise ValueError("no planar face at the board top")


def add_edge(msp, edge):
    c = BRepAdaptor_Curve(edge)
    a, b = c.FirstParameter(), c.LastParameter()
    p, q, m = (to_dxf(v.X(), v.Z()) for v in (c.Value(a), c.Value(b), c.Value((a + b) / 2)))
    if c.GetType() == GeomAbs_Line:
        msp.add_line(p, q)
    elif c.GetType() == GeomAbs_Circle and abs(b - a) < 6.28:
        ConstructionArc.from_3p(p, q, m).add_to_layout(msp)
    elif c.GetType() == GeomAbs_Circle:
        circ = c.Circle()
        msp.add_circle(to_dxf(circ.Location().X(), circ.Location().Z()), circ.Radius())
    else:
        raise ValueError("unexpected edge type %s" % c.GetType())


def main(step_path, out_dir):
    pcb = [s for s in load_solids(step_path)
           if abs(s.hi[0] - s.lo[0] - 118.65) < 1.5 and s.hi[1] - s.lo[1] < 2.0]
    if len(pcb) != 1:
        raise ValueError("expected one board solid, found %d" % len(pcb))
    face = top_face(pcb[0])
    outer = breptools.OuterWire(face)
    outline, holes = ezdxf.new(units=4), ezdxf.new(units=4)
    n_holes = 0
    for w in iter_sub(face, TopAbs_WIRE):
        doc = outline if w.IsSame(outer) else holes
        n_holes += 0 if w.IsSame(outer) else 1
        for e in iter_sub(w, TopAbs_EDGE):
            add_edge(doc.modelspace(), e)
    outline.saveas(os.path.join(out_dir, "card-outline.dxf"))
    holes.saveas(os.path.join(out_dir, "card-holes.dxf"))
    print("outline + %d holes -> %s" % (n_holes, out_dir))


if __name__ == "__main__":
    main(*sys.argv[1:3])
