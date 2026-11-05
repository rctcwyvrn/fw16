"""Placement of the card's remaining parts (0012 placement pass, zones approved 2026-11-05): power input and 5V0_MAIN
left of J1's power field, VCCINT buck left of U14 outside the heatsink envelope, 1V8/1V6 bucks and MGT LDOs right
of U14, config flash/pull-ups right of U14, fan connectors at the right edge, Tag-Connect on the right tab, debug
test points on B.Cu in the left region. Checks every courtyard against the board outline (0.3 mm inset), the
heatsink envelope (x +-20, y -55.55..-27.55, no new top parts), the rear strip (y < -56.55: low parts only on top,
nothing on the bottom) and every other courtyard on the same side.
usage: card_place.py OUT.json [render.svg] -> {ref: [x, y, rot, side]} and a report."""
import json, sys, re, math, board_copper as g
from shapely.geometry import Polygon, box
from shapely.affinity import rotate as srot, translate

P = {}   # ref: (x, y, rot, side)
def put(ref, x, y, r=0, side='F'): P[ref] = (x, y, r, side)

# --- A: power input, sense, 5V0_MAIN buck (front zone, left of J1's VSYS pins at x -13.7) ---
put('U2', -22.0, -6.5)                                   # eFuse
put('C3', -19.3, -5.6, 90); put('C4', -19.3, -8.2, 90)   # VSYS_GPU input
for i, r in enumerate(['R6', 'R7', 'R8', 'R9']): put(r, -25.0, -5.0 - 1.2 * i)
put('R10', -22.0, -9.4); put('C5', -20.0, -9.9); put('C6', -22.0, -10.6)
put('R11', -27.6, -9.8); put('R12', -27.6, -11.0)
put('R14', -28.6, -6.4)                                  # shunt VSYS_F -> VSYS_SNS
put('C7', -29.0, -3.9)                                   # VSYS_F bulk
put('U3', -34.3, -9.0); put('C8', -34.3, -6.3)           # INA237
put('R15', -38.6, -7.6); put('R16', -38.6, -8.8); put('R17', -38.6, -10.0)
put('R13', -38.6, -11.2); put('R20', -38.6, -12.4)
put('U4', -30.0, -14.5)                                  # LMR33630
put('C9', -24.6, -14.5, 90); put('C10', -21.8, -14.5, 90); put('C11', -25.0, -11.0)
put('C12', -34.5, -15.0, 90); put('C13', -34.5, -17.0, 90); put('R18', -34.5, -19.0); put('R19', -34.5, -20.2)
put('L1', -30.0, -21.5)
put('C14', -25.4, -20.5, 90); put('C15', -22.6, -20.5, 90); put('C16', -25.4, -25.3, 90)
put('C18', -22.6, -24.0); put('C21', -22.6, -25.2)
put('C19', -30.0, -26.0); put('C20', -33.6, -26.0)
# sequencer
put('U6', -40.0, -18.0); put('R35', -40.0, -21.0); put('R36', -40.0, -22.2); put('R37', -40.0, -23.4)
# --- B: VCCINT buck, middle band left of the heatsink envelope ---
put('U7', -30.5, -35.5); put('C23', -32.6, -35.5, 90); put('C22', -30.5, -33.6)
put('C28', -30.5, -31.4); put('R40', -34.5, -31.2)
put('L2', -25.0, -33.8)
put('C24', -21.4, -37.0, 90); put('C25', -21.4, -33.6, 90); put('C26', -24.0, -29.8); put('C27', -22.3, -27.0)
put('R38', -34.5, -37.4); put('R39', -34.5, -38.6); put('R35b', 0, 0) if False else None
# --- C: 1V8 / 1V6 bucks, MGT LDOs, aux LDOs: right of U14, front zone + middle band (x > 20) ---
put('U8', 24.0, -8.0); put('L3', 27.0, -8.0); put('C29', 30.5, -8.0); put('C30', 24.0, -5.6)
put('R41', 21.6, -10.2); put('R42', 21.6, -11.4); put('R43', 21.6, -12.6)
put('U9', 24.0, -14.0); put('L4', 27.0, -14.0); put('C31', 30.5, -14.0); put('C36', 30.5, -11.6)
put('C41', 34.2, -14.0); put('C32', 24.0, -11.8); put('R44', 21.6, -14.6); put('R45', 21.6, -15.8)
put('U12', 26.0, -20.0); put('C38', 22.6, -18.6); put('C39', 30.6, -20.0, 90); put('C40', 33.4, -20.0, 90)
put('C37', 22.6, -21.4); put('R46', 22.6, -22.6)
put('U13', 26.0, -26.0); put('C43', 22.6, -24.6); put('C44', 30.6, -26.0, 90); put('C45', 33.4, -26.0, 90)
put('C42', 22.6, -27.4); put('R47', 22.6, -28.6)
put('U10', 38.0, -9.0); put('C33', 38.0, -11.6); put('C34', 41.0, -9.0, 90)
put('U11', 38.0, -15.0); put('C35', 38.0, -17.6); put('C17', 41.0, -15.0, 90)
# --- D: config flash + pull-ups, JTAG, thermal level shift, misc (right of U14) ---
put('U5', 39.0, -51.5)
for i, r in enumerate(['R25', 'R26', 'R27', 'R28', 'R29', 'R30', 'R31', 'R32', 'R33', 'R34']):
    put(r, 23.0 + (i % 5) * 1.6, -51.0 - (i // 5) * 2.0, 90)
put('J4', 47.5, -20.0)                                     # Tag-Connect on the right tab
put('Q1', 24.0, -2.9 - 0.0) if False else None
put('Q1', 45.0, -6.0); put('Q2', 45.0, -10.0); put('R21', 48.5, -6.0, 90); put('R22', 48.5, -10.0, 90)
put('R23', 19.4, -4.4); put('R24', 19.4, -5.6)          # CLKREQ 0R / DNP
put('J2', 50.0, -49.0, 270); put('J3', 50.0, -35.0, 270); put('C1', 44.0, -42.0, 90)
# EEPROM + write-protect jumper + I2C pull-ups on the left tab
put('U1', -44.0, 1.0); put('C2', -44.0, 4.3); put('JP1', -38.5, 1.0); put('R3', -38.5, 3.6)
put('R4', -49.4, 0.4, 90); put('R5', -49.4, 2.4, 90)
put('R1', -16.6, 2.4, 90); put('R2', -16.6, 0.4, 90)    # interposer ID straps beside J1
# test points: B.Cu grid in the left region
tps = [f'TP{i}' for i in range(1, 24)]
for i, t in enumerate(tps):
    put(t, -60.0 + (i % 6) * 3.0, -14.0 - (i // 6) * 3.0, 0, 'B')

P = {k: v for k, v in P.items() if k and v}
TALL = ('L', 'J', 'C39', 'C44', 'C46')                   # parts that are not 'low' for the rear strip

# ---------- checks ----------
board = max((Polygon(l) for l in g.outline()), key=lambda p: p.area).buffer(-0.3)
heat = box(-20, -55.55, 20, -27.55)
rear = box(-70, -70, 60, -56.55)

def crt_local(blk):
    xs, ys = [], []
    for item in blk.split('\n\t\t(')[1:]:
        if 'CrtYd' in item[:400] or 'CrtYd' in item:
            if re.search(r'\(layer "[FB]\.CrtYd"\)', item):
                for c in re.findall(r'\((?:start|end|xy|center|mid) ([-\d.]+) ([-\d.]+)\)', item):
                    xs.append(float(c[0])); ys.append(float(c[1]))
    if not xs or max(xs) - min(xs) < 0.2: return box(-0.75, -0.75, 0.75, 0.75)   # test points: circle courtyard
    return box(min(xs), min(ys), max(xs), max(ys))

def placed_poly(local, x, y, r, side):
    p = local
    if side == 'B': p = srot(p, 0) ; p = Polygon([(-a, b) for a, b in p.exterior.coords])
    p = srot(p, -r, origin=(0, 0))                        # KiCad rotation is CCW on a y-down board
    return translate(p, x, y)

existing, newp = {'F': [], 'B': []}, []
for ref, layer, x, y, r, blk in g.fps():
    side = 'F' if layer == 'F.Cu' else 'B'
    if ref in P:
        nx, ny, nr, ns = P[ref]; newp.append((ref, placed_poly(crt_local(blk), nx, ny, nr, ns), ns))
    elif -65 < x < 54 and -67 < y < 6:
        existing[side].append((ref, placed_poly(crt_local(blk), x, y, r, side)))
missing = set(P) - {n[0] for n in newp}
bad = 0
if missing: print('unknown refs', missing); bad += 1
for i, (ref, poly, side) in enumerate(newp):
    if not board.contains(poly): print(f'{ref}: outside the board'); bad += 1
    if side == 'F' and poly.intersects(heat) and poly.intersection(heat).area > 1e-6: print(f'{ref}: in the heatsink envelope'); bad += 1
    if poly.intersects(rear) and poly.intersection(rear).area > 1e-6 and (side == 'B' or ref.startswith(TALL)): print(f'{ref}: rear strip'); bad += 1
    for oref, op in existing[side] + [(r2, p2) for r2, p2, s2 in newp[i + 1:] if s2 == side]:
        if poly.intersects(op) and poly.intersection(op).area > 1e-6: print(f'{ref} overlaps {oref}'); bad += 1
print(len(P), 'parts placed; problems', bad)
json.dump(P, open(sys.argv[1], 'w'))
if len(sys.argv) > 2:
    S, X0, Y0 = 8, -66, -68
    o = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{121*S}" height="{75*S}" style="background:#fff">']
    pth = lambda p, c, op: o.append('<path d="M ' + ' L '.join(f'{(a-X0)*S:.1f},{(b-Y0)*S:.1f}' for a, b in p.exterior.coords) + f' Z" fill="{c}" fill-opacity="{op}" stroke="#333" stroke-width="0.5"/>')
    pth(max((Polygon(l) for l in g.outline()), key=lambda p: p.area), '#efe', 1)
    pth(heat, '#f00', 0.08); pth(rear.intersection(board.buffer(0.3)), '#f80', 0.15)
    for s, c in (('F', '#88a'), ('B', '#8a8')):
        for ref, p in existing[s]: pth(p, c, 0.35)
    for ref, p, s in newp:
        pth(p, '#36f' if s == 'F' else '#3a3', 0.6)
        o.append(f'<text x="{(p.centroid.x-X0)*S:.0f}" y="{(p.centroid.y-Y0)*S+3:.0f}" font-size="7" text-anchor="middle">{ref}</text>')
    o.append('</svg>'); open(sys.argv[2], 'w').write('\n'.join(o))
