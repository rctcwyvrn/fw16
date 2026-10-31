"""Read an Ergogen outline DXF as polylines (lists of (x, y) in mm, y up).

Ergogen writes outlines as loose LINE and ARC entities (expanded corners are
arcs). Arcs are flattened to 0.05 mm so the template and the fit table work on
plain point lists and mirroring is just x -> -x.
"""

import ezdxf


def read_outline(path, sagitta=0.05):
    doc = ezdxf.readfile(path)
    polylines = []
    for e in doc.modelspace():
        kind = e.dxftype()
        if kind == "LINE":
            s, t = e.dxf.start, e.dxf.end
            polylines.append([(s.x, s.y), (t.x, t.y)])
        elif kind == "ARC":
            polylines.append([(v.x, v.y) for v in e.flattening(sagitta)])
        elif kind == "CIRCLE":
            polylines.append([(v.x, v.y) for v in e.flattening(sagitta)])
        else:
            raise ValueError(f"{path}: unexpected DXF entity {kind}")
    return polylines


def bbox(polylines):
    xs = [x for pl in polylines for x, _ in pl]
    ys = [y for pl in polylines for _, y in pl]
    return min(xs), min(ys), max(xs), max(ys)
