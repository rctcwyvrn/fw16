"""L5/L6 power planes (0012; user 2026-11-06: rails split across L5/L6, GND fills the rest).
L5 In4.Cu: VCCINT (left feed + BGA core), MGTAVCC (BGA front row east to U12, front-left to C72), +1V8 (BGA south,
east caps, east corridor north to U8). L6 In5.Cu: MGTAVTT (BGA rows -35..-32 east to U13, front to C73), +5V0_MAIN
(left regulator cluster, front band, right regulator cluster). Both layers: a priority-0 GND zone over the board.

1. via sites: every listed source pad gets N vias (and its 2-pad partner's GND pad the same), each beside the pad with
   a 0.25 mm F.Cu stub, clear of other-net copper on every layer (0.1; PCIe 0.3), holes (0.25), hole-to-hole (0.2),
   board edge (0.5).
2. regions: on each layer, a Voronoi split over that layer's power-net vias (existing + new), each net clipped to its
   CORRIDOR, shrunk by GAP/2 so islands sit GAP apart.
3. check: each region minus other-net antipads (via r + ZCLR, TH holes) opened by MIN_T; every own-net via must sit
   in the component holding the net's source vias.
usage: power_plane_plan.py OUT.json [svgdir] [--v18 L5|L6]"""
import json, math, sys, os, collections
import board_copper as g
from shapely.geometry import Point, LineString, Polygon, MultiPoint, box
from shapely.affinity import rotate
from shapely.ops import unary_union, voronoi_diagram

VIA_R, DRILL_R, CLR, PCIE_CLR, HOLE_CLR, H2H, EDGE = 0.225, 0.1, 0.1, 0.3, 0.25, 0.2, 0.5
ZCLR, GAP, MIN_T, STUB_W, JW = 0.1, 0.3, 0.25, 0.25, 0.15
BGA = box(-14.5, -55.55, 14.5, -26.55)
V18 = sys.argv[sys.argv.index('--v18') + 1] if '--v18' in sys.argv else 'L5'
# per layer, in precedence order outside the BGA: (net, corridor); FORCE overrides the Voronoi split inside the BGA
V18_EAST = unary_union([box(14.5, -55, 24.5, -33.0)])
MGTAVTT = unary_union([box(-14.5, -38, 14.5, -26.55), box(-10, -27, -6, -22.6), box(14.5, -34.5, 35.5, -26.2),
                       box(29.3, -26.3, 35.5, -23.0)]).difference(box(20.8, -28.6, 23.6, -25))
FIVE = unary_union([box(-44, -38, -20, -17), box(-44, -22.0, 44, -17.2), box(20, -25.9, 44, -4),
                    box(20.8, -28.6, 23.6, -25)]).difference(box(29.3, -26.6, 36, -22.7))
LAYERS = {
    'In4.Cu': [('+0V85_VCCINT', unary_union([BGA, box(-37.5, -47, -14.5, -25.5)])),
               ('+0V9_MGTAVCC', unary_union([BGA, box(14.5, -32.6, 30, -17), box(-11.8, -27, -4, -22.4)]))]
              + ([('+1V8', unary_union([BGA, V18_EAST, box(24.5, -55, 37, -33.0), box(30, -45, 37, -5), box(21.5, -11, 37, -5)]))]
                 if V18 == 'L5' else []),
    'In5.Cu': [('+1V2_MGTAVTT', MGTAVTT), ('+5V0_MAIN', FIVE)]
              + ([('+1V8', unary_union([box(-14.5, -55.55, 14.5, -35.7), V18_EAST]))] if V18 == 'L6' else []),
}
FORCE = {('In4.Cu', '+0V85_VCCINT'): box(-14.5, -41.45, -3.4, -37.65)} if V18 == 'L5' else {}
# source pads: (ref, pad, vias); a 2-pad cap's other (GND) pad gets the same count
SOURCES = {
    '+0V85_VCCINT': [('L2', '2', 4)] + [(c, '1', 2) for c in ('C24', 'C25', 'C26', 'C27', 'C46', 'C47', 'C48', 'C49')]
                    + [(c, '1', 1) for c in ('C50', 'C51')],
    '+1V8': [('L3', '2', 2), ('C29', '1', 2)] + [(c, '1', 1) for c in ('C60', 'C61', 'C62', 'C63', 'C64', 'C65',
                                                                        'C66', 'C67', 'C68', 'C69')],
    '+0V9_MGTAVCC': [('U12', '1', 2), ('C39', '1', 2), ('C72', '1', 1)],   # C40 ties to C39 on F.Cu
    '+1V2_MGTAVTT': [('U13', '1', 2), ('C44', '1', 2), ('C45', '1', 1), ('C73', '1', 1), ('R48', '1', 1)],
    '+5V0_MAIN': [('L1', '2', 3), ('C14', '1', 2), ('C15', '1', 2), ('C16', '1', 1), ('C19', '1', 1), ('C20', '1', 1),
                  ('C28', '1', 1), ('U7', '5', 1), ('C42', '1', 1), ('C37', '1', 1), ('U12', '12', 1),
                  ('U13', '12', 1), ('U8', '3', 1), ('C30', '1', 1), ('U9', '3', 1), ('C32', '1', 1),
                  ('U11', '1', 1), ('U10', '1', 1), ('C34', '1', 1), ('U6', '1', 1)],
}
PCIE = ('PEX_TX', 'PEX_RX', 'PEX_REFCLK', '/FPGA/GT_TX', '/FPGA/GT_REFCLK')

