"""Read key positions from Ergogen's debug output `points/points.yaml`.

The dev shell's python has no PyYAML, and points.yaml has a fixed shape:
each point is a top-level `name:` line followed by 4-space-indented `x:`,
`'y':` and `r:` scalars (then a nested `meta:` block we skip). So this reads
only those lines rather than parsing YAML in general.
"""

import re

_TOP = re.compile(r"^([A-Za-z0-9_]+):\s*$")
_FIELD = re.compile(r"^    '?(x|y|r)'?:\s*(-?[0-9.eE+-]+)\s*$")


def read_points(path):
    """Return {name: (x, y, r)} in Ergogen's frame (mm, y up, r in degrees CCW)."""
    points = {}
    name = None
    fields = {}
    with open(path) as f:
        for line in f:
            top = _TOP.match(line)
            if top:
                if name is not None:
                    points[name] = (fields["x"], fields["y"], fields["r"])
                name, fields = top.group(1), {}
                continue
            field = _FIELD.match(line)
            if field and name is not None:
                fields[field.group(1)] = float(field.group(2))
    if name is not None:
        points[name] = (fields["x"], fields["y"], fields["r"])
    return points
