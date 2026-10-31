# Fusion: a DXF imported onto a component's XZ plane maps DXF y to −Z, and analyzeInterference returns None when clean

Found 2026-10-31 building the card envelope (`fw16-bay-card`) through the Fusion MCP:
- `createDXF2DImportOptions(file, comp.xZConstructionPlane)` gives a sketch with xDirection (1,0,0) and yDirection
  (0,0,−1), so DXF y lands on −Z. Our DXFs are y-up KiCad-frame (`card_frame.to_dxf`: y = 70.55 − Z), so an occurrence
  translated by (7.325, 0, 70.55) puts them back in the laptop frame.
- The imported sketch also picked up the occurrence's Y offset a second time (origin at local Y −1.065 cm after the
  occurrence was already at Y −10.65 mm), so the body came out 10.65 mm low. Create the occurrence with Y = 0 and check
  the body's world bounding box after every import.
- `design.analyzeInterference(input)` returns `None`, not an empty collection, when nothing interferes; guard for it,
  or confirm with `TemporaryBRepManager` intersections and `measureManager.measureMinimumDistance`.
