"""Run inside KiCad's PCB editor scripting console (Tools -> Scripting Console) with the card board open:

    exec(open(r"C:\\Users\\17786\\fw16-card\\pcbnew_add_fanout.py").read())

Adds one through via (0.45 mm pad / 0.2 mm drill, JLC multilayer limits 0.25/0.15) and one F.Cu stub (0.2 mm) per row
of u14_fanout.csv (from bga_fanout.py), on the row's net. Refuses to run twice: it stops if any via already sits at a
target position. Undo with Ctrl+Z if anything looks wrong; save only after checking.
"""
import csv
import os

import pcbnew

HERE = r"C:\Users\17786\fw16-card"
VIA_PAD, VIA_DRILL, STUB_W = 0.45, 0.2, 0.2

board = pcbnew.GetBoard()
mm = pcbnew.FromMM
rows = list(csv.DictReader(open(os.path.join(HERE, "u14_fanout.csv"))))
existing = {(v.GetPosition().x, v.GetPosition().y) for v in board.GetTracks() if isinstance(v, pcbnew.PCB_VIA)}
targets = {(mm(float(r["via_x"])), mm(float(r["via_y"]))) for r in rows}
if existing & targets:
    raise SystemExit(f"{len(existing & targets)} vias already at fanout positions; not adding again")

missing = sorted({r["net"] for r in rows if board.FindNet(r["net"]) is None})
if missing:
    raise SystemExit(f"nets not on the board: {missing}")

added = 0
for r in rows:
    net = board.FindNet(r["net"])
    pad = pcbnew.VECTOR2I(mm(float(r["pad_x"])), mm(float(r["pad_y"])))
    at = pcbnew.VECTOR2I(mm(float(r["via_x"])), mm(float(r["via_y"])))
    via = pcbnew.PCB_VIA(board)
    via.SetPosition(at)
    via.SetDrill(mm(VIA_DRILL))
    try:
        via.SetWidth(mm(VIA_PAD))
    except TypeError:  # KiCad 9+ padstack API
        via.SetWidth(pcbnew.F_Cu, mm(VIA_PAD))
    via.SetNet(net)
    board.Add(via)
    stub = pcbnew.PCB_TRACK(board)
    stub.SetStart(pad)
    stub.SetEnd(at)
    stub.SetWidth(mm(STUB_W))
    stub.SetLayer(pcbnew.F_Cu)
    stub.SetNet(net)
    board.Add(stub)
    added += 1
pcbnew.Refresh()
print(f"added {added} vias + stubs; check, then File > Save")
