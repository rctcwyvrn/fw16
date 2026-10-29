Framework's ExpansionBay STEP files are Creo exports in which every product is named `design`, so parts have to be identified by geometry (bbox, placement, colour), not by name.

- `FW_EXPANSION_BAY_PCB_0807.stp` (1.7 MB) is mostly 2D curves with only 4 solids. Despite what `plan/phase-0-acquire.md` step 5 implies, it is not a populated board model.
- `FW_EXPANSION_BAY_PCB_REF__0807.stp` (22 MB, 43 solids) is the PCB placed in the full-width bay context (356.6 × 22.2 × 80.1 mm). It shares its frame with `FW_UMA_MODULE_REF_0.stp` (shell, 72 solids, 356.6 × 22.2 × 92.2 mm).
- Frame: Y is the thickness axis (−17.1 … +5.1 mm, about 22.2 mm total height); all three files are in mm.
- Checked at ExpansionBay commit c9b8eef (2025-09-09).

**The ceiling is only in the full-laptop model.** The ExpansionBay repo's STEPs model nothing above the open middle of the bay; the lid is the laptop's own chassis and input modules. That geometry is in `FrameworkComputer/Framework-Laptop-16` → `Framework Laptop 16 with Expansion Bay Shell CAD.zip` → `Framework_16_all_UMA_asm_0408.stp`, which is in the same coordinate frame as the bay STEPs (X ±178.29, Y −17.1…5.1, bay at Z 0–80). It isn't listed in that repo's README, so a STEP-only search misses it.
- Raycast from the reference PCB top (Y −9.45): the ceiling is **7.8 mm** at Z 44–76.5 (the input-module plate underside at Y −1.65), **8.65 mm** at Z 14–44, and 10.25–13.55 mm only in the rear strip Z < 14.
- Cross-check: the Graphics Module's heatsink top is Y −1.6, meaning Framework filled exactly this gap.
- The full-laptop model is simplified (no bay internals) and dated 2024-04, so confirm with calipers.
