"""Build every candidate: Ergogen -> 1:1 template PDF (two pages, L and R) + fit table.

Run from the repo root inside the dev shell:  python3 keyboard/layout/build.py
Ergogen output goes to _build/keyboard/layout/<candidate>/ (git-ignored); the
PDFs go to keyboard/layout/templates/ and are committed so they can be printed
without the shell. The fit table is printed to stdout for keyboard/layout/README.md.
"""

import glob
import os
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)

from dxf_outline import read_outline  # noqa: E402
from fit import HEADER, row  # noqa: E402
from points_yaml import read_points  # noqa: E402
from template_svg import page  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(HERE))
OUT = os.path.join(ROOT, "_build", "keyboard", "layout")
PDF_DIR = os.path.join(HERE, "templates")


def meta_name(config):
    with open(config) as f:
        for line in f:
            if line.strip().startswith("name:"):
                return line.split(":", 1)[1].strip()
    return os.path.basename(config)


def build(config):
    stem = os.path.splitext(os.path.basename(config))[0]
    out = os.path.join(OUT, stem)
    subprocess.run(["ergogen", config, "-o", out, "-d", "--clean"], check=True,
                   stdout=subprocess.DEVNULL)
    points = read_points(os.path.join(out, "points", "points.yaml"))
    case = read_outline(os.path.join(out, "outlines", "case.dxf"))
    mcu = read_outline(os.path.join(out, "outlines", "mcu.dxf"))
    thumbs = sorted((n for n in points if n.startswith("thumb_")), key=lambda n: points[n][0])
    title = meta_name(config)
    subtitle = (f"{len(points)} keys per half, {2 * len(points)} total · Choc pitch 18 x 17 mm · "
                f"source keyboard/layout/{stem}.yaml · Ergogen 4.2.1")
    svgs = []
    for right in (False, True):
        svg = os.path.join(out, f"{stem}-{'right' if right else 'left'}.svg")
        with open(svg, "w") as f:
            f.write(page(title, subtitle, points, case, mcu, right, thumbs))
        svgs.append(svg)
    os.makedirs(PDF_DIR, exist_ok=True)
    pdf = os.path.join(PDF_DIR, f"{stem}.pdf")
    subprocess.run(["rsvg-convert", "-f", "pdf", "-o", pdf, *svgs], check=True)
    return row(title.split(" ")[0], case, len(points))


def main():
    rows = [build(c) for c in sorted(glob.glob(os.path.join(HERE, "*.yaml")))]
    print(HEADER)
    print("\n".join(rows))


if __name__ == "__main__":
    main()
