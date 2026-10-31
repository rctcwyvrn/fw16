# Expansion Bay keepout — v1 (from STEP, calipers pending)

Mechanical envelope for a card in the Expansion Bay Shell. Every number here is derived from Framework's
published CAD; none has been checked against hardware yet. v2 replaces these with caliper measurements on
the real shell and laptop (Phase 2 Path 1 step 2). Until then, treat each limit as nominal, with no margin
taken off.

## Sources

| File | Repo @ commit | Used for |
| --- | --- | --- |
| `Mechanical/ExpansionBayShell/FW_UMA_MODULE_REF_0.stp` | `FrameworkComputer/ExpansionBay` @ `c9b8eef` (2025-09-09) | shell, fans, rear bracket, floor plate |
| `PCB Reference Design/FW_EXPANSION_BAY_PCB_REF__0807.stp` | same | reference PCB placed in the shell, interposer connectors, standoffs |
| `PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.stp` | same | bare PCB: outline and hole table |
| `Framework_16_all_UMA_asm_0408.stp` (inside `Framework Laptop 16 with Expansion Bay Shell CAD.zip`) | `FrameworkComputer/Framework-Laptop-16` @ `51a94f1` | **the ceiling** — chassis and input modules above the bay |
| `Mechanical/GraphicsModule/FW_GRAPHICS_MODULE_REF_00-1.stp` | ExpansionBay @ `c9b8eef` | cross-check: how high Framework built |

Local clones live at `C:\Users\17786\ExpansionBay` and `C:\Users\17786\Framework-Laptop-16`; neither is
vendored. Every STEP is a Creo export in which every product is named `design`, so parts were identified by
geometry, not by name.

Method: the cad-step tools (`step_bbox`, `step_features`), plus OpenCASCADE rays cast ±Y from the
reference board's faces on a 2 mm XZ grid, recording the first solid hit. Ray hits are exact B-rep
intersections. Per-part extents quoted as sizes are vertex boxes, accurate to a few tenths of a mm on curved
parts.

## Frame

All files share one frame, so they can be loaded together in Fusion with no moving.

- **X**: across the laptop, 0 at the centreline, ±178.29 at the outer walls.
- **Y**: up. Laptop bottom is Y −17.1 and the top of the deck is Y +5.1. Expect the model to come into
  Fusion lying on its side.
- **Z**: from the rear outer face (Z 0) into the laptop. The bay spans Z 0–80 and the full laptop runs to
  Z 270.
- Units mm.

## Board outline (reference PCB)

- Thickness **1.2 mm**: bottom face Y −10.65, top face Y **−9.45**. The interposer and standoffs fix the
  board's Y position, so a thicker card raises its top face by the difference and eats directly into the
  ceiling below.
- Main body X −57.45…61.20, Z 4.0…67.5 (118.65 × 63.5 mm). Rear corners R2, front corners R0.5.
- **Rear notch**: X −4.0…8.0, Z 4.0…9.8 (12 × 5.8 mm), clearing a shell feature at X 1–5.
- **Front tabs** run forward to Z 76.5. Between the tabs are notches that clear the shell's bosses:

| Feature | X | Z | Notes |
| --- | --- | --- | --- |
| Tab L | −45.45 … −28.00 | to 76.5 | |
| Notch | −27.00 … −11.35 | from 68.5 | clears shell boss at (−13.5, 73.4), which rises to Y −9.5 |
| **Tab C (interposer)** | −10.35 … 24.35 | to 76.5 | carries both FXBeam fields |
| Notch | 25.35 … 38.00 | from 68.5 | clears shell boss at (27.5, 73.4) |
| Tab R | 39.00 … 48.20 | to 76.5 | |
| Outer edges | < −45.45, > 48.20 | to 67.5 | |

Framework's `fw_expansion_bay_pcb_0{1,2,3}_0807.dxf` are **A2 drawing sheets**: 594 × 420 mm in sheet
coordinates, holding several views with dimensions and hatching. They are not a clean board outline and
can't be imported into KiCad as Edge.Cuts. The outline above was taken from the bare-PCB STEP's top face.
A clean outline DXF in KEEPOUT coordinates is a CAD deliverable for S2.