pads, vias, segs = g.pads(), g.vias(), g.segments()
edge = unary_union([Polygon(l) for l in g.outline()])
def pad_geo(p):
    if p['shape'] == 'circle': return Point(p['x'], p['y']).buffer(p['sx'] / 2)
    return rotate(box(p['x'] - p['sx'] / 2, p['y'] - p['sy'] / 2, p['x'] + p['sx'] / 2, p['y'] + p['sy'] / 2),
                  -p['rot'], origin=(p['x'], p['y']))
def copper_layers(p):
    return ['F.Cu', 'B.Cu', 'In1.Cu', 'In2.Cu', 'In3.Cu', 'In4.Cu', 'In5.Cu', 'In6.Cu'] if '*.Cu' in p['layers'] \
        else [l for l in p['layers'] if l.endswith('.Cu')]
# obstacles: (geometry, net, layers or None for all, is_hole)
obst = [(pad_geo(p), p['net'], copper_layers(p), False) for p in pads]
obst += [(Point(p['x'], p['y']).buffer(p['drill'] / 2), p['net'], None, True) for p in pads if p['drill'] > 0]
obst += [(Point(v['x'], v['y']).buffer(v['size'] / 2), v['net'], None, False) for v in vias]
obst += [(Point(v['x'], v['y']).buffer(v['drill'] / 2), v['net'], None, True) for v in vias]
obst += [(LineString([(s['x1'], s['y1']), (s['x2'], s['y2'])]).buffer(s['w'] / 2), s['net'], [s['layer']], False)
         for s in segs]

def via_ok(x, y, net):
    c = Point(x, y)
    if edge.exterior.distance(c) < EDGE + VIA_R or not edge.contains(c): return False
    for geo, n, lay, hole in obst:
        d = geo.distance(c)
        if hole and d < DRILL_R + H2H: return False
        if n == net: continue
        need = max(VIA_R + (PCIE_CLR if n.startswith(PCIE) else CLR), DRILL_R + HOLE_CLR) if not hole else DRILL_R + HOLE_CLR
        if d < need: return False
    return True
def stub_ok(line, net, layer):
    geo = line.buffer(STUB_W / 2)
    return all(n == net or geo.distance(gg) >= CLR for gg, n, lay, hole in obst if hole or lay is None or layer in lay)

new_vias, stubs, missing = [], [], []
PLANE_AREA = {n: c.difference(BGA).buffer(-0.3) for l in LAYERS.values() for n, c in l}
def place_vias(p, count):
    pg = pad_geo(p); placed = 0
    area = PLANE_AREA.get(p['net'])
    t = math.radians(-p['rot'])
    cands = []
    for k in range(48):
        a = 2 * math.pi * k / 48
        for extra in (0.35, 0.55, 0.8, 1.1):
            ray = LineString([(p['x'], p['y']), (p['x'] + 10 * math.cos(a), p['y'] + 10 * math.sin(a))])
            hit = ray.intersection(pg.exterior)
            e = hit if hit.geom_type == 'Point' else list(hit.geoms)[0]
            cands.append((extra, k, e.x + (VIA_R + extra - 0.2) * math.cos(a), e.y + (VIA_R + extra - 0.2) * math.sin(a), e))
    cands.sort(key=lambda c: c[0])
    layer = p['layers'][0]
    for extra, k, x, y, e in cands:
        if placed == count: break
        if area is not None and not area.contains(Point(x, y)): continue
        if not via_ok(x, y, p['net']): continue
        line = LineString([(e.x, e.y), (x, y)])
        if not stub_ok(line, p['net'], layer): continue
        new_vias.append(dict(x=round(x, 3), y=round(y, 3), size=0.45, drill=0.2, net=p['net']))
        stubs.append(dict(x1=round(p['x'], 3), y1=round(p['y'], 3), x2=round(x, 3), y2=round(y, 3), w=STUB_W, layer=layer, net=p['net']))
        obst.append((Point(x, y).buffer(VIA_R), p['net'], None, False))
        obst.append((Point(x, y).buffer(DRILL_R), p['net'], None, True))
        obst.append((LineString([(p['x'], p['y']), (x, y)]).buffer(STUB_W / 2), p['net'], [layer], False))
        placed += 1
    if placed < count: missing.append((p['ref'], p['pad'], p['net'], placed, count))

