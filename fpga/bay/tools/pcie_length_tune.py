"""Intra-pair length tuning for the PCIe lanes (0012): adds square bumps to the shorter leg of each pair section
(TX lanes end to end; each RX lane on each side of its B.Cu cap hop), on that leg's longest straight corridor run on
L3, bumping away from the partner and starting at the end nearest the U14/cap side where the mismatch comes from.
usage: pcie_length_tune.py OUT.json -> {"delete": [uuid], "segments": [...]}, plus a clearance check."""
import json, sys, math, board_copper as g
from shapely.geometry import LineString, Point
L3, S, GAP, HMAX = 'In2.Cu', 0.3, 0.45, 0.5
segs, vias, pads = g.segments(), g.vias(), g.pads()

def length(net):
    return sum(math.dist((s['x1'], s['y1']), (s['x2'], s['y2'])) for s in segs if s['net'] == net)

sections = [(f'PEX_TX{i}_N', f'PEX_TX{i}_P') for i in range(4)] + \
           [(f'/FPGA/GT_TX{i}_N', f'/FPGA/GT_TX{i}_P') for i in range(4)] + \
           [(f'PEX_RX{i}_N', f'PEX_RX{i}_P') for i in range(4)]

obst = [(Point(v['x'], v['y']).buffer(v['size'] / 2), v['net']) for v in vias]
obst += [(Point(p['x'], p['y']).buffer(max(p['sx'], p['sy']) / 2), p['net']) for p in pads if '*.Cu' in p['layers']]

def clear(geom, net, partner, skip):
    worst = 9
    for s in segs:
        if s['layer'] != L3 or s['net'] in (net,) or s['uuid'] == skip: continue
        need = 0.1 if s['net'] == partner else 0.3
        d = geom.distance(LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2)) - need
        worst = min(worst, d)
    for o, n in obst:
        if n == net: continue
        worst = min(worst, geom.distance(o) - 0.3)
    return worst

out = {'delete': [], 'segments': []}
for nN, nP in sections:
    d = length(nP) - length(nN)
    tgt, partner = (nN, nP) if d > 0 else (nP, nN)
    need = abs(d)
    if need < 0.05:
        print(f'{tgt}: matched ({d:+.3f})'); continue
    cands = sorted([s for s in segs if s['net'] == tgt and s['layer'] == L3 and abs(s['x1'] - s['x2']) < 1e-6
                    and s['w'] >= 0.15], key=lambda s: -abs(s['y2'] - s['y1']))
    done = False
    for s in cands:
        x = s['x1']; y0, y1 = sorted((s['y1'], s['y2']))
        ps = [p for p in segs if p['net'] == partner and p['layer'] == L3 and abs(p['x1'] - p['x2']) < 1e-6
              and min(p['y1'], p['y2']) < y1 and max(p['y1'], p['y2']) > y0]
        if not ps: continue
        out_dir = 1 if x > ps[0]['x1'] else -1
        for n in range(1, 8):
            h = need / (2 * n)
            if h > HMAX: continue
            span = n * S + (n - 1) * GAP
            if span + 0.6 > y1 - y0: break
            yb = y0 + 0.3
            pts = [(x, y0)]
            for k in range(n):
                a = yb + k * (S + GAP)
                pts += [(x, a), (x + out_dir * h, a), (x + out_dir * h, a + S), (x, a + S)]
            pts.append((x, y1))
            geom = LineString(pts).buffer(s['w'] / 2)
            m = clear(geom, tgt, partner, s['uuid'])
            if m >= -0.005:
                out['delete'].append(s['uuid'])
                for (p, q) in zip(pts, pts[1:]):
                    if math.dist(p, q) > 1e-4:
                        out['segments'].append(dict(x1=round(p[0], 4), y1=round(p[1], 4), x2=round(q[0], 4),
                                                    y2=round(q[1], 4), w=s['w'], layer=L3, net=tgt))
                print(f'{tgt}: +{need:.3f} mm as {n} bump(s) of {h:.3f} at x {x:.3f}, y {yb:.2f}..{yb + span:.2f} (margin {m:.3f})')
                done = True; break
        if done: break
    if not done: print(f'{tgt}: NO ROOM for +{need:.3f}')
json.dump(out, open(sys.argv[1], 'w'))