## Holes (bare PCB, world X, Z)

| Ø mm | Count | Positions | Purpose |
| --- | --- | --- | --- |
| 6.2 | 4 | (−53.64, 7.9), (54.0, 7.9), (50.9, 63.0), (−33.0, 72.0) | board mounting through standoffs; 7.4 mm square standoff body, Y −9.75…−8.05 |
| 2.7 | 2 | (−1.0, 70.55), (15.65, 70.55) | interposer screws into the SMT threaded inserts (bottom side, 4 × 4 mm, down to Y −11.25) |
| 1.5 | 4 | (−7.34, 74.89), (5.34, 74.89), (9.315, 66.21), (21.995, 66.21) | FXBeam alignment |
| 1.4 | 2 | (−1.0, 66.35), (15.655, 74.75) | FXBeam alignment |
| 2.0 | 6 | (−27, 69.5), (−11.35, 69.5), (25.35, 69.5), (38, 69.5), (−3, 8.8), (7, 8.8) | notch / rear-notch corner reliefs |
| 2.3–2.4 | 3 | (−43.35, 66.9), (45.55, 66.9), (46.15, 66.9) | small holes beside Tab L and Tab R; the X 45.55–46.15 pair are the ends of one slot, 2.3 mm wide and 2.9 mm long in X |
| 1.0 | 2 | (−45.95, 68.0), (48.7, 68.0) | **not holes**: R0.5 inside-corner fillets of the outline at the tab roots (checked in Fusion 2026-10-31, 0012) |

## Interposer (FXBeam) fields

- Two fields, centred at **X −1.0 and X 15.65 (16.65 mm apart)**, Z ≈ 65.3–75.8, each about 14 × 10.5 mm.
  "27.75 mm" is the FXBeam connector's *length* from its part name; it is not the spacing, contrary to
  `plan/phase-2-bay-bringup.md`.
- Mated height above the board top: connector body **4.45 mm** (Y −5.0); alignment bosses **5.86 mm**
  (Y −3.59).

## Vertical clearance above the board

Gap from the reference board's top face (Y −9.45) to the first solid above, in the full-laptop model:

| Zone | Z | Gap | What's above |
| --- | --- | --- | --- |
| **Front** | 44 – 76.5 | **7.8 mm** | underside of the input-module plate, Y −1.65; 8.55 mm between its ribs |
| **Middle** | 14 – 44 | **8.65 mm** | 0.8 mm plate, Y −0.8; 9.75 mm in a grid of pockets |
| Rear strip | 4 – 14 | 10.25 – 13.55 mm to the chassis | **but the shell's rear bracket fills it**: only **1.85 mm** above the board at Y −7.6 (X −58…62, Z 2–15) |

- The ceiling is uniform across X over the whole board width, so these are full-width bands.
- **Cross-check:** the Graphics Module's heatsink tops out at Y −1.6, the same as the front-zone ceiling. Framework
  filled exactly this gap. That module grows only downward (to Y −21.05) and rearward (to Z −20.2).
- The ExpansionBay STEPs alone show **no ceiling** over the open middle. Only the full-laptop model
  closes it.

## Clearance below the board

| Zone | Gap | What's below |
| --- | --- | --- |
| Most of the board (Z 15–76) | **1.4 mm** | 0.8 mm floor plate at Y −12.05 (279.6 × 55.9 mm, X ±139.8, Z 19.3–75.1) |
| Rear strip, Z 2–15 | **0.1 – 0.46 mm** | rear bracket |
| Near the interposer | 0 – 0.85 mm | shell bosses, threaded inserts |

Bottom-side parts are limited to passives of about 1 mm, and nothing goes behind Z 15.

## Fans and airflow

- Two fans, outboard of the board: X −142.6…−67.2 and 67.2…142.6, Z 1.7…77.7, Y −11.55…−3.7. Fan bodies
  are about 75 × 75 × 7.6 mm, their tops 5.45 mm above board-top level.
- Inlet and outlet directions and duct paths are **not yet mapped**. Inspect them in Fusion before sizing a
  heatsink (Path 1 step 5).

