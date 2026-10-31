"""QWERTY legends printed on the paper templates, keyed by Ergogen point name.

Only for orientation while typing on paper; the real keymap is K-fw's job.
Thumb keys get T1.. counted from the one under the alphas outward, toward the
split (T1 is the leftmost thumb key on the left half, the rightmost on the right).
"""

LEFT = {
    "outer": ("Tab", "Ctl", "Sft"),
    "pinky": ("Q", "A", "Z"),
    "ring": ("W", "S", "X"),
    "middle": ("E", "D", "C"),
    "index": ("R", "F", "V"),
    "inner": ("T", "G", "B"),
}
RIGHT = {
    "outer": ("Bsp", "'", "Ent"),
    "pinky": ("P", ";", "/"),
    "ring": ("O", "L", "."),
    "middle": ("I", "K", ","),
    "index": ("U", "J", "M"),
    "inner": ("Y", "H", "N"),
}
ROWS = ("top", "home", "bottom")


def legend(name, right, thumb_order):
    """Legend for point `name`; `thumb_order` lists thumb point names T1 first."""
    if name in thumb_order:
        return f"T{thumb_order.index(name) + 1}"
    zone, col, row = name.split("_")
    table = RIGHT if right else LEFT
    return table[col][ROWS.index(row)]
