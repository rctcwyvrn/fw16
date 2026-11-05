"""Bench reference clock (0012): Y2 (BENCH_CLK_P pin 4, _N pin 5) -> F.Cu to the AC caps C89 (P) / C90 (N), which
face Y2 (rotation 270) -> vias under the caps -> B.Cu north to U14 GT_REFCLK1 (T7 P, T6 N), microstrip over the L7
GND plane. On L3 the via-free channel (y -33.66) east of x 4.9 belongs to GT_REFCLK0 and the north-east dogbone
sites collide with existing vias, so this pair takes the same channel on B.Cu (it costs one strip of B.Cu under the
BGA for decoupling caps). 0.10/0.10 (0.09 between via rows).
usage: bench_clk_route.py OUT.json -> vias + segments, clearance check and P/N lengths."""
import json, sys, math, board_copper as g
from shapely.geometry import LineString, Point, box
L3, F = 'In2.Cu', 'F.Cu'
vias, segs = [], []
def via(x, y, n): vias.append(dict(x=x, y=y, size=0.45, drill=0.2, net=n))
def path(pts, n, layer, w=0.1):
    for p, q in zip(pts, pts[1:]): segs.append(dict(x1=p[0], y1=p[1], x2=q[0], y2=q[1], w=w, layer=layer, net=n))
GN, GP, BN, BP = '/FPGA/GT_REFCLK1_N', '/FPGA/GT_REFCLK1_P', '/FPGA/BENCH_CLK_N', '/FPGA/BENCH_CLK_P'
# Y2 -> caps on F.Cu (P leaves west between Y2 pins 3 and 4; N goes round the east side)
path([(16.8, -20.875), (16.8, -19.9), (15.5, -18.9), (15.5, -18.48)], BP, F)
path([(16.8, -22.0), (17.6, -22.0), (17.6, -19.2), (16.7, -18.5), (16.7, -18.48)], BN, F)
# caps -> vias (GT pads at y -17.52)
via(15.5, -16.7, GP); path([(15.5, -17.52), (15.5, -16.7)], GP, F)
via(16.7, -16.7, GN); path([(16.7, -17.52), (16.7, -16.7)], GN, F)
# U14 dogbones (north-east sites)
via(2.889, -33.161, GN); path([(2.5, -33.55), (2.889, -33.161)], GN, F)
via(2.889, -34.161, GP); path([(2.5, -34.55), (2.889, -34.161)], GP, F)
B = 'B.Cu'
# B.Cu: east along the -33.66 channel, south outside the BGA (x 14.3 N / 14.5 P), to the cap vias
path([(2.889, -33.161), (3.29, -33.56), (14.3, -33.56)], GN, B, 0.09)
path([(2.889, -34.161), (3.29, -33.76), (14.5, -33.76)], GP, B, 0.09)
path([(14.3, -33.56), (14.3, -16.2), (16.7, -16.2), (16.7, -16.7)], GN, B)
# P is 3.73 mm short end to end (2.0 here, 1.73 on the short F.Cu run from Y2), all made up here: 4 bumps east
pts, a = [(14.5, -33.76), (14.5, -27.0)], -27.0
for k in range(4):
    b = a + 0.3 + k * 0.75
    pts += [(14.5, b), (14.5 + 3.725 / 8, b), (14.5 + 3.725 / 8, b + 0.3), (14.5, b + 0.3)]
path(pts + [(14.5, -16.7), (15.5, -16.7)], GP, B)
json.dump(dict(vias=vias, segments=segs), open(sys.argv[1], 'w'))

ALL = {'F.Cu', 'In1.Cu', 'In2.Cu', 'In3.Cu', 'In4.Cu', 'In5.Cu', 'In6.Cu', 'B.Cu'}
items = []
for p in g.pads():
    lay = ALL if '*.Cu' in p['layers'] else {l for l in p['layers'] if l.endswith('.Cu')}
    w, h = (p['sx'], p['sy']) if p['rot'] % 180 == 0 else (p['sy'], p['sx'])
    geo = Point(p['x'], p['y']).buffer(w / 2) if p['shape'] == 'circle' else box(p['x'] - w / 2, p['y'] - h / 2, p['x'] + w / 2, p['y'] + h / 2)
    items.append((geo, p['net'], lay, 'npth' if p['kind'] == 'np_thru_hole' else 'pad', None))
for v in g.vias(): items.append((Point(v['x'], v['y']).buffer(v['size'] / 2), v['net'], ALL, 'via', (v['x'], v['y'])))
for s in g.segments(): items.append((LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2), s['net'], {s['layer']}, 'seg', None))
new = [(Point(v['x'], v['y']).buffer(0.225), v['net'], ALL, 'via', (v['x'], v['y'])) for v in vias] + \
      [(LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2), s['net'], {s['layer']}, 'seg', None) for s in segs]
pcie = lambda n: n.startswith(('PEX_', '/FPGA/GT_'))
bad = 0
for i, (ga, na, la, ka, ca) in enumerate(new):
    for gb, nb, lb, kb, cb in items + new[i + 1:]:
        if na == nb or not (la & lb): continue
        mid = ga.centroid.coords[0]
        in_cy = (mid[1] < -26.55 and abs(mid[0]) < 14.5) or mid[1] > -5.34
        need = 0.25 if 'npth' in (ka, kb) else (0.3 if (pcie(na) or pcie(nb)) and not in_cy else 0.1)
        if (na, nb) in ((GN, GP), (GP, GN)): need = 0.1
        d = ga.distance(gb)
        for c, other in ((ca, gb), (cb, ga)):
            if c and other.distance(Point(*c).buffer(0.1)) < 0.25 - 1e-3: bad += 1; print('drill', na, nb)
        if d < need - 1e-3:
            bad += 1; print(f'{na} vs {nb}: {d:.3f} < {need} near {tuple(round(v, 2) for v in ga.centroid.coords[0])}')
print(len(vias), 'vias', len(segs), 'segments; violations', bad)
L = {}
for s in segs: L[s['net']] = L.get(s['net'], 0) + math.dist((s['x1'], s['y1']), (s['x2'], s['y2']))
print({k: round(v, 3) for k, v in L.items()})
print('end-to-end P-N', round(L[GP] + L[BP] - L[GN] - L[BN], 4))