## Caveats

- The full-laptop model is Framework's "external CAD" for skins and printed parts. It is dated 2024-04
  (original FW16), and the chassis interior may be simplified, without ribs, foam or thermal pads. The bay
  internals are absent from it, which is why the bracket and boss numbers come from the shell file.
- Nothing above accounts for tolerance, screw heads, foam or flex. Take margin off when allocating.
- v2 must measure, with calipers on the real laptop: the front and middle ceilings, the floor gap, and the
  rear bracket height.

## v1.1 (2026-10-29, Path 1 scripts; accepted 2026-10-31, decision 0012)

Accepted as the design basis by the user on 2026-10-31, with one rule while the support-rubber question is open:
**no top-side parts at Z < 15** (assume `IFGP6_UMA_SUPPORT_RUBBER` stays with a card fitted). Originally proposed, not adopted. It comes from `bay/tools/card_zones.py` (1 mm ray grid,
`bay/cad/card-zones-{top,bottom}.dxf`, raw gaps in `bay/cad/card-zones-grid.json`), `strip_probe.py`,
`fan_probe.py` and `step_section.py`. Where it disagrees with
v1 above, v1 stands until the user decides.

### Height zones (card_zones.py)

`card-zones-top.dxf` has 106 rectangles and `card-zones-bottom.dxf` has 57, in the card/KiCad frame.
Each label is the gap floored to a design class (0, 0.5, 1, 1.4, 1.85, 2.5, 3.5, 4.5, 5.5, 6.5, 7.8,
8.65). Each cell takes the minimum of its 3 × 3 neighbourhood, so every rectangle is safe at any point
inside it, and obstacles grow by 1 mm. Raw gaps, X −50…50, by Z row:

| Z | Top (raw) | Bottom (raw) |
| --- | --- | --- |
| 4 – 11 | 13.55 to the chassis (0 at the standoffs and the rear-notch block) | 0.10 |
| 11 – 14 | steps 13.0 → 10.25 → 8.45 | 0.10 |
| 14 – 15 | 8.65 | 0.10 |
| 15 – 18 | 8.65 | 0.6 – 1.4, mixed |
| 18 – 23 | 8.65 (9.75 under grille holes) | **0.30** |
| 23 – 44 | 8.65 (9.75 under grille holes) | 1.40 |
| 44 – 47 | 7.8 | 1.40 |
| 47 – 60 | 8.55 between ribs, 7.8 on ribs | 1.40 |
| 60 – 68.5 | 7.8 / 8.55; 0 at standoffs, bosses and the interposer | 1.40, 0 at bosses |
| 68.5 – 76.5 | 0 – 8.55 (interposer, tabs) | **0.30** (0 at the inserts) |

Differences from v1:
- **Rear strip, top.** v1's "rear bracket at Y −7.6, 1.85 mm above the board, X −58…62, Z 2–15" is
  `IFGP6_UMA_SUPPORT_RUBBER` (named in Fusion), a solid in the *shell* file only
  (`FW_UMA_MODULE_REF_0.stp`: X −57.65…58.0, Y −10.75…−7.65, Z 2.7…14.7). It sits on the lock plate and
  fills the board's own volume (Y −10.65…−9.45); its top is 1.80 mm above board-top level. It is absent
  from `PCB_REF__0807`, where a board is fitted, and Framework's Dual SSD reference card puts its M.2
  drive (X −51.55…28.45, Z 12.13…34.13, up to Y −6.02) through the rubber's volume. So it reads as the
  no-card filler of the UMA (empty-bay) configuration.
