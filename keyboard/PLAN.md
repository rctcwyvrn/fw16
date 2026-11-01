# Keyboard — split Choc halves

From the design doc's "Split keyboard" section, starting from omnimeter's `plan/side-keyboard.md` sketch. The
halves stow in the belly tray ([tray/PLAN.md](tray/PLAN.md)), so the deck's 3.7 mm limit never applies; the
stock Framework keyboard stays in the deck. Wired first; wireless and the deck sled are later options.

## Design targets (from the doc)

- Two halves, each ≈ 130 × 100 mm with a thumb cluster, joined by their own cable.
- Kailh Choc (PG1350) in hot-swap sockets.
- QMK on an RP2040 per half for the wired version.
- One short USB-C cable from one half to a laptop expansion card.
- Tenting 15–30° deployed: by the tray halves as tenting bases, not legs on the case (decision 0003 amendment).

## Steps

1. **Layout.** 5–6 columns × 3 rows + 3–4 thumb keys per half; print paper templates at 1:1 and pick column
   stagger and thumb arc by typing on them. Check that two halves fit side by side in the tray's inner area
   (≈ 356 × 270 mm outer, minus walls). Output: `keyboard/layout/` (KLE or Ergogen config) and the key count
   for the parts order.
2. **Electronics.** Reuse HandyWork's routed RP2040 block (HandyWork itself is a 3.65 mm deck Input Module, so its board and
   Framework footprints don't carry over; see `keyboard/docs/electronics-options.md`) or use an
   RP2040 module per half; pick the inter-half link (TRRS vs USB-C; USB-C if the cable must also nest in the
   tray). Record the choice in `keyboard/docs/decisions/`.
3. **PCB.** Per-key hot-swap sockets and diode matrix in KiCad (via Konnect), reversible if both halves share one
   board. Keep the PCB + switch + keycap stack within the 10–12 mm stowed budget minus the case floor and leg.
   Optional: footprint for a PMW3360 trackball on the right thumb.
4. **Case.** Fusion 360 (`keyboard/case/fusion/case_body.py`): low-profile printed case per half with a solid floor and
   an FR4 switch plate; no legs (fold-flat legs don't fit the stack, decision 0003). Locating features (magnets or
   pegs) hold each half in the tray insert and on its tray half when tented. Export STEP/STL to `keyboard/case/`.
   Hands the stowed envelope to tray step 3.
5. **Firmware.** QMK keymap and split config in `keyboard/firmware/`; build via `nix develop` (add `qmk` and
   the ARM toolchain to `flake.nix`).
6. **Bring-up.** Matrix scan test per half, split handshake, USB-C to the laptop; layer and home-row-mod tuning.

## Later options (not in the first build)

- **Wireless:** nRF52840 + ZMK + a thin LiPo per half (the RP2040 is not low-power); a new PCB revision.
- **Deck sled:** a thin input module with an edge USB-C port, so the cable plugs into the deck. It must fill
  the deck slot and fit 3.7 mm (Input Module spec).

## Acceptance

- [ ] Both halves type over USB-C into the FW16 under NixOS.
- [ ] Each half's measured stowed stack (≈ 13.4 mm on estimates; the tray insert deepens to fit, decision 0003) fits the tray insert, both side by side.
- [ ] Each half holds at 20° on its tray half without wobble while typing (tray desk mode).
