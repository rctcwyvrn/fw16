"""PCIe lane pairs on L3 (In2.Cu), decided 2026-11-05 (0012 "PCIe escape"): one centerline per pair, offset to P/N.
Corridor pairs are 0.15/0.10 (85 ohm estimate); inside the U14 and J1 courtyards they neck to 0.10/0.10.
usage: pcie_lane_routes.py OUT.json  -> segments for route_trace, plus a clearance check (min 0.1, 0.3 between
different pairs and to other nets outside the U14/J1 courtyards, 0.25 to holes)."""
import json, sys, math, board_copper as g
from shapely.geometry import LineString, Point
L = 'In2.Cu'

def narrow(x, y):
    return (y < -26.55 and abs(x) < 14.5) or y > -5.34

def split_at_courtyards(cl):
    """Add a vertex wherever the centerline crosses a courtyard edge, so width jogs happen on straight segments."""
    out = [cl[0]]
    for a, b in zip(cl, cl[1:]):
        for yb in (-26.55, -5.34):
            if (a[1] - yb) * (b[1] - yb) < 0:
                t = (yb - a[1]) / (b[1] - a[1])
                out.append((a[0] + t * (b[0] - a[0]), yb))
        out.append(b)
    return out

def offset_runs(cl):
    """Split the centerline into runs of one width class; return [(LineString, h, w)]."""
    cl = split_at_courtyards(cl)
    runs, cur, cls = [], [cl[0]], None
    for a, b in zip(cl, cl[1:]):
        c = narrow((a[0] + b[0]) / 2, (a[1] + b[1]) / 2)
        if cls is not None and c != cls:
            runs.append((cur, cls)); cur = [a]
        cur.append(b); cls = c
    runs.append((cur, cls))
    return [(LineString(p), 0.1 if c else 0.125, 0.1 if c else 0.15) for p, c in runs]

PAIRS = {
 # name: (N net, P net, centerline, N start pts, P start pts, N end pts, P end pts)
 'TX0': ('PEX_TX0_N', 'PEX_TX0_P',
         [(0.0, -26.3), (-5.0, -26.3), (-5.0, -8.0), (-1.4, -4.4), (-0.6, -3.6), (-0.6, 0.77), (1.2, 0.77)],
         [(0.5, -27.7), (0.5, -26.175)], [(0.0, -27.0), (0.0, -26.425)],
         [(1.85, 0.47)], [(1.85, 1.07)]),
 'RX0a': ('/FPGA/GT_TX0_N', '/FPGA/GT_TX0_P',
         [(2.39, -30.5), (2.39, -28.9), (1.5, -28.01), (1.5, -25.5), (-1.1, -22.9)],
         [(1.889, -31.161), (2.29, -30.76)], [(1.889, -32.161), (2.49, -31.56)],
         [(-1.8, -23.0)], [(-1.1, -22.6), (-1.25, -22.45), (-1.25, -22.0), (-1.8, -22.0)]),
 'RX0b': ('PEX_RX0_N', 'PEX_RX0_P',
         [(-3.875, -21.3), (-3.875, -7.875), (0.0, -4.0), (0.0, -1.03), (1.2, -1.03)],
         [(-4.0, -23.0), (-3.55, -22.55), (-3.55, -21.6)], [(-4.0, -22.0)],
         [(1.85, -1.33)], [(1.85, -0.73)]),
 'RX1a': ('/FPGA/GT_TX1_N', '/FPGA/GT_TX1_P',
         [(4.39, -30.5), (4.39, -28.9), (3.75, -28.26), (3.75, -25.54), (1.6, -23.39), (1.6, -20.2)],
         [(3.889, -31.161), (4.29, -30.76)], [(3.889, -32.161), (4.49, -31.56)],
         [(1.4, -19.5)], [(1.95, -19.95), (1.95, -18.95), (1.4, -18.5)]),
 'RX1b': ('PEX_RX1_N', 'PEX_RX1_P',
         [(-1.875, -17.8), (-1.875, -6.975), (0.6, -4.5), (0.6, -3.5)],
         [(-2.0, -19.5), (-1.55, -19.05), (-1.55, -18.1)], [(-2.0, -18.5)],
         [(1.1, -3.13), (1.6, -3.13)], [(0.5, -2.53), (1.85, -2.53)]),
 'TX1': ('PEX_TX1_N', 'PEX_TX1_P',
         [(2.75, -26.3), (2.75, -25.6), (0.0, -22.85), (0.0, -9.5), (3.2, -6.3), (3.2, -3.6), (2.5, -2.9), (2.5, 2.1)],
         [(2.5, -27.7), (2.5, -26.6)], [(3.0, -27.0), (3.0, -26.6)],
         [(1.85, 2.27)], [(2.6, 2.87), (1.85, 2.87)]),
 'TX2': ('PEX_TX2_N', 'PEX_TX2_P',
         [(4.75, -26.3), (4.75, -25.48), (2.6, -23.33), (2.6, -9.0), (3.95, -7.65), (3.95, -3.6), (3.15, -2.8),
          (3.15, 0.17), (4.5, 0.17)],
         [(4.5, -27.7), (4.5, -26.6)], [(5.0, -27.0), (5.0, -26.6)],
         [(5.0, 0.67), (5.0, 0.87), (5.5, 1.07)], [(4.95, 0.07), (5.5, 0.47)]),
 'RX2a': ('/FPGA/GT_TX2_N', '/FPGA/GT_TX2_P',
         [(6.39, -30.5), (6.39, -28.9), (5.75, -28.26), (5.75, -24.9), (6.3, -24.35), (6.3, -23.6)],
         [(5.889, -31.161), (6.29, -30.76)], [(5.889, -32.161), (6.49, -31.56)],
         [(6.2, -23.0)], [(6.75, -23.3), (6.75, -22.3), (6.2, -22.0)]),
 'RX2b': ('PEX_RX2_N', 'PEX_RX2_P',
         [(4.125, -21.5), (4.125, -10.0), (4.7, -9.425), (4.7, -3.6), (3.8, -2.7), (3.8, -1.63), (4.5, -1.63)],
         [(4.0, -23.0), (4.45, -22.55), (4.45, -21.7)], [(4.0, -22.0)],
         [(4.95, -1.73), (5.5, -1.33)], [(4.6, -1.53), (5.0, -1.13), (5.0, -0.93), (5.5, -0.73)]),
 'RX3a': ('/FPGA/GT_TX3_N', '/FPGA/GT_TX3_P',
         [(8.39, -30.5), (8.39, -26.0), (9.7, -24.69), (9.7, -20.2)],
         [(7.889, -31.161), (8.29, -30.76)], [(7.889, -32.161), (8.49, -31.56)],
         [(9.6, -19.5)], [(10.15, -19.95), (10.15, -18.95), (9.6, -18.5)]),
 'RX3b': ('PEX_RX3_N', 'PEX_RX3_P',
         [(6.125, -17.8), (6.125, -7.0), (5.5, -6.375), (5.5, -3.7)],
         [(6.0, -19.5), (6.45, -19.05), (6.45, -18.1)], [(6.0, -18.5)],
         [(5.6, -3.4), (5.5, -3.13)], [(5.025, -3.25), (5.025, -2.73), (5.5, -2.53)]),
 'TX3': ('PEX_TX3_N', 'PEX_TX3_P',
         [(6.75, -26.1), (6.75, -25.9), (7.5, -25.15), (7.5, -7.0), (6.3, -5.8), (6.3, 2.1)],
         [(6.5, -27.7), (6.5, -26.35)], [(7.0, -27.0), (7.0, -26.35)],
         [(5.5, 2.27)], [(6.4, 2.87), (5.5, 2.87)]),
}

