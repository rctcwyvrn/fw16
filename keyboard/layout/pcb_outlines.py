"""Board outlines for KiCad: Ergogen's `pcb` outline as Edge.Cuts DXFs for both keyboard boards.

The `pcb` outline (b-6col-corne-3fan.yaml) is y-up in the CSV frame; KiCad negates y on DXF import, so importing it at
origin 0,0 puts it in positions-b.csv's KiCad frame. The right board is the exact mirror (pcb-spec section 7): x -> -x.

usage: python3 pcb_outlines.py <ergogen out dir>   (e.g. _build/keyboard/layout/b-6col-corne-3fan)
writes keyboard/pcb/left/outline.dxf and keyboard/pcb/right/outline.dxf
"""
import os
import sys

import ezdxf
from ezdxf.math import Matrix44

src = os.path.join(sys.argv[1], "outlines", "pcb.dxf")
pcb = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "pcb")
for side, m in (("left", Matrix44()), ("right", Matrix44.scale(-1, 1, 1))):
    doc = ezdxf.readfile(src)
    for e in doc.modelspace():
        e.transform(m)
    out = os.path.join(pcb, side, "outline.dxf")
    doc.saveas(out)
    print("wrote", os.path.relpath(out))