- **Rear strip, what PCB_REF actually puts there** (`strip_probe.py`, 0.5 mm rays, board excluded).
  `IFGP6_UMA_LOCK_PLATE` (X −58.20…61.70, Y −11.25…−1.90, Z 2.10…15.00, 735 mm³) is a **0.5 mm sheet
  under the board**, Y −11.25…−10.75, lying on the shell floor. Its Y −1.90 top is a lanced latch tab
  bent up out of the sheet at X ≈ −2.75…−0.5, Z 2.1…7.5, inside the rear notch, so off the board. Over
  the board (X −50…50, Z 4–15, rear notch excluded) no PCB_REF solid stands above the board top; the
  only on-board hits are at the rear mounting holes: standoff bodies to Y −8.05 and screw heads to Y
  −7.35 (X −57.2…−50.2 and 50.3…57.3, Z 4.25…11.25), with the shell boss coming up through each hole.
  The M2 standoff (X 1.05…5.25, Z 3.6…7.8, to Y −7.90) and the block above it (X 0.71…5.59, to Y
  −6.00) are both inside the notch. The top of the strip is therefore set by the chassis in the
  full-laptop model: 13.55 to Z 11, then 13.0, 10.25 and 8.45 by Z 14, 8.65 from Z 14. The DXF caps it at
  the 8.65 class (7.8 over Z 13–14), unchanged. Calipers on a real shell decide whether the rubber
  comes out when a card is fitted.
- **Rear strip, bottom.** The 0.10 gap over Z 4–15 is the lock plate. Through its cut-outs (Ø ≈ 5 mm at
  both rear mounting holes; a window at X −10.2…−4.2, Z 4.25…7.75; a sliver at X 8.3…8.8) the next
  surface is the shell floor at Y −11.25, 0.60 below the board. Under the plate the shell floor is 1.6 mm
  thick (to Y −12.85). Just forward of the plate's front edge (Z 15–16) the gap is also 0.60, to the
  shell floor. Either way, no bottom-side parts behind Z 15.
- **Bottom, rear.** Over the main body (X −50…50), bottom parts are blocked from Z 4 to Z **23**, not
  Z 15: the gap is 0.10 to Z 15, then 0.30 over Z 18–23.
