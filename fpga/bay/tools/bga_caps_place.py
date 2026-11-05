"""B.Cu decoupling under U14 (0012; 0201 parts, user 2026-11-05). KiCad's IPC rotates rounded-rect footprints only
in 90-degree steps, so every part sits square to the 1.0 mm via grid at a cell centre (x.389, y.661) or row/column
midpoint, with each pad tied to an adjacent same-net via by a short 0.1 mm B.Cu stub. A site is valid only if both
pads and both stubs clear every other-net via (0.1 copper, 0.25 to drills) and every already-placed part/stub.
usage: bga_caps_place.py OUT.json -> {"parts": {ref: [x, y, rot]}, "segments": [...]}"""
import json, sys, math, board_copper as g
from shapely.geometry import Point, LineString, box
V = [(round(v['x'], 3), round(v['y'], 3), v['net']) for v in g.vias()]
HALF, PL, PW = 0.32, 0.46, 0.40
DIRS = {0: (-1, 0), 180: (1, 0), 90: (0, 1), 270: (0, -1)}   # B.Cu: pad 1 direction for each KiCad rotation

def pad_boxes(cx, cy, r):
    d = DIRS[r]; w, h = (PL, PW) if d[1] == 0 else (PW, PL)
    c1 = (cx + HALF * d[0], cy + HALF * d[1]); c2 = (cx - HALF * d[0], cy - HALF * d[1])
    return (box(c1[0]-w/2, c1[1]-h/2, c1[0]+w/2, c1[1]+h/2), c1), (box(c2[0]-w/2, c2[1]-h/2, c2[0]+w/2, c2[1]+h/2), c2)

taken = []   # (geometry, net)
bodies = []  # courtyards of placed parts (0201: 1.4 x 0.7)
def clear(geo, net):
    for x, y, n in V:
        if n == net: continue
        if geo.distance(Point(x, y).buffer(0.225)) < 0.1 - 1e-6 or geo.distance(Point(x, y).buffer(0.1)) < 0.25 - 1e-6: return False
    return all(tn == net or geo.distance(tg) >= 0.1 - 1e-6 for tg, tn in taken)

def try_site(cx, cy, r, n1, n2, maxstub=0.8):
    w, h = (1.4, 0.7) if r in (0, 180) else (0.7, 1.4)
    if any(box(cx - w/2, cy - h/2, cx + w/2, cy + h/2).intersects(b) for b in bodies): return None
    out = []
    for (pad, c), net in zip(pad_boxes(cx, cy, r), (n1, n2)):
        if not clear(pad, net): return None
        best = None
        for x, y, n in V:
            if n != net: continue
            dist = math.dist(c, (x, y))
            if dist > maxstub: continue
            stub = LineString([c, (x, y)]).buffer(0.05)
            if clear(stub, net) and (best is None or dist < best[0]): best = (dist, (x, y), stub)
        if best is None: return None
        out.append((pad, net, c, best[1], best[2]))
    return out

parts, segs = {}, []
def commit(ref, cx, cy, r, site):
    parts[ref] = [round(cx, 4), round(cy, 4), r]
    w, h = (1.4, 0.7) if r in (0, 180) else (0.7, 1.4)
    bodies.append(box(cx - w/2, cy - h/2, cx + w/2, cy + h/2))
    for pad, net, c, via, stub in site:
        taken.extend([(pad, net), (stub, net)])
        segs.append(dict(x1=round(c[0], 4), y1=round(c[1], 4), x2=via[0], y2=via[1], w=0.1, layer='B.Cu', net=net))

def sites(xr, yr, step=0.5):
    for i in range(int((xr[1] - xr[0]) / step) + 1):
        for j in range(int((yr[1] - yr[0]) / step) + 1):
            yield round(xr[0] + i * step + 0.389, 3), round(yr[0] + j * step + 0.661, 3)

def place_spread(refs, n1, n2, xr, yr, centre, minsep, maxstub=0.8):
    chosen = []
    cand = sorted(sites(xr, yr), key=lambda p: math.dist(p, centre))
    for ref in refs:
        best = None
        for cx, cy in cand:
            sep = min([math.dist((cx, cy), m) for m in chosen] or [99])
            if sep < minsep: continue
            for r in DIRS:
                s = try_site(cx, cy, r, n1, n2, maxstub)
                if s and (best is None or sep > best[0] + 0.5 or (abs(sep - best[0]) <= 0.5 and math.dist((cx, cy), centre) < best[4])):
                    best = (sep, cx, cy, r, math.dist((cx, cy), centre), s)
                    break
        if best is None: print('no site for', ref); continue
        sep, cx, cy, r, _, s = best
        commit(ref, cx, cy, r, s); chosen.append((cx, cy))

# VCCINT field x -3.5..4.5, y -44.5..-36.5
place_spread(['C52', 'C53', 'C54', 'C55', 'C56', 'C57', 'C58', 'C59'], '+0V85_VCCINT', 'GND', (-5, 5), (-46, -35), (0.5, -40.5), 2.0)
place_spread(['FB2'], '/FPGA/GNDADC', 'GND', (-2, 3), (-43, -38), (0.5, -41), 0)
place_spread(['C75'], '/FPGA/VCCADC', '/FPGA/GNDADC', (-3, 2), (-44, -39), (-0.889, -41.5), 0)
place_spread(['C76'], '/FPGA/VCCADC', '/FPGA/GNDADC', (-4, 3), (-45, -38), (-0.889, -41.5), 0, 3.0)
place_spread(['FB1'], '+1V8', '/FPGA/VCCADC', (-5, 3), (-48, -37), (-0.889, -42), 0, 4.0)
print(json.dumps(parts)); print(len(segs), 'stubs, longest', round(max(math.dist((s['x1'], s['y1']), (s['x2'], s['y2'])) for s in segs), 3))
json.dump(dict(parts=parts, segments=segs), open(sys.argv[1], 'w'))
