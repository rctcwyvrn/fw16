"""Fan outlet and duct probe: horizontal rays fanned out from each blower's impeller axis at several
heights, reporting how far each direction runs before a wall. Short runs are the scroll housing; long
runs are the outlet and where its air goes. Vertical rays over the impeller show the inlets, and
horizontal rays from the card bay outward show the wall between the bay and each fan.

Impeller axes are the centres of the impeller solids in FW_EXPANSION_BAY_PCB_REF__0807.stp
(X ±102.88, Z 35.16; KEEPOUT.md "Fans and airflow"). Solids lying wholly inside R_SKIP of an axis are
the impeller itself and are ignored, and rays start at R_START so the blades don't block them.

usage: bay-python fan_probe.py <file.stp> [<file.stp> ...]
"""
import math
import sys

from step_rays import load_solids

AXES = {"left": (-102.88, 35.16), "right": (102.88, 35.16)}
LEVELS = (-10.9, -9.5, -7.6, -5.5, -4.2)  # fan body spans Y -11.55..-3.7
R_SKIP, R_START = 30.0, 29.5
STEP_DEG = 3
OPEN = 12.0  # a run longer than this past R_START counts as open


def wall_solids(solids, cx, cz):
    def inside(s):  # bbox within the axis-aligned square of half-side R_SKIP (an R 28 disc fits)
        return (s.lo[0] >= cx - R_SKIP and s.hi[0] <= cx + R_SKIP and
                s.lo[2] >= cz - R_SKIP and s.hi[2] <= cz + R_SKIP)
    return [s for s in solids if not inside(s)]


def run(solids, cx, cz, y, deg):
    a = math.radians(deg)
    d = (math.cos(a), 0.0, math.sin(a))
    p = (cx + R_START * d[0], y, cz + R_START * d[2])
    best, who = None, None
    for s in solids:
        t = s.cast(p, d)
        if t is not None and (best is None or t < best):
            best, who = t, s
    return best, who


def sectors(rows):
    """Merge consecutive open angles into (start, end, min run, max run) sectors."""
    out, cur = [], None
    for deg, t, _ in rows:
        is_open = t is None or t > OPEN
        if is_open:
            v = 999.0 if t is None else t
            cur = [deg, deg, v, v] if cur is None else [cur[0], deg, min(cur[2], v), max(cur[3], v)]
        elif cur is not None:
            out.append(cur); cur = None
    if cur is not None:
        if out and out[0][0] == 0:
            out[0] = [cur[0], out[0][1] + 360, min(cur[2], out[0][2]), max(cur[3], out[0][3])]
        else:
            out.append(cur)
    return out


def inlets(walls, cx, cz):
    """Vertical runs from the fan mid-plane over the impeller disc: a short run is a housing cover,
    a long one an inlet opening (and what it reaches)."""
    print("  inlets: run up / down from Y -7.6 (mm; '-' = nothing hit)")
    for r in (0.0, 6.0, 12.0, 18.0, 24.0, 28.0, 32.0):
        cells = []
        for deg in range(0, 360, 45):
            a = math.radians(deg)
            p = (cx + r * math.cos(a), -7.6, cz + r * math.sin(a))
            ups = [t for t in (s.cast(p, (0, 1, 0)) for s in walls) if t is not None]
            dns = [t for t in (s.cast(p, (0, -1, 0)) for s in walls) if t is not None]
            cells.append("%s/%s" % ("%.1f" % min(ups) if ups else "-", "%.1f" % min(dns) if dns else "-"))
        print("    r %4.1f  %s" % (r, "  ".join(cells)))


def bay_walls(solids, side):
    """Horizontal runs from inside the card bay (X ±50) outward toward a fan: where the bay's side
    wall is, at each height and depth. A run past the fan housing means the bay opens into it."""
    sign = 1 if side == "right" else -1
    print("\n%s bay wall: X where a ray from X %+.0f outward first hits (by Y row, Z column)" % (side, 50 * sign))
    zs = list(range(5, 80, 5))
    print("    Y \\ Z  " + " ".join("%6d" % z for z in zs))
    for y in (-10.9, -9.0, -7.6, -5.5, -4.2, -3.5, -3.0, -2.5, -1.0):
        row = []
        for z in zs:
            p = (50.0 * sign, y, float(z))
            ts = [t for t in (s.cast(p, (sign, 0, 0)) for s in solids) if t is not None]
            row.append("%6.1f" % (50 * sign + sign * min(ts)) if ts else "     -")
        print("  %6.2f   %s" % (y, " ".join(row)))


def side_vents(solids, side):
    """Rays from X ±150 (outboard of the fan housing) straight out through the laptop's side wall,
    1 mm apart in Z: '#' = wall hit within 40 mm, '.' = passes through (an opening)."""
    sign = 1 if side == "right" else -1
    print("\n%s side wall from X %+.0f outward, Z -5..119 (one char per mm):" % (side, 150 * sign))
    for y in (-11.0, -9.5, -7.6, -5.5, -4.2):
        row = ""
        for z in range(-5, 120):
            p = (150.0 * sign, y, float(z))
            ts = [t for t in (s.cast(p, (sign, 0, 0)) for s in solids) if t is not None]
            row += "#" if ts and min(ts) < 40 else "."
        print("  Y %6.2f  %s" % (y, row))
    print("            " + "".join(("%d" % (z // 10 % 10)) if z % 10 == 0 else " " for z in range(-5, 120)))


def main(*paths):
    solids = [s for p in paths for s in load_solids(p)]
    for side in ("left", "right"):
        bay_walls(solids, side)
        side_vents(solids, side)
    print("angles: 0 = +X (outboard on the right fan), 90 = +Z (forward, into the laptop), "
          "270 = -Z (rear face); run measured from R %.1f" % R_START)
    for name, (cx, cz) in AXES.items():
        walls = wall_solids(solids, cx, cz)
        print("\n%s fan, axis X %.2f Z %.2f, %d wall solids" % (name, cx, cz, len(walls)))
        inlets(walls, cx, cz)
        for y in LEVELS:
            rows = []
            for deg in range(0, 360, STEP_DEG):
                t, who = run(walls, cx, cz, y, deg)
                rows.append((deg, t, who))
            desc = ", ".join("%d..%d deg (run %s..%s)" % (a, b % 360 if b >= 360 else b,
                                                          "%.0f" % lo if lo < 999 else "inf",
                                                          "%.0f" % hi if hi < 999 else "inf")
                             for a, b, lo, hi in sectors(rows))
            print("  Y %6.2f open: %s" % (y, desc or "none"))
            if y == -7.6:
                for deg, t, who in rows[::4]:
                    hit = ("%.1f -> solid X %.0f..%.0f Z %.0f..%.0f Y %.1f..%.1f" % (
                        t, who.lo[0], who.hi[0], who.lo[2], who.hi[2], who.lo[1], who.hi[1])
                           if t is not None else "nothing")
                    print("      %3d deg  %s" % (deg, hit))


if __name__ == "__main__":
    main(*sys.argv[1:])