# outer-row power balls the 2026-11-01 fanout left for surface escape: (ref, pad, via site), 0.2 mm F.Cu stub
DOGBONES = [('U14', 'A7', (-12.889, -34.161))]
byref = collections.defaultdict(list)
for p in pads: byref[p['ref']].append(p)
for net, srcs in SOURCES.items():
    for ref, pn, n in srcs:
        p = next(q for q in byref[ref] if q['pad'] == pn)
        assert p['net'] == net, (ref, pn, p['net'], net)
        place_vias(p, n)
        cu = [q for q in byref[ref] if q['net']]
        if len(cu) == 2 and ref[0] in 'C':
            gp = next(q for q in cu if q is not p)
            if gp['net'] == 'GND': place_vias(gp, n)

for ref, pn, (x, y) in DOGBONES:
    p = next(q for q in byref[ref] if q['pad'] == pn)
    line = LineString([(p['x'], p['y']), (x, y)])
    ok = via_ok(x, y, p['net']) and all(n == p['net'] or gg.distance(line) >= 0.1 + 0.1 for gg, n, lay, hole in obst
                                         if lay is None or 'F.Cu' in lay)
    if not ok: missing.append((ref, pn, p['net'], 0, 1)); continue
    new_vias.append(dict(x=x, y=y, size=0.45, drill=0.2, net=p['net']))
    stubs.append(dict(x1=p['x'], y1=p['y'], x2=x, y2=y, w=0.2, layer='F.Cu', net=p['net']))
    obst += [(Point(x, y).buffer(VIA_R), p['net'], None, False), (Point(x, y).buffer(DRILL_R), p['net'], None, True)]
