"""Check an FXBeam footprint (and optionally its symbol) against fxbeam_pads.py.

Reads KiCad library files as text, read-only. The footprint is aligned on its two largest holes (the
interposer screw holes), so any origin works; pad numbering may be anything — each pad is identified by
position, then its number (and the symbol pin carrying that number) is compared with the README.

usage: python3 fxbeam_check.py <README.md> <footprint.kicad_mod> [symbol.kicad_sym]
exit status: 0 clean, 1 if any check failed.
"""
import re
import sys

from fxbeam_pads import mechanical, pads

TOL = 0.02


def footprint_pads(path):
    txt = open(path, encoding="utf-8").read()
    fp_margin = re.search(r"^\s*\(solder_mask_margin ([-\d.]+)\)", txt, re.M)
    res = []
    for m in re.finditer(r'\(pad "([^"]*)" (\w+) (\w+)(.*?)\(layers ([^)]*)\)(.*?)(?=\n\s*\(pad |\Z)', txt, re.S):
        num, typ, _shape, body, layers, rest = m.groups()
        at = re.search(r"\(at ([-\d.]+) ([-\d.]+)", body)
        size = re.search(r"\(size ([-\d.]+) ([-\d.]+)", body)
        drill = re.search(r"\(drill ([-\d.]+)", body + rest)
        margin = re.search(r"\(solder_mask_margin ([-\d.]+)", body + rest)
        res.append({
            "number": num, "type": typ, "x": float(at[1]), "y": float(at[2]),
            "w": float(size[1]), "h": float(size[2]), "layers": layers,
            "drill": float(drill[1]) if drill else None,
            "margin": float(margin[1]) if margin else (float(fp_margin[1]) if fp_margin else None),
        })
    return res


def symbol_pins(path):
    txt = open(path, encoding="utf-8").read()
    return {num: name for name, num in re.findall(
        r'\(pin \w+ \w+.*?\(name "([^"]*)".*?\(number "([^"]*)"', txt, re.S)}


def norm(s):
    s = s.upper().replace("3.3V", "3V3").replace("SDA", "DAT")
    s = re.sub(r"(\d)P$", r"\1", s)
    return re.sub(r"(\d)N$", r"\1#", s)


def main(readme, mod, sym=None):
    errors, notes = [], []
    want = pads(readme)
    mech = mechanical()
    have = footprint_pads(mod)
    pins = symbol_pins(sym) if sym else None

    holes = sorted((p for p in have if p["type"] == "np_thru_hole"), key=lambda p: -(p["drill"] or p["w"]))
    if len(holes) < 2:
        sys.exit("cannot align: fewer than two holes")
    ox = (holes[0]["x"] + holes[1]["x"]) / 2
    oy = (holes[0]["y"] + holes[1]["y"]) / 2

    contacts = [p for p in have if p["type"] == "smd" and p["w"] < 1 and p["h"] < 1]
    by_pos = {(round(w["x"], 2), round(w["y"], 2)): w for w in want}
    seen = set()
    for p in contacts:
        x, y = p["x"] - ox, p["y"] - oy
        match = next((w for w in want if abs(w["x"] - x) < TOL and abs(w["y"] - y) < TOL), None)
        if match is None:
            errors.append(f"pad {p['number']!r} at ({x:.3f},{y:.3f}) is not on the drawing's grid")
            continue
        seen.add(match["number"])
        where = f"{match['field']}{match['row']}{match['col']}"
        if {round(p["w"], 2), round(p["h"], 2)} != {0.7, 0.4}:
            errors.append(f"pad {p['number']} size {p['w']}x{p['h']}, drawing says 0.70x0.40")
        if (p["w"] > p["h"]) != True:
            errors.append(f"pad {p['number']} is rotated: long side must run along x")
        if p["margin"] is None or p["margin"] < mech["pad"]["mask_margin"] - 1e-9:
            notes.append(f"pad {p['number']} mask margin {p['margin']} < {mech['pad']['mask_margin']}")
        if p["number"] != match["number"]:
            notes.append(f"pad {p['number']!r} at {where} is README pin {match['number']} ({match['signal']})")
        if pins is not None:
            name = pins.get(p["number"])
            if name is None:
                errors.append(f"symbol has no pin {p['number']!r} (at {where}, {match['signal']})")
            elif norm(name) != norm(match["signal"]):
                errors.append(f"symbol pin {p['number']} is {name!r}; at {where} README says {match['signal']!r}")
    for w in want:
        if w["number"] not in seen:
            errors.append(f"README pin {w['number']} ({w['signal']}) has no pad at ({w['x']},{w['y']})")
    if len(contacts) != len(want):
        errors.append(f"{len(contacts)} contact pads, expected {len(want)}")

    hx = [p for p in have if p["type"] == "np_thru_hole"]
    for h in mech["holes"]:
        got = next((p for p in hx if abs(p["x"] - ox - h["x"]) < TOL and abs(p["y"] - oy - h["y"]) < TOL), None)
        d = got and (got["drill"] or got["w"])
        if got is None:
            errors.append(f"missing Ø{h['d']} NPTH at ({h['x']},{h['y']})")
        elif abs(d - h["d"]) > 0.051:
            errors.append(f"NPTH at ({h['x']},{h['y']}) is Ø{d}, drawing says Ø{h['d']}")
    if len(hx) != len(mech["holes"]):
        errors.append(f"{len(hx)} NPTH, expected {len(mech['holes'])}")
    for r in mech["rings"]:
        got = [p for p in have if p["type"] in ("smd", "connect") and p["w"] >= 1
               and abs(p["x"] - ox - r["x"]) < TOL and abs(p["y"] - oy - r["y"]) < TOL]
        if not got:
            errors.append(f"missing Ø{r['od']} SMT ring at ({r['x']},{r['y']})")
        elif "B.Cu" not in got[0]["layers"]:
            errors.append(f"SMT ring at ({r['x']},{r['y']}) is not on B.Cu")

    for n in notes:
        print("note:", n)
    for e in errors:
        print("ERROR:", e)
    print(f"{len(errors)} errors, {len(notes)} notes")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(*sys.argv[1:4]))
