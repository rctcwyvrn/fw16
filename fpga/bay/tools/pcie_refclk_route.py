"""PEX_REFCLK route (0012): U14 GT_REFCLK0 balls (V6 N, V7 P) -> dogbone vias -> L3 east along the via-free channel
between via rows y -33.161 / -34.161 -> down outside the BGA at x 13.6/13.8 -> via hop up to the F.Cu AC caps C85 (P)
and C86 (N) -> back to L3 -> J1 column 12.85 through stub vias at x 12.0 (entered from the right; P is the upper pad),
plus GND vias for that column's GND pads S49/S52. 0.10/0.10 throughout (HCSL, ~100 ohm).
usage: pcie_refclk_route.py OUT.json -> vias + segments, and a clearance check."""
import json, sys, math, board_copper as g
from shapely.geometry import LineString, Point
W, L3, F = 0.1, 'In2.Cu', 'F.Cu'
vias, segs = [], []
def via(x, y, n): vias.append(dict(x=x, y=y, size=0.45, drill=0.2, net=n))
def path(pts, n, layer, w=W):
    for p, q in zip(pts, pts[1:]): segs.append(dict(x1=p[0], y1=p[1], x2=q[0], y2=q[1], w=w, layer=layer, net=n))
GN, GP, PN, PP = '/FPGA/GT_REFCLK0_N', '/FPGA/GT_REFCLK0_P', 'PEX_REFCLK_N', 'PEX_REFCLK_P'
# U14 dogbones
via(4.889, -33.161, GN); path([(4.5, -33.55), (4.889, -33.161)], GN, F)
via(4.889, -34.161, GP); path([(4.5, -34.55), (4.889, -34.161)], GP, F)
# L3 escape east (0.09 between the via rows: a 0.10 track would sit 0.249 from the drills), then south
path([(4.889, -33.161), (5.29, -33.56), (13.6, -33.56)], GN, L3, 0.09)
path([(4.889, -34.161), (5.29, -33.76), (13.8, -33.76)], GP, L3, 0.09)
# turning south puts N on the west; P wraps under N's cap via (L3 is free below the F.Cu caps) to reach the west cap
def bumps(x, y0, y1, n, h, d):
    # straight run x from y0 to y1 with n square bumps of height h toward d (+1 east / -1 west), 0.3 wide, 0.45 apart
    pts, a = [(x, y0)], y0 + 0.3
    for k in range(n):
        b = a + k * 0.75
        pts += [(x, b), (x + d * h, b), (x + d * h, b + 0.3), (x, b + 0.3)]
    return pts + [(x, y1)]
# intra-pair match: N is 3.55 mm short on the chip side, P 1.05 mm short on the J1 side (bumps away from the partner)
path([(5.29, -33.56)][:0] + [(13.6, -33.56)] + bumps(13.6, -27.0, -13.7, 4, 3.554 / 8, -1)[0:] + [(13.4, -13.3)], GN, L3)
path([(13.8, -33.76), (13.8, -12.5), (12.2, -12.5), (12.2, -13.3)], GP, L3)
# hop to the F.Cu caps (GT pads at y -12.48, PEX pads at -11.52)
via(13.4, -13.3, GN); path([(13.4, -13.3), (13.4, -12.48)], GN, F)
via(12.2, -13.3, GP); path([(12.2, -13.3), (12.2, -12.48)], GP, F)
via(13.4, -10.7, PN); path([(13.4, -11.52), (13.4, -10.7)], PN, F)
via(12.2, -10.7, PP); path([(12.2, -11.52), (12.2, -10.7)], PP, F)
# L3 down to J1, entering the x 12.0 stub vias from the right
path([(13.4, -10.7), (12.6, -9.9), (12.6, -1.33), (12.0, -1.33)], PN, L3)
path([(12.2, -10.7), (12.4, -10.5)] + bumps(12.4, -10.5, -1.93, 2, 1.048 / 4, -1)[1:] + [(12.0, -1.93)], PP, L3)
via(12.0, -1.33, PN); path([(12.0, -1.33), (12.85, -1.33)], PN, F)
via(12.0, -1.93, PP); path([(12.0, -1.93), (12.85, -1.93)], PP, F)
for y in (-2.53, -0.73):
    via(12.0, y, 'GND'); path([(12.0, y), (12.85, y)], 'GND', F, 0.2)
json.dump(dict(vias=vias, segments=segs), open(sys.argv[1], 'w'))

# clearance check: 0.1 copper, 0.25 copper-to-drill
ALL = {'F.Cu', 'In1.Cu', 'In2.Cu', 'In3.Cu', 'In4.Cu', 'In5.Cu', 'In6.Cu', 'B.Cu'}
items = []
for p in g.pads():
    lay = ALL if '*.Cu' in p['layers'] else {l for l in p['layers'] if l.endswith('.Cu')}
    items.append((Point(p['x'], p['y']).buffer(max(p['sx'], p['sy']) / 2) if p['shape'] == 'circle' else
                  Point(p['x'], p['y']).buffer(0).buffer(0) if False else
                  __import__('shapely.geometry', fromlist=['box']).box(p['x'] - (p['sx'] if p['rot'] % 180 == 0 else p['sy']) / 2,
                      p['y'] - (p['sy'] if p['rot'] % 180 == 0 else p['sx']) / 2, p['x'] + (p['sx'] if p['rot'] % 180 == 0 else p['sy']) / 2,
                      p['y'] + (p['sy'] if p['rot'] % 180 == 0 else p['sx']) / 2), p['net'], lay, 'npth' if p['kind'] == 'np_thru_hole' else 'pad'))
for v in g.vias(): items.append((Point(v['x'], v['y']).buffer(v['size'] / 2), v['net'], ALL, ('drill', v['x'], v['y'])))
for s in g.segments(): items.append((LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2), s['net'], {s['layer']}, 'seg'))
new = [(Point(v['x'], v['y']).buffer(0.225), v['net'], ALL, ('drill', v['x'], v['y'])) for v in vias] + \
      [(LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2), s['net'], {s['layer']}, 'seg') for s in segs]
bad = 0
for i, (ga, na, la, ka) in enumerate(new):
    for gb, nb, lb, kb in items + new[i + 1:]:
        if na == nb or not (la & lb): continue
        need = 0.25 if 'npth' in (ka, kb) else 0.1
        d = ga.distance(gb)
        for k, other in ((ka, gb), (kb, ga)):
            if isinstance(k, tuple) and k[0] == 'drill':
                d2 = other.distance(Point(k[1], k[2]).buffer(0.1))
                if d2 < 0.25 - 1e-3: bad += 1; print(f'drill {na} vs {nb}: {d2:.3f}')
        if d < need - 1e-3: bad += 1; print(f'{na} vs {nb}: {d:.3f} < {need} near {tuple(round(c, 2) for c in ga.centroid.coords[0])}')
print(len(vias), 'vias', len(segs), 'segments; violations', bad)
L = {}
for s in segs: L[s['net']] = L.get(s['net'], 0) + math.dist((s['x1'], s['y1']), (s['x2'], s['y2']))
print({k: round(v, 3) for k, v in L.items()})
