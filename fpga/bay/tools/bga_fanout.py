"""Selective dogbone fanout for U14: filter Konnect's plan_bga_fanout output into the vias/stubs we actually want.

Konnect's planner puts a dogbone via on every inner ball (576 on the FFVB676). The user chose a selective fanout
(2026-11-01): only balls that carry a net, none on no-connect balls, and none on the PCIe lane / refclk balls, which
are escaped by hand for signal integrity. Konnect adds one via per call, so the result is applied in KiCad by
pcbnew_add_fanout.py (run in KiCad's scripting console) and verified with Konnect afterwards.

usage: python3 bga_fanout.py <plan.json> <out.csv>
CSV columns: pad,net,pad_x,pad_y,via_x,via_y (board mm)
"""
import csv
import json
import re
import sys

HAND_ROUTED = re.compile(r"(GT_TX\d|PEX_TX\d|GT_REFCLK)")

plan = json.load(open(sys.argv[1]))
stubs = {s["pad"]: s for s in plan["stubs"]}
rows, skipped_nc, skipped_hand = [], 0, 0
for v in plan["vias"]:
    net = v["net"]
    if not net or net.startswith("unconnected"):
        skipped_nc += 1
        continue
    if HAND_ROUTED.search(net):
        skipped_hand += 1
        continue
    s = stubs[v["pad"]]
    rows.append((v["pad"], net, s["from"]["x"], s["from"]["y"], v["x"], v["y"]))
with open(sys.argv[2], "w", newline="") as f:
    w = csv.writer(f)
    w.writerow(["pad", "net", "pad_x", "pad_y", "via_x", "via_y"])
    w.writerows(rows)
print(f"{len(rows)} vias; skipped {skipped_nc} no-connect, {skipped_hand} hand-routed")
