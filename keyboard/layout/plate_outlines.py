"""Switch-plate outlines (decision 0003): one 1.2 mm FR4 plate per keyboard half, as an Edge.Cuts DXF for KiCad.

Reads the routed board itself (keyboard/pcb/<side>/fw16kb-<side>.kicad_pcb, KiCad frame, y down), so the plate
follows whatever the board holds:
- outline: the board's Edge.Cuts loop, inset by EDGE_INSET, minus the controller zone: the convex hull of the
  F.CrtYd of every top-side part taller than the 1 mm PCB-to-plate gap allows (the module, J1, the SOT-23s, the
  SOD-123, the 0805 cap and PTC), grown by ZONE_MARGIN. 0805 resistors (~0.5 mm) sit under the plate. BOOT stays
  reachable from above. Tongues narrower than MIN_FEATURE are opened away.
- a CUTOUT square at every switch, at the switch's rotation;
- an M2 clearance hole at each H* that the plate still covers with at least HOLE_RING of material. A hole without
  that ring is left off: its screw clamps the PCB alone.
The DXF is y-up (y -> -y), as `outline.dxf` is, so KiCad's DXF import at origin 0,0 lands it in the board's frame.

usage: python3 plate_outlines.py left|right
writes keyboard/pcb/plate-<side>/plate.dxf and prints the clearance checks
"""
import math
import os
import re
import sys

import ezdxf
from shapely.geometry import MultiPoint, Point, Polygon, box
from shapely.affinity import rotate
from shapely.ops import unary_union

CUTOUT = 13.8       # Choc v1 plate cutout
EDGE_INSET = 0.3    # plate edge inside the PCB edge
ZONE_MARGIN = 1.0   # clearance around the controller-zone parts
MIN_FEATURE = 3.2   # narrower FR4 tongues are removed (morphological opening, rounds outer corners by half this)
HOLE_D = 2.2        # M2 clearance
HOLE_RING = 1.0     # minimum plate material around a hole
ARC_STEPS = 24

side = sys.argv[1]
here = os.path.dirname(os.path.abspath(__file__))
pcb_dir = os.path.join(here, "..", "pcb")
board = open(os.path.join(pcb_dir, side, f"fw16kb-{side}.kicad_pcb")).read()


def arc_points(s, m, e):
    """Points along the circular arc through s, m, e (KiCad's start/mid/end form)."""
    (ax, ay), (bx, by), (cx, cy) = s, m, e
    d = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
    ux = ((ax * ax + ay * ay) * (by - cy) + (bx * bx + by * by) * (cy - ay) + (cx * cx + cy * cy) * (ay - by)) / d
    uy = ((ax * ax + ay * ay) * (cx - bx) + (bx * bx + by * by) * (ax - cx) + (cx * cx + cy * cy) * (bx - ax)) / d
    a0, am, a1 = (math.atan2(p[1] - uy, p[0] - ux) for p in (s, m, e))
    sweep = (a1 - a0) % (2 * math.pi)
    if (am - a0) % (2 * math.pi) > sweep:      # the mid point says we go the other way round
        sweep -= 2 * math.pi
    r = math.hypot(ax - ux, ay - uy)
    return [(ux + r * math.cos(a0 + sweep * i / ARC_STEPS), uy + r * math.sin(a0 + sweep * i / ARC_STEPS))
            for i in range(ARC_STEPS + 1)]


def edge_loop():
    num = r"([-\d.]+) ([-\d.]+)"
    segs = []
    for m in re.finditer(r"\(gr_(line|arc)(.*?)\(layer \"Edge.Cuts\"\)", board, re.S):
        body = m.group(2)
        pt = lambda k: tuple(map(float, re.search(r"\(%s %s\)" % (k, num), body).groups()))
        segs.append([pt("start"), pt("end")] if m.group(1) == "line" else arc_points(pt("start"), pt("mid"), pt("end")))
    loop = segs.pop(0)
    key = lambda p: (round(p[0], 3), round(p[1], 3))
    while segs:
        for i, s in enumerate(segs):
            if key(s[0]) == key(loop[-1]):
                loop += s[1:]
                break
            if key(s[-1]) == key(loop[-1]):
                loop += s[::-1][1:]
                break
        else:
            raise SystemExit("Edge.Cuts is not one closed loop")
        segs.pop(i)
    return Polygon(loop)


def footprints():
    """(reference, footprint name, layer, x, y, rotation, block text) for every footprint."""
    out = []
    starts = [m.start() for m in re.finditer(r"\n\t\(footprint \"", board)] + [len(board)]
    for a, b in zip(starts, starts[1:]):
        blk = board[a:b]
        name, layer = re.search(r"\(footprint \"([^\"]+)\"\s*\(layer \"([^\"]+)\"\)", blk).groups()
        x, y, r = re.search(r"\(at ([-\d.]+) ([-\d.]+)(?: ([-\d.]+))?\)", blk).groups()
        ref = re.search(r"\(property \"Reference\" \"([^\"]+)\"", blk).group(1)
        out.append((ref, name, layer, float(x), float(y), float(r or 0), blk))
    return out