- **Bottom, under the reference fan connectors.** The solid at X 54.65…61.65, Z 18.45…32.25 sits *on the
  floor plate* (Y −12.05…−10.73), not on the board. Gaps above it: 0.08 at Z 18.5–22.5, 1.28 at
  Z 23–32. On the left (board edge X −57.45 to −51) the gap is 1.28 at Z 23.5–29.5. These cells mark
  where Framework's fan-cable connectors sit. They are not empty space, and the card must route the
  fans to meet them there (decision 0007's fan pass-through).
- **Bottom, front tabs.** On Tabs L, C and R (Z 68.5–76.5) the median gap below the board is 0.30, not
  1.4. The minimum is 0 at the inserts; the maximum is 1.4 at the tab roots.
- **Front ceiling.** Rib pitch is visible: 7.8 on the ribs, 8.55 between them, from Z 47. The DXF floors
  all of it to 7.8.
- **Middle ceiling.** v1's "9.75 mm in a grid of pockets" are holes in a perforated plate: Y −0.80…0,
  Z 14–44, full width, holes about 0.9 mm on a 1.6 mm X pitch, staggered. A solid 1 mm deck lies above at
  Y 0.30…1.30, so the holes are 1.1 mm dimples, not vents, in this model. Design to the plate (8.65).

### Fans and airflow (fan_probe.py, step_section.py)

- **Blower outlets face outboard.** Each fan is a scroll blower, impeller axis X ±102.88, Z 35.16. The
  scroll gap grows from the tongue (1.5 mm at 204° on the left fan, 336° on the right), round through
  the rear and the inboard side, to the outlet. The outlet opens between about 144° and 198° (left
  fan, toward −X) and 342° to 36° (right fan, toward +X). That is a mouth on the housing's outboard
  face at X ≈ ±133…142, Z ≈ 22…77, the same at every height from Y −10.9 to −5.5.
- **Exhaust is through the laptop's side walls.** At X ≈ ±177, the full-laptop model has slotted side
  vents at fan height (Y −11…−4.2): 5 mm slots at Z 21–25, 28–32, 34–38, 40–44, 52–56, 58–62, 65–69,
  and 71–75, with 1–2 mm ribs between them and a 7 mm web at Z 45–51. Outlet rays pass through these
  slots.
- **The card bay is walled off from the fans up to fan-top height.** Rays from X ±50 outward hit a wall
  at X ±65.9 (Y −10.9…−7.6) and ±67.2 (Y −5.5…−4.2) at every Z 15–75. The housing on its inboard side is
  1.5–4.9 mm from the impeller tips, and closed.
- **Above the fan tops it is open.** Between the fan top (Y −3.7) and the ceiling (Y −1.65 front,
  −0.80 middle) nothing stands between the card bay and the side walls: rays at Y −3.5, −3.0 and −2.5
  run from X ±50 to X ±175.8 at all Z ≥ 15. At Y −1.0 the front-zone ribs (X ±52…60) appear.
- **Inlets: unresolved.** With the impeller excluded, rays from the fan mid-plane show a top cover at
  Y −4.5 with openings near r ≈ 18 mm that reach the ceiling plate (runs of 6.8–8.3 mm). They also show
  bottom openings at r ≤ 12 where nothing is hit below, even the laptop's bottom cover. Either the
  model leaves out the bottom cover's grille, or there is a bottom intake. If the top-cover openings are
  real, the fans draw a little air from the ~2.9 mm band above the card bay walls. That is the only
  path for forced air over the card, and it runs along ±X.

### Heatsink envelope over the FPGA (middle zone)

Y stack, from KEEPOUT numbers and the PCB path's FFVB676 data:

| Item | Y | Note |
| --- | --- | --- |
| Board top | −9.45 | |
| Package lid top | −5.93 | 3.52 mm seated maximum (3.31 nominal) |
| Thermal pad | 0.30 | heatsink base at **−5.63** |
| Ceiling plate underside | −0.80 | middle zone, Z 14–44 |
| Margin | 0.50 | heatsink top at **−1.30** |
| **Heatsink height** | **4.33 mm** | base to top |

- **Footprint, full height:** Z 15–43 (1 mm inside the 8.65 band), any X on the board (−57.45…61.20).
  For a 27 × 27 mm package centred at Z 30, the package covers Z 16.5–43.5, which fits.
- **Forward extension, reduced height:** Z 44–63 under the 7.8 ceiling: top ≤ −2.15, which leaves
  3.48 mm. Stop by Z 63, because the interposer fields start at Z 65.3 and the mated connector top
  (Y −5.0) sits above the heatsink base (−5.63).
- **No rearward extension** until the rear-strip question is settled. If the support rubber stays in
  with a card, it would sit on the board top over X −57.65…58, Z 2.7…14.7 and leave no room for parts
  there. Without it, the strip has 8.45–13.55 mm to the chassis.
- **Clearance over parts:** a 3 mm inductor tops out at Y −6.45, 0.82 mm under the base, so the base can
  overhang the middle-zone inductors. No standoffs fall within Z 15–43.
- **Underneath:** any backplate or screw head sits in the 1.40 mm floor gap, only within Z 23–68.5 and
  inboard of the fan-connector regions. With 0.5 mm margin that is ≤ 0.9 mm. Nothing goes under the main
  body behind Z 23.
- **Fins:** if any forced air reaches the card, it moves along ±X toward the fans, over the bay walls,
  between Y −3.7 and the ceiling. Fins should run along X. Only the top ~2.4 mm of a 4.33 mm fin
  (Y −3.7…−1.30) sits in that band.

### Fusion checklist (2026-10-29, omnimeter `omnimeter-bay-assembly` doc; carried over)
- **Fan outlets face ±X — confirmed:** the right fan, seen from +X, shows an open rectangular mouth (rotor visible)
  on its outboard face.
- **Fan inlets, top — confirmed:** each fan's top cover has three large kidney-shaped intakes around the hub.
  The bottom-cover intake is still unchecked.
- **Rear strip:** see "Rear strip, top" above (`IFGP6_UMA_SUPPORT_RUBBER` vs `IFGP6_UMA_LOCK_PLATE`).
- Still open: side vents (real?), gap above the bay walls, ceiling grille deck, fan-connector solids
  (`IFGP6_UMA_FAN_BOARD_ASSY_V` at X −46…54.7, Z 58…76 and `IFGP6_UMA_FAN_FPC_PCBEZM_ASSY` are in the box).
