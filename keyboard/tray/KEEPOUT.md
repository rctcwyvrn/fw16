# FW16 underside keepout for the belly tray — v1 (from CAD, calipers pending)

What the tray mates against. Tray plan step 1 ([PLAN.md](PLAN.md)). Measured 2026-10-29 in Fusion 360 from
`Framework_16_all_UMA_asm_0408.stp` (Framework-Laptop-16 repo → "Framework Laptop 16 with Expansion Bay Shell
CAD.zip"; UMA = empty bay shell, no graphics module). The model is dated 2024-04 and simplified; v2 replaces these
numbers with caliper measurements once the laptop is here (tray step 2).

The Fusion working document is unsaved ("Untitled", assembly intent): the STEP import plus component
`tray_keepout_v1` with sketch `underside_keepout` on plane `underside_datum_Y-12.85`.

## Frame

Same frame as the bay STEPs (`fpga/bay/KEEPOUT.md`), in mm:
- **X** across the laptop, centred (±178.37).
- **Y** thickness, +Y toward the keyboard/lid. Lid closed: Y −17.13 (feet) … +5.14 (lid back).
- **Z** front-to-back depth: **Z 0 = rear** (hinge and expansion bay), Z 270.07 = front edge (touchpad side).

## Outline

| Feature | X | Z | Y (bottom) | Below datum |
| --- | --- | --- | --- | --- |
| Overall footprint | ±178.37 | −0.04 … 270.07 | — | — |
| **Underside datum** (bottom cover `DDOOR`, flat) | ±177.49 | 19.56 … 259.26 | −12.85 | 0 |
| Rear foot (`UMA_REAR_FOOT`, full-width bar on the bay shell) | ±178.29 | 14.36 … 19.06 | −17.10 | **4.25** |
| Front foot (`UMA_FRONT_FOOT`) | ±142.65 | 241.90 … 249.62 | −15.85 | **3.00** |
| Bay bottom-cover lip (`UMA_BOT_COVER` Body2, thin ring) | ±141.92 | 18.00 … 77.27 | −13.85 | 1.00 |
| Expansion-card release buttons (`EXC_BTN`, ×2) | ±146.79 … ±157.19 | 192.45 … 196.35 | −14.05 | 1.20 |

Feet are uneven (rear 4.25, front 3.00 below the datum), so the laptop sits tilted ≈ 0.3° (1.25 mm over ≈ 229 mm between feet) on a flat table.
The tray must carry its own feet and not latch onto these (design doc); it can pocket them to locate the laptop.
Lid-closed body without feet: Y −12.85 … +5.14 = **17.99 mm**; with a 12 mm tray whose pockets take the feet,
the stack is ≈ 30 mm, matching the doc.

## Air intakes (must stay open)

The only underside openings are in the **bay shell's bottom cover** (`UMA_BOT_COVER` Body1), feeding the bay fans
(which are the laptop's cooling; see `fpga/bay/KEEPOUT.md` § Fans and airflow):

| Grille | X | Z | Slots |
| --- | --- | --- | --- |
| Left | −133.91 … −77.77 | 22.28 … 72.12 | 2 rows × 23 |
| Right | 77.77 … 133.91 | 22.28 … 72.12 | 2 rows × 23 |

Slots 1.58 × 24.22 mm at 2.48 mm pitch (rows Z 22.28–46.50 and 47.90–72.12). Open area ≈ 1 760 mm² per side,
≈ 3 520 mm² total. The main bottom cover (`DDOOR`) has no vents; its only openings are the two release buttons.
Not in this model: whether a graphics module's bottom cover has different intakes (irrelevant while the bay holds
the FPGA shell, which keeps the UMA bottom cover).

## Sides (tray rim height limits)

| Feature | X | Z | Y | Consequence |
| --- | --- | --- | --- | --- |
| Expansion cards (3 per side, `EXPANSION_CARD_COV1`) | ±178.31 | 81.73 … 181.37 | −12.87 … −6.23 | rim ≤ Y −12.85 (datum) over this Z range on both sides, or cut-outs; card ports and cables must clear |
| Side LEDs (`SIDELED_L/R`) | ±177.48 … ±178.20 | 113.59 … 114.61 | −9.86 … −8.84 | inside the card zone; keep visible |
| Bay shell cap (`UMA_CAP`) | ±178.31 | 0 … 14.47 | −12.86 … +5.12 | rear face; the hinge opens from here, so no rim above the datum at Z < 14.5 |

## Open (for v2 / later steps)

- Calipers: datum flatness, foot heights, grille positions (tray step 2).
- The underside's side edges taper toward the front in the bottom view; check whether the flat datum narrows.
- Latch points: the flat `DDOOR` area between Z 81.7 and 241.9 is clear of feet, vents and buttons for magnets or
  clips; no screw positions were checked (don't use the laptop's screws).
