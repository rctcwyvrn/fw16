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
| 1.0 | 2 | (−45.95, 68.0), (48.7, 68.0) | |

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
