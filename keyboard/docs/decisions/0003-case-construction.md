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
- Defaults chosen by Claude (override any of them): PETG; 2 mm walls on the Ergogen `case` outline (PCB + 3 mm);
  M2 × 3 heat-set inserts in the bosses; one design mirrored for the right half, except the right board's H1 boss,
  which sits at (−23.5, −45.5) rather than the mirror of the left H1; magnet pockets as placeholder tray locators
  until tray step 4 picks magnets or clips; openings for J1 (1 mm overhang) and the RP2040-Zero USB-C at the top
  edge, plus BOOT-button access.
