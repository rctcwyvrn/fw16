# Fusion 360 script (run through the fusion360 MCP): one keyboard half's case body, per decision 0003.
#
# Builds, in a new document, from the board's Edge.Cuts DXF (y-up, mm; Fusion x/y = KiCad x/-y):
#   case        solid floor + walls on the PCB outline offset by `case_off`, M2 heat-set bosses at H1-H4,
#               open-top notches for J1 (link USB-C) and the RP2040-Zero's USB-C
#   pcb_ref     the board slab at its stack height, for fit checks (not printed)
# Every stack dimension is a user parameter, so the measured socket/switch values update the model.
# SIDE picks the board; the right half's coordinates are its own (its H1 is not the mirror of the left's).

import adsk.core, adsk.fusion

SIDE = "left"

BOARDS = {
    "left": {
        "dxf": r"C:\Users\17786\fw16-keyboard\left\outline.dxf",
        "holes": [(25.5, -45.5), (30.0, 5.5), (62.5, 4.5), (102.0, -6.0)],
        "j1": (119.1, -14.0, +1),        # origin x, y; mouth direction in x
        "mcu_usb": (110.0, -46.5),       # module USB-C mouth centre on the board's top edge
    },
    "right": {
        "dxf": r"C:\Users\17786\fw16-keyboard\right\outline.dxf",
        "holes": [(-23.5, -45.5), (-30.0, 5.5), (-62.5, 4.5), (-102.0, -6.0)],
        "j1": (-119.1, -14.0, -1),
        "mcu_usb": (-110.0, -46.5),
    },
}

# name, expression, comment (mm)
PARAMS = [
    ("floor_t", "1.2 mm", "solid case floor (0003)"),
    ("socket_gap", "2.0 mm", "socket body below PCB, 1.8 estimated + 0.2 clearance; measure"),
    ("pcb_t", "1.2 mm", "keyboard PCB"),
    ("plate_gap", "1.0 mm", "PCB top to plate bottom (Choc v1: plate top 2.2 above PCB)"),
    ("plate_t", "1.2 mm", "FR4 switch plate"),
    ("cap_above_pcb", "9.0 mm", "switch + MBK cap above PCB top, estimated; measure"),
    ("case_off", "3.0 mm", "case outline past the PCB edge (Ergogen case)"),
    ("wall_clear", "0.5 mm", "PCB edge to inner wall"),
    ("boss_od", "5.0 mm", "M2 boss"),
    ("insert_d", "3.2 mm", "M2 x 3 heat-set insert hole"),
    ("plug_w", "13.0 mm", "USB-C plug overmold 12.35 + clearance"),
    ("plug_h", "7.0 mm", "USB-C plug overmold 6.5 + clearance"),
]

MM = 0.1  # API lengths are cm


