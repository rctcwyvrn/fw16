# Keyboard layout candidates (K step 1, sync KS1)

Three Ergogen 4.2.1 configs. Each describes the left half; the right half is its mirror. Each has a printable
1:1 paper template so the user can pick by typing on paper. The pick freezes the key count and outline (**KS1**)
for the parts order, the PCB (K-elec), the case (K-case) and the tray insert.

| Candidate | Config | Template | Columns | Stagger (per column, outer→inner, mm) | Thumbs | Keys/half (total) |
| --- | --- | --- | --- | --- | --- | --- |
| **A** | `a-5col-mild-3thumb.yaml` | `templates/a-5col-mild-3thumb.pdf` | 5 | mild: 0, +4, +2, −2, −2 | 3 in a straight row | 18 (36) |
| **B** | `b-6col-corne-3fan.yaml` | `templates/b-6col-corne-3fan.pdf` | 6 | Corne-like: 0, 0, +6, +3, −4, −2 | 3 fanned, 12° steps | 21 (42) |
| **C** | `c-5col-strong-4arc.yaml` | `templates/c-5col-strong-4arc.pdf` | 5 | strong: 0, +10, +4, −6, −3 | 4 on an arc, 9° steps | 19 (38) |

Stagger values are relative to the previous column. They differ along three axes the user has to choose on:
- column count: 5 (A, C) vs 6 (B);
- stagger strength: mild (A) vs moderate (B) vs strong (C);
- thumb count and shape: 3 straight (A) vs 3 fanned (B) vs 4 on an arc (C).

A favourite can be a mix, e.g. B's thumb fan on A's columns. Mixing is a config edit plus a rebuild.

## Printing and using the templates

- Each PDF has two pages, LEFT and RIGHT half, on **US Letter landscape**. All content sits inside the
  Letter ∩ A4 area (279.4 × 210 mm), so printing on A4 also works.
- Print at **Actual size / 100 %**, not "Fit to page" or "Shrink oversized pages". Then measure **both** 50 mm
  bars (horizontal and vertical). If either is off by more than ~0.5 mm, the printer scaled the page; fix
  the setting and reprint.
- What the page shows: solid outlines are 17.5 × 16.5 mm Choc keycaps (MBK/CFX size), thin squares are the 15 mm
  switch housing, `+` marks the switch centre, the dashed line is the case outline including the wall allowance,
  and the dotted box is the controller zone. Legends are QWERTY plus T1… for the thumbs, for orientation only.
- Rest the fingers on the home row and reach each key. Things to judge: pinky and ring reach to the top row
  (stagger), whether the inner column is comfortable, and whether every thumb key is reachable without moving
  the hand (T3 in A, T4 in C).

## Tray fit (step 3)

Output of `build.py`. The tray inner area is taken as 356 × 270 mm outer (`keyboard/tray/KEEPOUT.md` v1:
footprint ±178.37 × 270.07) minus an assumed 3 mm printed wall each side, giving **350 × 264**. The halves lie side
by side in typing orientation with a 5 mm finger gap. The half outline is the convex hull of the key pitch boxes
(18 × 17 per key) plus the 22 × 40 mm controller zone, expanded by a **4 mm case-wall allowance** (1 mm keycap
clearance + 3 mm wall).

| Candidate | Keys/half (total) | Half outline W × D (mm) | Two halves + gap W × D | Tray inner 350 × 264: spare W × D | Fits |
| --- | --- | --- | --- | --- | --- |
| A | 18 (36) | 124.0 × 90.0 | 253.0 × 90.0 | 97.0 × 174.0 | yes |
| B | 21 (42) | 142.0 × 102.1 | 289.0 × 102.1 | 61.0 × 161.9 | yes |
| C | 19 (38) | 128.6 × 106.5 | 262.3 × 106.5 | 87.7 × 157.5 | yes |

All three fit with a lot of room to spare, so the tray does not constrain the pick. The spare depth (≥ 157 mm)
could hold the cable caddy, a trackball, or the battery from the tray plan. Not checked yet:
- The halves are 2-D outlines only. The 10–12 mm stowed thickness is K-case's job (KS3).
- The tray's latch and foot pockets may eat into the inner area at the rim. KEEPOUT v1 puts the feet at Z 14–20
  and Z 242–250, which is within 20 mm of the tray's front and rear edges, and the halves need only 90–107 of the
  264 mm depth.

## Rebuilding

From the repo root, in the dev shell (`ergogen`, `rsvg-convert` and `ezdxf` come from `flake.nix`):

```
python3 keyboard/layout/build.py
```

Ergogen's debug output lands in `_build/keyboard/layout/<candidate>/` (points, `case`/`mcu`/`caps` DXF and SVG
outlines). The PDFs are rewritten in `templates/` and the fit table is printed to stdout. Files:
- `build.py`: the driver.
- `points_yaml.py`: reads Ergogen's `points.yaml` without PyYAML.
- `dxf_outline.py`: reads outline DXF entities as polylines.
- `template_svg.py`: draws one page.
- `legends.py`: the QWERTY labels.
- `fit.py`: the tray table.