def build(name, spec):
    nN, nP, cl, ns, ps, ne, pe = spec
    sides = {+1: [], -1: []}
    for line, h, w in offset_runs(cl):
        for s in (+1, -1):
            sides[s].append((list(line.offset_curve(s * h, join_style='mitre', mitre_limit=3).coords), w))
    def flat(s):
        pts, ws = [], []
        for coords, w in sides[s]:
            if pts:
                ws.append(min(w, ws[-1]))  # jog between runs at the narrower width
            pts += coords; ws += [w] * (len(coords) - 1)
        return pts, ws
    a, b = flat(+1), flat(-1)
    d = lambda p, q: math.dist(p, q)
    if d(ns[-1], a[0][0]) + d(ne[0], a[0][-1]) > d(ns[-1], b[0][0]) + d(ne[0], b[0][-1]):
        a, b = b, a
    for nm, (pts, ws), st, en in ((nN, a, ns, ne), (nP, b, ps, pe)):
        if d(st[-1], pts[0]) > 1.2 or d(en[0], pts[-1]) > 1.2:
            print(f'{name} {nm}: endpoint far from offset ({d(st[-1], pts[0]):.2f}, {d(en[0], pts[-1]):.2f}) - polarity?')
        full = st + pts + en
        wl = [0.1] * (len(st)) + ws + [0.1] * len(en)
        for (p, q), w in zip(zip(full, full[1:]), wl):
            if d(p, q) > 1e-4:
                yield dict(x1=round(p[0], 4), y1=round(p[1], 4), x2=round(q[0], 4), y2=round(q[1], 4), w=w, layer=L, net=nm, pair=name)

segs = [s for n, sp in PAIRS.items() for s in build(n, sp)]
json.dump(dict(vias=[], segments=segs), open(sys.argv[1], 'w'))

# clearance check on L3
from shapely.geometry import box
obst = []
for v in g.vias(): obst.append((Point(v['x'], v['y']).buffer(v['size'] / 2), v['net'], 'via', None))
for p in g.pads():
    if '*.Cu' in p['layers'] or L in p['layers']:
        r = max(p['sx'], p['sy']) / 2
        obst.append((Point(p['x'], p['y']).buffer(r), p['net'], 'hole' if p['kind'] == 'np_thru_hole' else 'pad', None))
for s in g.segments():
    if s['layer'] == L: obst.append((LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2), s['net'], 'seg', None))
new = [(LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2, cap_style='round'), s['net'], 'new', s['pair'], s) for s in segs]
partner = {sp[0]: sp[1] for sp in PAIRS.values()}; partner.update({v: k for k, v in partner.items()})
bad = 0
for i, (ga, na, _, pa, sa) in enumerate(new):
    mid = ((sa['x1'] + sa['x2']) / 2, (sa['y1'] + sa['y2']) / 2)
    exempt = narrow(*mid)
    for gb, nb, kb, pb, *_ in [o + (None,) for o in obst] + new[i + 1:]:
        if nb == na: continue
        need = 0.25 if kb == 'hole' else (0.1 if exempt or partner.get(na) == nb else 0.3)
        dd = ga.distance(gb)
        if dd < need - 1e-3:
            bad += 1
            if bad <= 40: print(f'{pa} {na} vs {kb} {nb}: {dd:.3f} < {need} at {tuple(round(c, 2) for c in mid)}')
print(len(segs), 'segments; violations', bad)
