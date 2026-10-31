# Ergogen 4.2.1: `hull` outlines take `points:` and size from each point's meta; debug outputs need `-d`

From the K-layout worker, 2026-10-31 (`keyboard/layout/`):
- `what: hull` takes an explicit `points:` list of point names and rejects `where:`/`size:`. It uses each point's
  `meta.width`/`height` (default 18 × 18), so set `key: {width: kx, height: ky}` in the zone, or the hull comes out
  0.5 mm too tall on each side at Choc 18 × 17 pitch.
- `points/points.yaml` is written only with `-d`. The dev shell's python3 has ezdxf but no PyYAML; the file's fixed
  layout parses with a regex.
- `rsvg-convert -f pdf a.svg b.svg` writes one multi-page PDF, and an SVG sized in mm with a matching viewBox prints at
  exactly 1:1. A CSS rule `text{text-anchor:middle}` overrides the `text-anchor` presentation attribute; set the anchor
  per class.
- In a multi-step outline, each step's `expand` applies to the union built so far, so earlier parts are expanded again
  by later ones. Give the earlier part `expand: 0`, or put the largest expand last (the layout B `case` outline came
  out 8 mm, not 4 mm, past the pitch boxes).
- The `kicad8` template's PCB loads in KiCad 10 (`kicad-cli pcb drc`), but its footprints are inline legacy `module`
  text with no library link. Custom footprints load from `footprints/*.js` beside `config.yaml`.
