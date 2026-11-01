# ezdxf mirrors an ARC by flipping its extrusion to (0, 0, −1); its raw `dxf.center` is then in OCS and looks unmirrored

`entity.transform(Matrix44.scale(-1, 1, 1))` on an ARC keeps `dxf.center.x` positive and sets extrusion −Z, which is a
correct DXF mirror: KiCad imported it at the mirrored position. Reading `dxf.center` raw (2026-11-01) made the right
keyboard outline look broken when it wasn't. Use `ezdxf.math.OCS(e.dxf.extrusion).to_wcs(...)` when reading, or write
mirrored geometry as plain WCS lines/arcs (as `keyboard/layout/pcb_outlines.py` now does).
