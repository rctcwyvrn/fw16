"""Turn the Vivado package-pin CSV (au15p_pins.tcl) into a Konnect create_symbol `units` payload.

Usage: python3 fpga/bay/tools/au15p_symbol.py fpga/bay/card/fpga/xcau15p-ffvb676-pins.csv > units.json
One unit per bank (config bank 0, GTH 224/225/226, HP 64/65/66, HD 84/85/86), then core/MGT power, two GND units
and an NC unit. Pins split half left / half right on a 2.54 mm pitch; create_symbol sizes the body.
"""
import csv
import json
import re
import sys

ORDER = ["0", "224", "225", "226", "64", "65", "66", "84", "85", "86"]


def pin_type(func):
    if func == "NC":
        return "no_connect"
    if func.startswith(("GND", "VCC", "MGTAV", "MGTVCCAUX")):
        return "power_in"
    if func.startswith("MGTHTX"):
        return "output"
    if func.startswith(("MGTHRX", "MGTREFCLK")):
        return "input"
    if func.startswith(("MGTRREF", "MGTAVTTRCAL", "VP", "VN", "DXP", "DXN", "VREF")):
        return "passive"
    return "bidirectional"


def natural(s):
    return [int(t) if t.isdigit() else t for t in re.split(r"(\d+)", s)]


def unit(pins):
    half = (len(pins) + 1) // 2
    out = []
    for i, p in enumerate(pins):
        left = i < half
        row = i if left else i - half
        d = {"number": p["ball"], "name": p["pin_func"], "type": pin_type(p["pin_func"]), "y": round(-2.54 * row, 2)}
        if not left:
            d["angle"] = 180
        out.append(d)
    return {"pins": out}


def main(path):
    rows = list(csv.DictReader(open(path)))
    units = []
    for b in ORDER:
        units.append(unit(sorted((r for r in rows if r["bank"] == b), key=lambda r: natural(r["pin_func"]))))
    nobank = [r for r in rows if not r["bank"]]
    power = sorted((r for r in nobank if r["pin_func"] not in ("GND", "NC")), key=lambda r: natural(r["pin_func"] + r["ball"]))
    gnd = sorted((r for r in nobank if r["pin_func"] == "GND"), key=lambda r: natural(r["ball"]))
    nc = sorted((r for r in nobank if r["pin_func"] == "NC"), key=lambda r: natural(r["ball"]))
    units.append(unit(power))
    h = (len(gnd) + 1) // 2
    units += [unit(gnd[:h]), unit(gnd[h:]), unit(nc)]
    assert sum(len(u["pins"]) for u in units) == len(rows)
    json.dump(units, sys.stdout, separators=(",", ":"))


if __name__ == "__main__":
    main(sys.argv[1])
