"""Checks for step_rays.Solid.hit against the reference board in FW_EXPANSION_BAY_PCB_REF__0807.stp,
whose geometry KEEPOUT.md fixes: 1.2 mm thick, faces at Y -10.65 / -9.45, rear notch X -4..8, Z 4..9.8.

Covers the starts-inside-solid case (returns 0), a start inside the bounding box but outside the solid
(the notch), and plain hits from above and below. Then lists every other solid that straddles the board
top face, i.e. parts sunk into the board, which a top-side ray reports as 0 clearance.

usage: python3 step_rays_check.py <PCB_REF__0807.stp>   (exit 1 on any failure)
"""
import sys

from card_frame import PCB_BOT_Y, PCB_TOP_Y
from step_rays import load_solids

MID_Y = (PCB_TOP_Y + PCB_BOT_Y) / 2


def close(a, b, tol=0.02):
    return a is not None and b is not None and abs(a - b) <= tol


def main(path):
    solids = load_solids(path)
    boards = [s for s in solids if abs(s.hi[0] - s.lo[0] - 118.65) < 1.5 and s.hi[1] - s.lo[1] < 2.0]
    if len(boards) != 1:
        print("FAIL: expected one board solid, found %d" % len(boards))
        return 1
    b = boards[0]
    cases = [
        ("inside board, ray up", b.hit(0.0, MID_Y, 40.0, +1), 0.0),
        ("inside board, ray down", b.hit(0.0, MID_Y, 40.0, -1), 0.0),
        ("inside bbox, in rear notch, ray up", b.hit(2.0, MID_Y, 6.0, +1), None),
        ("0.45 above top face, ray down", b.hit(0.0, PCB_TOP_Y + 0.45, 40.0, -1), 0.45),
        ("0.35 below bottom face, ray up", b.hit(0.0, PCB_BOT_Y - 0.35, 40.0, +1), 0.35),
        ("0.45 above top face, ray up", b.hit(0.0, PCB_TOP_Y + 0.45, 40.0, +1), None),
        ("outside bbox in Z", b.hit(0.0, MID_Y, 2.0, +1), None),
    ]
    fails = 0
    for name, got, want in cases:
        ok = got is None if want is None else close(got, want)
        fails += not ok
        print("%s  %-36s got %-8s want %s" % ("ok  " if ok else "FAIL", name,
                                              "None" if got is None else "%.3f" % got, want))

    print("\nsolids straddling the board top face (Y %.2f), board excluded:" % PCB_TOP_Y)
    for s in solids:
        if s is b or not (s.lo[1] < PCB_TOP_Y - 0.01 and s.hi[1] > PCB_TOP_Y + 0.01):
            continue
        if s.hi[0] - s.lo[0] < 1.5 and s.hi[2] - s.lo[2] < 1.5:
            continue  # the ~140 FXBeam contacts, 1 x 0.4 mm each

        cx, cz = (s.lo[0] + s.hi[0]) / 2, (s.lo[2] + s.hi[2]) / 2
        d = s.hit(cx, PCB_TOP_Y, cz, +1)
        print("  X %7.2f..%7.2f  Y %6.2f..%6.2f  Z %6.2f..%6.2f  hit at centre: %s" % (
            s.lo[0], s.hi[0], s.lo[1], s.hi[1], s.lo[2], s.hi[2], "None" if d is None else "%.3f" % d))
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
