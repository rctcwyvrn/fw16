"""One 1:1 template page (one half) as an SVG string, sized in mm.

The page is US Letter landscape, but everything is drawn inside the 279.4 x 210
mm overlap of Letter and A4 landscape, so an A4 printer at "Actual size" loses
nothing. The right half is the left half mirrored (x -> -x, r -> -r); legends
stay readable. Two 50 mm scale bars (horizontal and vertical) check that the
printer did not scale either axis.
"""

from legends import legend

PAGE_W, PAGE_H = 279.4, 215.9  # Letter landscape
SAFE_H = 210.0                 # A4 landscape height
CAP_W, CAP_H = 17.5, 16.5      # MBK / CFX Choc keycap
SWITCH = 15.0                  # PG1350 housing (top view)


def _pl(points, cls):
    d = " ".join(f"{x:.3f},{y:.3f}" for x, y in points)
    return f'<polyline class="{cls}" points="{d}"/>'


def _key(x, y, r, text, col_label):
    out = [f'<g transform="translate({x:.3f},{y:.3f}) rotate({r:.3f})">',
           f'<rect class="cap" x="{-CAP_W/2}" y="{-CAP_H/2}" width="{CAP_W}" height="{CAP_H}" rx="1"/>',
           f'<rect class="sw" x="{-SWITCH/2}" y="{-SWITCH/2}" width="{SWITCH}" height="{SWITCH}"/>',
           '<path class="cross" d="M -1.5 0 H 1.5 M 0 -1.5 V 1.5"/>',
           f'<text class="leg" x="0" y="5.5">{text}</text>']
    if col_label:
        out.append(f'<text class="col" x="0" y="{-CAP_H/2 - 2:.2f}">{col_label}</text>')
    out.append("</g>")
    return "".join(out)


def _scale_bars(x0, y0):
    out = [f'<path class="bar" d="M {x0} {y0} H {x0 + 50}"/>',
           f'<path class="bar" d="M {x0} {y0} V {y0 - 50}"/>']
    for i in range(6):
        out.append(f'<path class="tick" d="M {x0 + 10*i} {y0} v 2.5"/>')
        out.append(f'<path class="tick" d="M {x0} {y0 - 10*i} h -2.5"/>')
    out.append(f'<text class="note" x="{x0 + 25}" y="{y0 + 6}" text-anchor="middle">'
               '50 mm — measure: must read exactly 50</text>')
    out.append(f'<text class="note" x="{x0 + 3}" y="{y0 - 25}" text-anchor="start">50 mm</text>')
    return "".join(out)


def page(title, subtitle, points, case, mcu, right, thumb_order):
    """points: {name: (x, y, r)} left-half Ergogen frame; case/mcu: polylines."""
    sx = -1.0 if right else 1.0
    xs = [sx * x for pl in case for x, _ in pl]
    ys = [y for pl in case for _, y in pl]
    # centre the case outline between the header (y < 30) and the footer (y > 185)
    cx = PAGE_W / 2 - (min(xs) + max(xs)) / 2
    cy = 107.5 + (min(ys) + max(ys)) / 2

    def tx(x, y):
        return cx + sx * x, cy - y

    # one column label per matrix column, on its top-row key
    body = []
    for pl in case:
        body.append(_pl([tx(x, y) for x, y in pl], "case"))
    for pl in mcu:
        body.append(_pl([tx(x, y) for x, y in pl], "mcu"))
    mx = [tx(x, y) for pl in mcu for x, y in pl]
    if mx:
        body.append(f'<text class="note" x="{sum(p[0] for p in mx)/len(mx):.2f}" '
                    f'y="{sum(p[1] for p in mx)/len(mx):.2f}" text-anchor="middle">MCU</text>')
    for name, (x, y, r) in points.items():
        px, py = tx(x, y)
        col = name.split("_")[1] if name.endswith("_top") and name.startswith("matrix_") else None
        body.append(_key(px, py, -sx * r, legend(name, right, thumb_order), col))

    side = "RIGHT" if right else "LEFT"
    head = (f'<text class="title" x="14" y="14" text-anchor="start">{title} — {side} half</text>'
            f'<text class="note" x="14" y="20" text-anchor="start">{subtitle}</text>'
            '<text class="note" x="14" y="25" text-anchor="start">Print at ACTUAL SIZE / 100 % '
            '(not "fit to page"), Letter or A4. Check both 50 mm bars before typing.</text>')
    foot = _scale_bars(16, 186)
    legend_txt = ('<text class="note" x="14" y="200" text-anchor="start">solid: 17.5 x 16.5 keycap · '
                  'thin: 15 mm switch body · dashed: case outline incl. wall allowance · '
                  'dotted: controller zone</text>')
    style = ("<style>"
             ".cap{fill:none;stroke:#000;stroke-width:0.35}"
             ".sw{fill:none;stroke:#999;stroke-width:0.15}"
             ".cross{stroke:#000;stroke-width:0.2}"
             ".case{fill:none;stroke:#555;stroke-width:0.3;stroke-dasharray:2 1.2}"
             ".mcu{fill:none;stroke:#777;stroke-width:0.3;stroke-dasharray:0.5 0.8}"
             ".bar{stroke:#000;stroke-width:0.4;fill:none}.tick{stroke:#000;stroke-width:0.3}"
             "text{font-family:DejaVu Sans,sans-serif}"
             ".leg{font-size:3.5px;text-anchor:middle}.col{font-size:2.4px;fill:#444;text-anchor:middle}"
             ".title{font-size:5px;font-weight:bold}.note{font-size:2.8px}"
             "</style>")
    return (f'<svg xmlns="http://www.w3.org/2000/svg" width="{PAGE_W}mm" height="{PAGE_H}mm" '
            f'viewBox="0 0 {PAGE_W} {PAGE_H}">{style}{head}{"".join(body)}{legend_txt}{foot}</svg>')
