# 0003 — Keyboard case construction (K-case, toward KS3)

**Date:** 2026-11-01
**Status:** accepted 2026-11-01 (user, case questions after both PCBs routed)

## Decision

- **Switch plate: a 1.2 mm FR4 plate per half from JLC**, ordered with the boards and screwed to the same M2 bosses
  (H1–H4) through standoffs. 13.8 mm switch cutouts. It sits in the switch's clip zone, so it adds no stack height.
  The plate is a third KiCad outline per half, drawn through Konnect.
- **Solid 1.2 mm case floor**, not pocketed under the sockets. Estimated stack: 12.7–13.2 mm against the 13 mm budget
  (0001), using `electronics-options.md` §4 numbers that haven't been measured yet.
- **Legs: two printed kickstand legs per half on a 2 mm steel dowel hinge**, with printed detents at 15° and 30°.
  The legs fold flush into the floor.
- **The case is drafted on estimates** (socket body 1.8 mm below the PCB, switch + MBK cap 9 mm above it). Every
  stack dimension is a named Fusion parameter so the measured values update the model. Until the parts arrive and are
  measured, the only prints are test coupons.

## Consequences

- With a solid floor, the only floor openings are the leg cutouts. Each folded leg fills its cutout (1.2 mm) and
  extends up into the 1.8 mm socket-depth gap below the PCB, so the legs can be up to ≈ 2.6 mm thick with clearance.
  The cutouts must avoid the socket bodies (B.CrtYd) and the bosses. The stowed thickness stays
  floor + gap + PCB + switch, with no extra for the legs.
- If the measured socket or switch height pushes the stack over 13 mm, there are two fixes: pocket the floor
  (≈ −1.8 mm), or deepen the tray insert. Either reopens this record.
- Defaults chosen by Claude (override any of them): PETG; 2.5 mm walls ending at the Ergogen `case` outline
  (PCB + 3 mm, 0.5 mm clearance to the PCB); M2 × 3 heat-set inserts in the bosses; one design mirrored for the right half, except the right board's H1 boss,
  which sits at (−23.5, −45.5) rather than the mirror of the left H1; magnet pockets as placeholder tray locators
  until tray step 4 picks magnets or clips; openings for J1 (1 mm overhang) and the RP2040-Zero USB-C at the top
  edge, plus BOOT-button access.

## Amendment 2026-11-01: deepen the tray insert, keep the solid floor

The first case body (`keyboard/case/fusion/case_body.py`) stacks 1.2 floor + 2.0 socket gap (1.8 + 0.2 clearance) +
1.2 PCB + 9.0 switch and cap ≈ 13.4 mm, still on estimates. **The user chose to deepen the tray insert** rather than
pocket the floor or drop the socket clearance. The stowed budget is now the measured stack, not 13 mm. Tray step 3
sizes the insert from the case model's `floor_t + socket_gap + pcb_t + cap_above_pcb` once the parts are measured,
plus its own clearance. The kit's ≈ 30 mm total grows by the same amount.

## Amendment 2026-11-01: no legs on the case; the tray halves are the tenting bases

Fold-flat legs don't fit inside the case's stack. The half pivots on its outer edge, so a leg near the inner edge
(≈ 125 mm from the pivot) square to the case must be ≈ 33 mm long for 15° and ≈ 72 mm for 30°. Folded, it would
lie under the key columns. The only free height there is the 1.2 mm floor plus the 2.0 mm socket gap, and that gap
is filled by socket bodies and diodes on a staggered grid. On the left board, the only free lane at least 5 mm wide
is 5.8 mm wide and ≈ 32 mm long (x 86–118, y 2.9–8.7), so most of a leg would be ≤ 1.2 mm thick. A
1.2 × 25 mm PETG strip 72 mm long buckles at a few newtons.

**The user chose tray halves as the tenting bases.** The tray splits into two halves for desk mode, and each half
carries fold-out supports that tent the keyboard half sitting on it. The tray's own depth hides the mechanism.

- The case loses its legs and leg cutouts. The floor is solid apart from the locating features.
- The case's locating features (magnets or pegs, tray step 4) now do two jobs: stowing in the tray insert, and
  holding the half on its tray half at 15° and 30°, so they must take typing load at an angle.
- Tenting needs the tray at the desk. The tray takes on the 15°/30° supports and the doc's tenting-rigidity risk.
- The "two printed kickstand legs" line above is superseded.