def to_board(x, y, r, lx, ly):
    t = math.radians(r)
    return (x + lx * math.cos(t) + ly * math.sin(t), y - lx * math.sin(t) + ly * math.cos(t))


def courtyard_points(fp):
    ref, name, layer, x, y, r, blk = fp
    pts = []
    for item in blk.split("\n\t\t(")[1:]:                # one footprint child per chunk (KiCad's indentation)
        m = re.match(r"fp_(line|rect|poly|circle)\b", item)
        if not m or '(layer "F.CrtYd")' not in item:
            continue
        coords = [tuple(map(float, c)) for c in re.findall(r"\((?:start|end|xy|center) ([-\d.]+) ([-\d.]+)\)", item)]
        if m.group(1) == "rect":
            (x0, y0), (x1, y1) = coords
            coords = [(x0, y0), (x1, y0), (x1, y1), (x0, y1)]
        if m.group(1) == "circle":
            (cx, cy), (ex, ey) = coords
            rr = math.hypot(ex - cx, ey - cy)
            coords = [(cx - rr, cy - rr), (cx + rr, cy - rr), (cx + rr, cy + rr), (cx - rr, cy + rr)]
        pts += [to_board(x, y, r, *c) for c in coords]
    return pts


fps = footprints()
pcb = edge_loop()
plate = pcb.buffer(-EDGE_INSET, join_style="mitre")

zone_pts = []
for fp in fps:
    if fp[2] == "F.Cu" and not fp[0].startswith(("SW", "H", "R")):    # R*: 0805 resistors fit under the plate
        zone_pts += courtyard_points(fp)
zone = MultiPoint(zone_pts).convex_hull.buffer(ZONE_MARGIN, join_style="mitre")
plate = plate.difference(zone)
plate = plate.buffer(-MIN_FEATURE / 2).buffer(MIN_FEATURE / 2)   # opening: drop tongues narrower than MIN_FEATURE
if plate.geom_type == "MultiPolygon":                   # keep the key area, drop islands
    plate = max(plate.geoms, key=lambda g: g.area)

cuts = []
for ref, name, layer, x, y, r, blk in fps:
    if ref.startswith("SW"):
        sq = box(x - CUTOUT / 2, y - CUTOUT / 2, x + CUTOUT / 2, y + CUTOUT / 2)
        cuts.append((ref, rotate(sq, -r, origin=(x, y))))   # KiCad rotation is counter-clockwise on a y-down board

holes = []
for ref, name, layer, x, y, r, blk in fps:
    if ref.startswith("H"):
        ring = Point(x, y).buffer(HOLE_D / 2 + HOLE_RING)
        if plate.contains(ring) and all(not ring.intersects(c) for _, c in cuts):
            holes.append((ref, Point(x, y).buffer(HOLE_D / 2, quad_segs=16)))
        else:
            print(f"  {ref} ({x}, {y}): no {HOLE_RING} mm ring on the plate, left off; its screw clamps the PCB alone")

for ref, c in cuts:
    web = plate.exterior.distance(c) if plate.contains(c) else -1
    if web < 1.0:
        print(f"  {ref}: cutout {'outside the plate' if web < 0 else f'{web:.2f} mm from the plate edge'}")
webs = sorted((a.distance(b), ra, rb) for i, (ra, a) in enumerate(cuts) for rb, b in cuts[i + 1:])
print(f"  narrowest web between cutouts: {webs[0][0]:.2f} mm ({webs[0][1]}-{webs[0][2]})")

doc = ezdxf.new()
doc.header["$INSUNITS"] = 4  # mm
msp = doc.modelspace()
flip = lambda ring: [(px, -py) for px, py in ring.coords]
msp.add_lwpolyline(flip(plate.exterior), close=True)
for _, g in cuts + holes:
    msp.add_lwpolyline(flip(g.exterior), close=True)
out_dir = os.path.join(pcb_dir, f"plate-{side}")
os.makedirs(out_dir, exist_ok=True)
out = os.path.join(out_dir, "plate.dxf")
doc.saveas(out)
b = plate.bounds
print(f"  controller zone x {zone.bounds[0]:.1f}..{zone.bounds[2]:.1f}, y {zone.bounds[1]:.1f}..{zone.bounds[3]:.1f}")
print(f"wrote {os.path.relpath(out)}: {len(cuts)} cutouts, holes {[h[0] for h in holes]}, "
      f"plate {b[2] - b[0]:.1f} x {b[3] - b[1]:.1f} mm")
