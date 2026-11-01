# Belly tray

From the design doc's "Belly tray" section. Keyboard-only tray, not omnimeter's Phase 3 instrument tray.
**Printed first to iterate fit, vents and latches, then machined in aluminum** (user, 2026-10-29), following
the doc's "print the body, machine only the latch points" weight note as a possible end state.

## Design targets (from the doc)

- Matches the laptop footprint: ≈ 356 × 270 mm, ≈ 12 mm deep; whole kit ≈ 30 mm thick, fits a 16" sleeve.
- **Travel mode:** both halves nest side by side inside, tray latched under the laptop.
- **Desk mode:** unlatch, lift out the halves; **the tray splits into two tenting bases** with fold-out supports at
  15° and 30° (keyboard decision 0003: the cases have no legs). Each keyboard half locates on its tray half.
- Latched by clips or magnets; its own feet; never latches onto the laptop's feet.
- Vent cutouts and standoffs over the FW16's underside intakes.
- 400–700 g added; spare space for a cable caddy, trackball, or a < 100 Wh USB-C PD battery.

## Steps

1. **Outline from CAD.** Pull the FW16 bottom-cover outline, feet and intake grille positions into Fusion 360
   from Framework's published FW16 CAD (find which repo carries the bottom cover; if none does, this step
   waits for step 2's measurements); record them in `keyboard/tray/KEEPOUT.md` (v1, from CAD).
2. **Measure the real laptop** once it arrives: calipers on the outline, feet height, intake areas; KEEPOUT v2.
3. **Insert layout.** Size the insert depth from the keyboard's measured stowed stack (≈ 13.4 mm on estimates, deeper
   than decision 0001's 13 mm; keyboard decision 0003), place the keyboard's stowed envelope (keyboard step 4) and the cable path; decide what the
   spare space holds, and where the tray splits into its two tenting halves.
4. **Latch concept.** Magnets vs clips; print test coupons against the laptop's bottom cover; check that it doesn't
   scratch and that the tray holds when carried by the laptop.
5. **Printed tray v1.** Full-size print (split into tiles if the bed is smaller than 356 × 270 mm), with vents,
   standoffs, feet and the insert. Weigh it.
6. **Thermal test.** 30-min CPU (+ FPGA, once it exists) load with and without the tray; logger from the parts list.
   Iterate vents and standoffs until within 3 °C of no-tray.
7. **Machined version.** Redesign the accepted print for CNC (6061, wall and pocket radii for the tooling); get
   quotes; or machine only the latch points into a printed body if weight or cost argues for it.

## Acceptance

- [ ] Both halves stow; the tray latches and survives carry; the kit fits a 16" sleeve.
- [ ] 30-min load within 3 °C of no-tray.
- [ ] Tray + insert ≤ 700 g.