def run(context):
    app = adsk.core.Application.get()
    b = BOARDS[SIDE]
    doc = app.documents.add(adsk.core.DocumentTypes.FusionDesignDocumentType)
    design = adsk.fusion.Design.cast(app.activeProduct)
    design.designType = adsk.fusion.DesignTypes.ParametricDesignType
    root = design.rootComponent
    up = design.userParameters
    for name, expr, note in PARAMS:
        up.add(name, adsk.core.ValueInput.createByString(expr), "mm", note)
    val = lambda n: up.itemByName(n).value  # cm
    vi = adsk.core.ValueInput.createByString

    # outline sketch: pcb loop, inner-wall loop, outer loop
    im = app.importManager
    opt = im.createDXF2DImportOptions(b["dxf"], root.xYConstructionPlane)
    opt.isSingleSketchResult = True
    im.importToTarget(opt, root)
    sk = adsk.fusion.Sketch.cast(opt.results.item(0))
    sk.name = "outline"
    pcb_curves = sk.findConnectedCurves(sk.sketchCurves.item(0))
    far = adsk.core.Point3D.create(1000, 1000, 0)
    sk.offset(pcb_curves, far, val("wall_clear"))
    sk.offset(pcb_curves, far, val("case_off"))
    profs = sorted((sk.profiles.item(i) for i in range(sk.profiles.count)),
                   key=lambda p: p.areaProperties().area, reverse=True)
    inside, wall = profs[0], profs[1]          # biggest: inside the PCB; then the wall ring (2.5 mm wide)
    clear_ring = profs[2]

    ext = root.features.extrudeFeatures
    NEW = adsk.fusion.FeatureOperations.NewBodyFeatureOperation
    JOIN = adsk.fusion.FeatureOperations.JoinFeatureOperation
    CUT = adsk.fusion.FeatureOperations.CutFeatureOperation

    floor_profs = adsk.core.ObjectCollection.create()
    for p in (inside, wall, clear_ring):
        floor_profs.add(p)
    fl = ext.addSimple(floor_profs, vi("floor_t"), NEW)
    case = fl.bodies.item(0)
    case.name = "case"
    wi = ext.createInput(wall, JOIN)
    wi.setDistanceExtent(False, vi("floor_t + socket_gap + pcb_t + plate_gap + plate_t"))
    wi.participantBodies = [case]
    ext.add(wi)

    # bosses + insert holes
    bs = root.sketches.add(root.xYConstructionPlane)
    bs.name = "bosses"
    hs = root.sketches.add(root.xYConstructionPlane)
    hs.name = "insert_holes"
    for x, y in b["holes"]:
        c = adsk.core.Point3D.create(x * MM, -y * MM, 0)
        bs.sketchCurves.sketchCircles.addByCenterRadius(c, val("boss_od") / 2)
        hs.sketchCurves.sketchCircles.addByCenterRadius(c, val("insert_d") / 2)
    bp = adsk.core.ObjectCollection.create()
    for i in range(bs.profiles.count):
        bp.add(bs.profiles.item(i))
    bi = ext.createInput(bp, JOIN)
    bi.setDistanceExtent(False, vi("floor_t + socket_gap"))
    bi.participantBodies = [case]
    ext.add(bi)
    hp = adsk.core.ObjectCollection.create()
    for i in range(hs.profiles.count):
        hp.add(hs.profiles.item(i))
    hi = ext.createInput(hp, CUT)
    hi.setDistanceExtent(False, vi("floor_t + socket_gap"))
    hi.participantBodies = [case]
    ext.add(hi)

    # port notches: open-top boxes through the wall, bottom at the plug's lower edge
    pcb_top = "floor_t + socket_gap + pcb_t"
    jx, jy, jd = b["j1"]
    edge_x = jx + jd * 2.9                       # PCB edge
    ports = [  # (x0, x1, y, width param, z-centre expr, axis the notch runs along)
        (min(edge_x - jd * 1.0, edge_x + jd * 5.0), max(edge_x - jd * 1.0, edge_x + jd * 5.0),
         jy, "plug_w", pcb_top + " + 1.63 mm", "x"),
        (b["mcu_usb"][0], None, b["mcu_usb"][1], "plug_w", pcb_top + " + 2.6 mm", "y"),
    ]
    ps = root.sketches.add(root.xYConstructionPlane)
    ps.name = "port_notches"
    w = val("plug_w")
    for p in ports:
        if p[5] == "x":
            x0, x1, yc = p[0], p[1], p[2]
            a = adsk.core.Point3D.create(x0 * MM, -yc * MM - w / 2, 0)
            c = adsk.core.Point3D.create(x1 * MM, -yc * MM + w / 2, 0)
        else:
            xc, yc = p[0], p[2]
            a = adsk.core.Point3D.create(xc * MM - w / 2, -(yc + 1.0) * MM, 0)
            c = adsk.core.Point3D.create(xc * MM + w / 2, -(yc - 5.0) * MM, 0)
        ps.sketchCurves.sketchLines.addTwoPointRectangle(a, c)
    for i, p in enumerate(ports):
        prof = ps.profiles.item(i)
        ni = ext.createInput(prof, CUT)
        ni.startExtent = adsk.fusion.OffsetStartDefinition.create(vi("%s - plug_h / 2" % p[4]))
        ni.setDistanceExtent(False, vi("20 mm"))
        ni.participantBodies = [case]
        ext.add(ni)

    # reference board slab
    ri = ext.createInput(inside, NEW)
    ri.startExtent = adsk.fusion.OffsetStartDefinition.create(vi("floor_t + socket_gap"))
    ri.setDistanceExtent(False, vi("pcb_t"))
    ref = ext.add(ri).bodies.item(0)
    ref.name = "pcb_ref"
    ref.opacity = 0.4

    bb = case.boundingBox
    print("case bbox mm", [round(v * 10, 2) for v in (bb.minPoint.x, bb.minPoint.y, bb.minPoint.z,
                                                     bb.maxPoint.x, bb.maxPoint.y, bb.maxPoint.z)])
    print("profiles mm2", [round(p.areaProperties().area * 100, 1) for p in profs])