allv = vias + new_vias
zones, report, jumpers = [], [], []
for layer, nets in LAYERS.items():
    names = [n for n, c in nets]
    seeds = list({(round(v['x'], 3), round(v['y'], 3)): (round(v['x'], 3), round(v['y'], 3), v['net'])
                  for v in allv if v['net'] in names and BGA.buffer(1).contains(Point(v['x'], v['y']))}.values())
    cells = voronoi_diagram(MultiPoint([(x, y) for x, y, n in seeds]), envelope=BGA.buffer(5))
    own = collections.defaultdict(list)
    for c in cells.geoms:
        for x, y, n in seeds:
            if c.contains(Point(x, y)): own[n].append(c.buffer(1e-4)); break
    forced = unary_union([f for (l, n), f in FORCE.items() if l == layer]) if any(l == layer for l, n in FORCE) else Polygon()
    taken, regions = Polygon(), {}
    for net, corr in nets:
        inside = unary_union(own[net]).intersection(BGA).intersection(corr)
        f = FORCE.get((layer, net))
        inside = inside.union(f) if f is not None else inside.difference(forced)
        outside = corr.difference(BGA).difference(taken)
        taken = taken.union(outside)
        regions[net] = unary_union([inside, outside])
    holes_other = lambda net: unary_union(
        [Point(v['x'], v['y']).buffer(v['size'] / 2 + ZCLR) for v in allv if v['net'] != net] +
        [Point(p['x'], p['y']).buffer(max(p['sx'], p['sy']) / 2 + ZCLR) for p in pads if '*.Cu' in p['layers'] and p['net'] != net] +
        [Point(p['x'], p['y']).buffer(p['drill'] / 2 + HOLE_CLR) for p in pads if p['drill'] > 0 and p['net'] != net])
    for prio, (net, corr) in enumerate(nets, start=1):
        reg = regions[net].intersection(edge.buffer(-EDGE))
        reg = reg.buffer(-GAP / 2, join_style='mitre').buffer(0)
        polys = [reg] if reg.geom_type == 'Polygon' else sorted(reg.geoms, key=lambda q: -q.area)
        polys = [q for q in polys if q.area > 1.0]
        cu = unary_union(polys).difference(holes_other(net)).buffer(-MIN_T / 2).buffer(MIN_T / 2)
        comps = [cu] if cu.geom_type == 'Polygon' else list(cu.geoms)
        mine = [v for v in allv if v['net'] == net]
        hit = collections.Counter()
        for v in mine:
            for i, c in enumerate(comps):
                if c.distance(Point(v['x'], v['y'])) < VIA_R: hit[i] += 1; break
            else: hit[-1] += 1
        main = max((i for i in hit if i >= 0), key=lambda i: hit[i], default=None)
        orphans = [v for v in mine if main is None or comps[main].distance(Point(v['x'], v['y'])) >= VIA_R]
        anchors = [v for v in mine if v not in orphans]
        for o in list(orphans):
            for m in sorted(anchors, key=lambda m: math.dist((m['x'], m['y']), (o['x'], o['y']))):
                if abs(m['x'] - o['x']) > 0.01: continue
                sg = 1 if m['y'] < o['y'] else -1
                for dx in (0.5, -0.5):
                    path = [(o['x'], o['y']), (o['x'] + dx, o['y'] - sg * 0.5), (m['x'] + dx, m['y'] + sg * 0.5), (m['x'], m['y'])]
                    line = LineString(path)
                    if all(geo.distance(line) >= JW / 2 + (HOLE_CLR if hole else CLR) or n == net
                           for geo, n, lay, hole in obst if hole or lay is None or 'B.Cu' in lay):
                        for a_, b_ in zip(path, path[1:]):
                            stubs.append(dict(x1=round(a_[0], 3), y1=round(a_[1], 3), x2=round(b_[0], 3), y2=round(b_[1], 3), w=JW, layer='B.Cu', net=net))
                        obst.append((line.buffer(JW / 2), net, ['B.Cu'], False))
                        jumpers.append((net, path)); orphans.remove(o); break
                if o not in orphans: break
        report.append(f"{layer} {net:14s} prio {prio}: {len(polys)} outline(s), area {sum(q.area for q in polys):6.0f} mm2, "
                      f"{len(mine)} vias, {len(orphans)} not on the main copper" +
                      (' ' + str([(v['x'], v['y']) for v in orphans[:8]]) if orphans else ''))
        for q in polys:
            zones.append(dict(layer=layer, net=net, priority=prio, clearance=ZCLR, min_thickness=MIN_T,
                              points=[[round(x, 3), round(y, 3)] for x, y in q.exterior.coords[:-1]]))
    zones.append(dict(layer=layer, net='GND', priority=0, clearance=ZCLR, min_thickness=MIN_T,
                      points=[[round(x, 3), round(y, 3)] for x, y in edge.buffer(-EDGE).simplify(0.05).exterior.coords[:-1]]))

print(f"{len(new_vias)} new vias, {len(stubs)} stubs")
for m in missing: print('SHORT', m)
for j in jumpers: print('B.Cu jumper', j[0], [tuple(round(c, 3) for c in pt) for pt in j[1]])
for r in report: print(r)
json.dump(dict(v18=V18, vias=new_vias, stubs=stubs, zones=zones), open(sys.argv[1], 'w'))
if len(sys.argv) > 2 and not sys.argv[2].startswith('--'):
    col = {'+0V85_VCCINT': '#d33', '+1V8': '#d90', '+0V9_MGTAVCC': '#39c', '+1V2_MGTAVTT': '#3a3', '+5V0_MAIN': '#a3c', 'GND': '#ccc'}
    x0, y0, x1, y1 = edge.bounds; s = 8
    for layer in LAYERS:
        out = [f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x0*s} {y0*s} {(x1-x0)*s} {(y1-y0)*s}" style="background:#fff">']
        out.append(f'<path d="M' + ' L'.join(f'{x*s},{y*s}' for x, y in edge.exterior.coords) + 'Z" fill="none" stroke="#000"/>')
        for z in zones:
            if z['layer'] == layer and z['net'] != 'GND':
                out.append(f'<path d="M' + ' L'.join(f'{x*s},{y*s}' for x, y in z['points']) + f'Z" fill="{col[z["net"]]}" fill-opacity="0.45" stroke="{col[z["net"]]}"/>')
        for v in allv:
            if v['net'] in col: out.append(f'<circle cx="{v["x"]*s}" cy="{v["y"]*s}" r="{VIA_R*s}" fill="{col[v["net"]]}"/>')
        out.append(f'<text x="{x0*s+10}" y="{y0*s+40}" font-size="36">{layer}</text></svg>')
        open(os.path.join(sys.argv[2], f'planes-{V18}-{layer}.svg'), 'w').write('\n'.join(out))
