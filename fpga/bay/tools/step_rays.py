"""Located solids from Framework's STEP files, and the first solid hit along a vertical ray.

Every STEP is a Creo export in which every product is named `design`, so callers pick parts by geometry
(bounding box), not by name. hit() runs along ±Y only, which is all the keepout needs; cast() takes any
direction, for the fan and duct probes.
"""
from OCC.Core.Bnd import Bnd_Box
from OCC.Core.BRepClass3d import BRepClass3d_SolidClassifier
from OCC.Core.BRepBndLib import brepbndlib
from OCC.Core.IFSelect import IFSelect_RetDone
from OCC.Core.IntCurvesFace import IntCurvesFace_ShapeIntersector
from OCC.Core.STEPControl import STEPControl_Reader
from OCC.Core.TopAbs import TopAbs_IN, TopAbs_SOLID
from OCC.Core.TopExp import TopExp_Explorer
from OCC.Core.gp import gp_Dir, gp_Lin, gp_Pnt


class Solid:
    def __init__(self, shape, lo, hi, box=None):
        self.shape, self.lo, self.hi, self.box = shape, lo, hi, box
        self._isect = None

    def cast(self, p, d, far=400.0):
        """Distance from point p along unit direction d (3-tuples) to this solid's first face, or None.
        Unlike hit(), no inside test: a start inside the solid reports its exit face."""
        lin = gp_Lin(gp_Pnt(*p), gp_Dir(*d))
        if self.box is not None and self.box.IsOut(lin):
            return None
        if self._isect is None:
            self._isect = IntCurvesFace_ShapeIntersector()
            self._isect.Load(self.shape, 1e-4)
        s = self._isect
        s.Perform(lin, 0.0, far)
        if not s.IsDone() or s.NbPnt() == 0:
            return None
        return min(s.WParameter(k) for k in range(1, s.NbPnt() + 1))

    def hit(self, x, y0, z, sign):
        """Distance from (x, y0, z) along ±Y to this solid's first face, or None.
        0 when the start point is inside the solid (a part sunk into the board)."""
        if not (self.lo[0] <= x <= self.hi[0] and self.lo[2] <= z <= self.hi[2]):
            return None
        if (sign > 0 and self.hi[1] < y0) or (sign < 0 and self.lo[1] > y0):
            return None
        if self.lo[1] < y0 < self.hi[1]:
            c = BRepClass3d_SolidClassifier(self.shape, gp_Pnt(x, y0, z), 1e-4)
            if c.State() == TopAbs_IN:
                return 0.0
        if self._isect is None:
            self._isect = IntCurvesFace_ShapeIntersector()
            self._isect.Load(self.shape, 1e-4)
        s = self._isect
        s.Perform(gp_Lin(gp_Pnt(x, y0, z), gp_Dir(0, sign, 0)), 0.0, 200.0)
        if not s.IsDone() or s.NbPnt() == 0:
            return None
        return min(s.WParameter(k) for k in range(1, s.NbPnt() + 1))


def load_solids(path):
    reader = STEPControl_Reader()
    if reader.ReadFile(path) != IFSelect_RetDone:
        raise IOError("could not read STEP file: %s" % path)
    reader.TransferRoots()
    out = []
    exp = TopExp_Explorer(reader.OneShape(), TopAbs_SOLID)
    while exp.More():
        shape = exp.Current()
        box = Bnd_Box()
        brepbndlib.Add(shape, box)
        if not box.IsVoid():
            x0, y0, z0, x1, y1, z1 = box.Get()
            out.append(Solid(shape, (x0, y0, z0), (x1, y1, z1), box))
        exp.Next()
    return out


def first_hit(solids, x, y0, z, sign):
    best = None
    for s in solids:
        d = s.hit(x, y0, z, sign)
        if d is not None and (best is None or d < best):
            best = d
    return best
