# Keyboard PCB — netlist-level spec (layout B, RP2040-Zero, USB-C link)

Written 2026-10-31 by the K-elec worker for `keyboard/PLAN.md` steps 2–3 and sync **KS2**. It builds on decision
[0001](decisions/0001-layout-controller-link-stack.md) (layout B, 42 keys, RP2040-Zero per half, USB-C link,
≈ 13 mm stack with Kailh hot-swap). Everything marked **proposed** is the worker's call and waits on the user
before KS2 freezes. It goes into a decision record once the user accepts it. The aim is that the orchestrator can
draw both boards in Konnect from this file, [`../layout/positions-b.csv`](../layout/positions-b.csv) and the
`pcb` outline, with no further design decisions.

## 1. Matrix (proposed)

- **6 columns × 4 rows per half, diodes COL2ROW.** Rows R0 top, R1 home, R2 bottom, R3 thumbs. Columns C0 outer
  … C5 inner, on both halves (so on the right half C0 is the far right column).
- The thumbs sit on **R3 × C3 / C4 / C5** (tuck, mid, reach), the three columns they sit under. R3 × C0–C2 are
  unused.
- This is exactly Corne's matrix (`LAYOUT_split_3x6_3`: 4 rows per half, thumbs at [3][3..5]), so Corne keymaps
  and QMK's community layout carry over to K-fw unchanged.
- Total: 10 matrix GPIO per half, 8 × 6 in QMK terms (`MATRIX_ROWS 8`, rows 0–3 left and 4–7 right).
- Alternatives considered: 7 × 3 (thumbs as a 7th column) and 5 × 5 also need 10 pins and fill all 21 positions.
  Neither saves a pin, and both lose Corne compatibility. 6 × 4 also routes naturally: each column trace runs
  down its own column and on to its thumb.
- **Diode direction, per pad:** switch pad 1 → column net. Switch pad 2 → diode **anode** (pad 2). Diode
  **cathode** (pad 1, the band) → row net. In stock `Diode_SMD:D_SOD-123`, pad 1 is the cathode (KiCad's
  convention for all `Diode_SMD` footprints; check the band silk on import). Current flows column → switch →
  diode → row, which is QMK's `COL2ROW` (the default).

## 2. GPIO map on the RP2040-Zero (proposed)

**Module pads.** There are 23 castellated pads at 2.54 mm pitch. Pad numbers follow the community footprint's
counter-clockwise walk ([tscircuit `rp2040-zero`](https://tscircuit.com/seveibar/rp2040-zero/blob/lib/PinOutCircuit.tsx)),
seen from the top with the USB-C end up:
- left edge, top → bottom: 1 `5V`, 2 `GND`, 3 `3V3`, 4 GP29, 5 GP28, 6 GP27, 7 GP26, 8 GP15, 9 GP14;
- bottom edge, left → right: 10 GP13, 11 GP12, 12 GP11, 13 GP10, 14 GP9;
- right edge, bottom → top: 15 GP8, 16 GP7, 17 GP6, 18 GP5, 19 GP4, 20 GP3, 21 GP2, 22 GP1, 23 GP0.

GP16 drives the on-board WS2812 and has no pad. GP17–GP25 are bottom solder points only and are not used.
**Verify the edge assignment against Waveshare's pinout drawing before drawing the footprint** (§7).

On the left board the module's left edge (pads 1–9) faces the keys. On the right board, which is mirrored, the
right edge (pads 15–23) faces the keys. The map is mirror-symmetric pad for pad, so the right board's routing is
the left board's routing mirrored:

| Net | Left GPIO (pad) | Right GPIO (pad) | Notes |
| --- | --- | --- | --- |
| R0 | GP29 (4) | GP3 (20) | |
| R1 | GP28 (5) | GP4 (19) | |
| R2 | GP27 (6) | GP5 (18) | |
| R3 | GP26 (7) | GP6 (17) | thumbs |
| C5 | GP15 (8) | GP7 (16) | inner column + reach thumb |
| C4 | GP14 (9) | GP8 (15) | index + mid thumb |
| C3 | GP13 (10) | GP9 (14) | middle + tuck thumb |
| C2 | GP12 (11) | GP10 (13) | ring |
| C1 | GP10 (13) | GP12 (11) | pinky |
| C0 | GP9 (14) | GP13 (10) | outer |
| **SERIAL** | **GP11 (12)** | **GP11 (12)** | **fixed**: QMK has one serial pin for both halves |
| **HAND** | **GP0 (23)** | **GP0 (23)** | **fixed**: one `SPLIT_HAND_PIN`. Left: 10 k to 3V3, right: 10 k to GND |
| spare | GP1–GP8 | GP1, GP2, GP14, GP15, GP26–29 | |

- **Fixed pins:** SERIAL and HAND must be the same GPIO on both halves. QMK has a single
  `SERIAL_USART_TX_PIN` and a single `SPLIT_HAND_PIN`; its RP2040 serial driver uses the same pins on both
  sides ([QMK serial driver](https://docs.qmk.fm/drivers/serial),
  [`serial_vendor.c`](https://github.com/qmk/qmk_firmware/blob/master/platforms/chibios/drivers/vendor/RP/RP2040/serial_vendor.c)).
  GP11 sits mid-bottom, so it reaches the link port on either board.
- **Matrix pins may be swapped freely within one half for routing.** That is a firmware-only change: QMK takes
  `MATRIX_ROW_PINS_RIGHT` / `MATRIX_COL_PINS_RIGHT` separately
  ([QMK split keyboard](https://docs.qmk.fm/features/split_keyboard)). Update this table in the same change.
- Any GPIO works for PIO serial ([QMK serial driver](https://docs.qmk.fm/drivers/serial)). GP26–29 are the ADC
  pins and are fine as plain digital I/O.

For K-fw (QMK `keyboard.json`, data-driven):

```json
"processor": "RP2040", "bootloader": "rp2040", "diode_direction": "COL2ROW",
"matrix_pins": { "rows": ["GP29","GP28","GP27","GP26"],
                 "cols": ["GP9","GP10","GP12","GP13","GP14","GP15"] },
"split": { "enabled": true,
           "serial": { "driver": "vendor", "pin": "GP11" },
           "handedness": { "pin": "GP0" },
           "matrix_pins": { "right": { "rows": ["GP3","GP4","GP5","GP6"],
                                       "cols": ["GP13","GP12","GP10","GP9","GP8","GP7"] } } }
```

Columns are listed C0 → C5. Half-duplex, so there is no `SERIAL_USART_FULL_DUPLEX`.

## 3. Boards: two mirrored PCBs, not one reversible (proposed)

| | **Two boards (left + mirrored right)** — proposed | **One reversible board** |
| --- | --- | --- |
| What gets built | two KiCad projects, `keyboard/pcb/left/` and `keyboard/pcb/right/`; the right board's placement is the left's mirrored (x → −x, rotation → −r, footprints **not** mirrored) | one board; the right half is the same PCB flipped over |
| Parts per side | everything single-sided: switches top, sockets + diodes bottom, module + link port top | every switch, socket and diode footprint on both layers (Ergogen's `choc` `reverse: true` does this). The module footprint goes on both faces with **solder jumpers on every module pad**, because flipping swaps the module's two edges and puts 5V/GND/3V3 on GPIO positions (Corne-style jumper pads) |
| Link circuit | different per board (§5: source on the left, sink on the right) | both circuits on the board, populated per side |
| USB-C link port | top-side SMD on both | must work from either face: a mid-mount part or two footprints |
| Fab cost | two designs: 2 × 5 pcs ≈ $10–20 more at JLCPCB | one design |
| MCP-lane cost | route twice; the right is a mirror, but KiCad has no "mirror the routing" command, so it is redrawn | route once, plus the jumper fields and double-sided parts |
| Firmware | `MATRIX_*_PINS_RIGHT` (already needed, §2) | same, plus jumper settings to get right per half |

Why two boards:
- The jumper field on a 23-pad castellated module is a lot of board area and soldering in the 22 × 40 mm
  controller zone.
- The link circuit differs per side anyway.
- Everything stays single-sided, which the 13 mm stack wants.

## 4. Laptop port and master/handedness (proposed)

- **Each RP2040-Zero has its own USB-C**; it is the only USB data path on the module (no D± pads). So the laptop
  cable plugs into a module's own port, which sits on the board's rear edge (§6). There is no laptop receptacle
  on our PCB.
- **The left half carries the laptop port and is always master** (proposed; question Q1). Link power then flows
  only left → right, which lets §5 use diodes instead of a bidirectional power path. The trade:
  - plugging the laptop into the right half's module port while the link is connected is unsupported;
  - the right half gets no power from its own module port in normal use.
- **The right module's USB-C still has to be reachable for flashing.** In a QMK split, each half is flashed over
  its own USB (BOOT button + UF2). The case (K-case) needs an opening at the right module's rear edge too, or a
  removable lid.
- **The module's BOOT button is on its top face, under the case lid.** It is needed for the first UF2 flash of each
  half. After that, K-fw puts `QK_BOOT` on a layer in the keymap so later flashes need no button. K-case either
  leaves a pin-hole over BOOT or accepts opening the lid once per half.
- **Master detection:** QMK's default on ARM, `SPLIT_USB_DETECT`: the half that enumerates on USB within
  `SPLIT_USB_TIMEOUT` (2 s) becomes master ([QMK split keyboard](https://docs.qmk.fm/features/split_keyboard)). No
  hardware needed: the RP2040-Zero brings no VBUS-sense GPIO out anyway.
- **Handedness: `SPLIT_HAND_PIN` on GP0**, with one 10 k strap per board (left to 3V3, right to GND; QMK's default
  is high = left). The same `.uf2` goes on both halves, and each half knows its side even when plugged in alone
  for flashing or a matrix test. Alternatives:
  - `MASTER_LEFT` (QMK's default) needs no pin, but a right half plugged in alone thinks it is left;
  - `EE_HANDS` needs no pin either, but needs a per-side flash target (`uf2-split-left` / `-right`) once per
    half;
  - `SPLIT_HAND_MATRIX_GRID` reads a matrix intersection by GPIO number, and our matrix GPIOs differ per side.

## 5. Link connector circuit (proposed)

Each board has a 16-pin USB-C receptacle (**J1**) on the inner edge, facing the other half. The link uses a
plain **USB 2.0 C-to-C cable**. Such a cable carries VBUS, GND, one CC wire and one D+/D− pair; it has no SBU and
no VCONN. The link carries **power + one half-duplex serial line on D+**. Nothing on the link is USB data.

**Why D+ and not SBU or CC:**
- SBU is not wired in USB 2.0 cables, so the link would need a full-featured (USB 3 / e-marked) cable.
- CC is how a laptop or charger decides whether to supply VBUS, so it can't carry serial.
- No compliant USB device drives D± above 3.6 V, so a misplugged laptop or charger can't push 5 V into the GPIO.
- J1 ties A6 to B6 and A7 to B7, so the link works with the plug either way up.
- QMK's half-duplex PIO serial idles on the RP2040's weak pull-ups and only drives the line high while sending
  ([`serial_vendor.c`](https://github.com/qmk/qmk_firmware/blob/master/platforms/chibios/drivers/vendor/RP/RP2040/serial_vendor.c);
  [serial docs](https://docs.qmk.fm/drivers/serial): "A pull-up resistor isn't required for RP2040").
  A USB host's 15 k D+ pull-down holds that idle level well under the 2 V attach threshold, so a host on the
  far end does not see a device attach.

**Why the halves need a power path at all:** the RP2040-Zero ties its 5V pad straight to USB VBUS with no diode
([Waveshare wiki](https://www.waveshare.com/wiki/RP2040-Zero), FAQ: "The VSYS pin of the RP2040 is connected to
the VUSB pin directly … you need to add a diode to avoid backflow"). Anything fed into one module's 5V pad
appears on that module's USB-C VBUS, and vice versa. Both directions are blocked with a Schottky diode.

**Left board (master) — the link port is a default-USB-power *source*:**
- `+5V` (module pad 1) → **D22** Schottky (anode at +5V) → `VSRC`.
- **Q1, Q2** P-MOSFETs in parallel: sources at `VSRC`, drains at `LINK_VBUS_SW`. Q1's gate on `LINK_CC1`, Q2's
  gate on `LINK_CC2`.
- **R1, R2** 56 k pull-ups from `+5V` to `LINK_CC1` and `LINK_CC2`. That is the USB Type-C Rp value for default
  USB power at 5 V.
- `LINK_VBUS_SW` → **F1** PTC fuse (200 mA hold) → `LINK_VBUS` (J1 A4/A9/B4/B9).
- How it switches: with nothing attached, CC sits at 5 V, Vgs ≈ +0.3 V, so the FETs are off and VBUS is off.
  When the right half's 1 k (below) is on the far end, CC ≈ 0.09 V, Vgs ≈ −4.6 V, and VBUS turns on. A laptop
  or phone presenting Rd 5.1 k gives ≈ 0.42 V and also turns on, so it gets default 5 V power, as from any
  source. The FET body diodes point link → `VSRC`, and D22 blocks that path, so nothing on the link can push
  current back into the module or the laptop.

**Right board (slave) — the link port presents Ra, not Rd:**
- **R1, R2** 1 k (Ra, 0.8–1.2 k) from `LINK_CC1` / `LINK_CC2` to GND. Not 5.1 k Rd: this keeps the handoff's
  rule "no CC pull-downs, so a source won't supply VBUS".
  - Through a passive cable, a laptop or charger sees **Ra on its CC and its other CC open**.
  - USB Type-C spec, source-side attach table: Ra / Open = "powered cable without sink attached", so a
    compliant source does not enable VBUS.
  - Our own left board's discrete FET gate does not tell Ra from Rd, so it powers the right half.
  - **This holds only for non-e-marked cables.** An e-marked cable (every cable rated above 60 W, e.g. a
    laptop-charger cable) has its own Ra on each plug's VCONN pin. A source on such a cable then sees Ra / Ra,
    the audio-accessory state: a source that implements it may apply VBUS and treat D± as analog audio. That
    does no damage (D22 and R3 absorb it), but it is outside the design.
  - **Requirement: the link cable is a USB 2.0 C-to-C cable without an e-marker** (any ≤ 60 W, USB 2.0-only
    cable). Label the link ports.
- `LINK_VBUS` → **D22** Schottky (anode at the link) → `+5V` (module pad 1). This blocks the right module's own
  USB VBUS (plugged in for flashing) from driving the link.
- **C1** 10 µF on `+5V` next to D22.

**Both boards:**
- `LINK_D` (J1 A6 + B6) → **R3** 330 Ω → `SERIAL` (GP11). It limits current if both ends drive at once or a
  host or charger drives D+. 330 Ω × ~150 pF cable + pins ≈ 50 ns, far below a bit time.
- `LINK_DN` (J1 A7 + B7) goes only to the ESD array. No GPIO in v1 (judgment call; see the report).
- **U2** USBLC6-2SC6 ESD array: I/O1 (pins 1, 6) on `LINK_D`, I/O2 (pins 3, 4) on `LINK_DN`, VBUS (pin 5) on
  `LINK_VBUS`, GND (pin 2).
- J1 GND pins (A1, A12, B1, B12) and the shell tabs go to GND.
- Power budget: left `+5V` ≈ laptop VBUS (≥ 4.75 V). The right `+5V` is about 0.6–0.7 V lower after two Schottkys
  (≈ 4.1 V worst case). That is still well above the module LDO's 3.3 V plus dropout at < 100 mA. The right half
  draws ≈ 25 mA, plus the WS2812 if firmware lights it.

**Misplug cases** (USB 2.0 C-to-C unless noted):

| Case | What happens | Damage? |
| --- | --- | --- |
| Link cable between the two link ports (normal) | left sees Ra → VBUS on through F1; right powered at ≈ 4.1–4.4 V; serial on D+ | — |
| Laptop or charger → **right** link port | source sees Ra/Open → no VBUS; D+ sees the host's 15 k pull-down via R3 | none |
| Laptop → **left** link port | laptop (DRP) sees Rp → attaches as sink; it presents Rd → our FETs turn on and the laptop draws default 5 V from its own other port through the keyboard, limited by F1 (200 mA hold) | none; pointless loop; label the port |
| Charger → **left** link port | source vs source: neither supplies VBUS | none |
| Legacy USB-A-to-C cable from a charger or PC (VBUS always on) → right link | 5 V through D22 powers the right half, as normal link power would | none |
| Same → left link | VBUS meets F1, the body diodes and D22 reversed: blocked | none |
| Link cable from a link port to the *other* half's module USB-C | the module port presents Rd. From the left link port: VBUS on into the right module's VUSB; harmless, but no serial (module D± are USB) | none, doesn't work |
| E-marked cable in the **left** link port, far end unplugged | the plug's own Ra pulls one CC to ≈ 0.09 V → FETs on → VBUS on a dangling plug (a compliant source would not) | none (current-limited by F1); outside the design |
| Laptop or charger → **right** link port over an **e-marked** cable | source sees Ra (ours) + Ra (the plug's) = audio-accessory state; it may apply VBUS (→ D22 powers the right half) and drive analog audio on D± (→ R3, ESD) | none expected; outside the design |
| Both module ports plugged into the laptop with the link connected | left supplies the link; right D22 blocks the right module's VBUS from the link; both enumerate, both think they're master | no damage; unsupported (§4) |

**Hot-plug:** USB-C mates GND and VBUS before the signal pins and has no sliding ring contacts. That removes the
TRRS short that makes QMK call TRRS splits "not hot pluggable"
([QMK split keyboard](https://docs.qmk.fm/features/split_keyboard)). QMK's own warning about USB-C links ("could
be mistaken for a normal USB connection and potentially short out the keyboard, depending on how it's wired") is
what the table above answers.

## 6. Parts, footprints and nets

**Libraries** (checked against `kicad-footprints` master on GitLab, 2026-10-31):
- Stock KiCad has **no Choc and no RP2040-Zero footprint**: `Button_Switch_Keyboard.pretty` holds only Cherry
  MX and Matias.
- Proposed: a project-local library **`fw16kb.pretty`** under `keyboard/pcb/` with the two custom footprints
  below. Everything else comes from stock libraries.

| Ref | Qty/half | Part | Footprint | Notes |
| --- | --- | --- | --- | --- |
| SW1–SW21 | 21 | Kailh Choc v1 (PG1350) in a Kailh **CPG135001S30** socket | `fw16kb:Choc_v1_Hotswap` (new, below) | switch on top, socket on bottom |
| D1–D21 | 21 | 1N4148W | `Diode_SMD:D_SOD-123` | **bottom**, key-local (0, +4.7) mm, i.e. 4.7 mm toward the user; rotated with the key |
| U1 | 1 | Waveshare RP2040-Zero | `fw16kb:RP2040-Zero_Castellated` (new, below) | top, USB-C end flush with the rear edge |
| J1 | 1 | HRO TYPE-C-31-M-12 (LCSC C165948), 16-pin | `Connector_USB:USB_C_Receptacle_HRO_TYPE-C-31-M-12` | top, mouth on the inner edge. 0.5 mm-class pads: hand-solderable with flux and a fine tip, or JLCPCB-assembled |
| U2 | 1 | USBLC6-2SC6 | `Package_TO_SOT_SMD:SOT-23-6` | |
| R3 | 1 | 330 Ω | `Resistor_SMD:R_0805_2012Metric` | 0805 throughout for hand soldering |
| R4 | 1 | 10 k (HAND) | `Resistor_SMD:R_0805_2012Metric` | left to 3V3, right to GND |
| D22 | 1 | B5819W Schottky (1 A, 40 V) | `Diode_SMD:D_SOD-123` | pad 1 = cathode. Left: cathode at `VSRC`; right: cathode at `+5V` |
| R1, R2 | 2 | left 56 k; right 1 k | `Resistor_SMD:R_0805_2012Metric` | |
| Q1, Q2 | 2 | AO3401A P-MOSFET | `Package_TO_SOT_SMD:SOT-23` | **left only**; pin 1 G, 2 S, 3 D (check the symbol's pin map against the AO3401A datasheet) |
| F1 | 1 | PTC, 200 mA hold, 0805 | `Fuse:Fuse_0805_2012Metric` | **left only**; pick a stocked LCSC part on order day |
| C1 | 1 | 10 µF 10 V X5R | `Capacitor_SMD:C_0805_2012Metric` | **right only** |

**`fw16kb:Choc_v1_Hotswap`:**
- Geometry copied from Ergogen 4.2.1's built-in `choc` footprint with `hotswap: true` (Ergogen is MIT; this is
  the footprint the probe in §8 placed):
  - NPTH 3.429 mm at (0, 0);
  - NPTH 1.7018 mm at (±5.5, 0);
  - socket NPTH 3.0 mm at (0, −5.95) and (5, −3.75);
  - **pad 1** SMD 2.6 × 2.6 on B.Cu at (−3.275, −5.95) → column;
  - **pad 2** SMD 2.6 × 2.6 on B.Cu at (8.275, −3.75) → diode anode.
- Add a 15 × 15 F.CrtYd, a B.CrtYd around the socket body, and a 17.5 × 16.5 Dwgs.User keycap box.
- Ceoloide's `switch_choc_v1_v2` has the same pad positions but is **CC-BY-NC-SA-4.0**. Don't copy it.
- The diode at (0, +4.7) on the bottom clears the socket body, which ends near y ≈ −1.5 on community silk, by
  > 5 mm. **Check against the CPG135001S30 drawing or calipers** when the sockets arrive.

**`fw16kb:RP2040-Zero_Castellated`:**
- 23 SMD pads at 2.54 mm pitch: 9 + 5 + 9, pad numbers and functions as in §2.
- Body 18 × 23.5 mm. Take pad positions and the edge offset from Waveshare's dimension drawing
  (`RP2040-Zero-details-size.jpg` on the [wiki](https://www.waveshare.com/wiki/RP2040-Zero)); not taken from
  memory here.
- No copper on F.Cu under the module apart from the pads: its bottom has exposed test points and GP17–25 pads.
  Route under it on B.Cu only.
- The 2 × 9 through-hole header variant (15.24 mm rows, on community libraries) is **not** proposed: headers
  add ≈ 2.5 mm under the module and drop the bottom-edge GPIO this map uses.

**Nets** (same names on both boards; the GPIO behind each is in §2):

| Net | Members |
| --- | --- |
| C0 | SW1.1, SW7.1, SW13.1, U1 C0 pad |
| C1 | SW2.1, SW8.1, SW14.1, U1 C1 pad |
| C2 | SW3.1, SW9.1, SW15.1, U1 C2 pad |
| C3 | SW4.1, SW10.1, SW16.1, SW19.1, U1 C3 pad |
| C4 | SW5.1, SW11.1, SW17.1, SW20.1, U1 C4 pad |
| C5 | SW6.1, SW12.1, SW18.1, SW21.1, U1 C5 pad |
| R0 | D1.1 … D6.1, U1 R0 pad |
| R1 | D7.1 … D12.1, U1 R1 pad |
| R2 | D13.1 … D18.1, U1 R2 pad |
| R3 | D19.1, D20.1, D21.1, U1 R3 pad |
| K*n* (n = 1…21) | SW*n*.2, D*n*.2 |
| SERIAL | U1.12 (GP11), R3.1 |
| LINK_D | R3.2, J1.A6, J1.B6, U2.1, U2.6 |
| LINK_DN | J1.A7, J1.B7, U2.3, U2.4 |
| LINK_CC1 | J1.A5, R1.1, Q1.G (left) |
| LINK_CC2 | J1.B5, R2.1, Q2.G (left) |
| LINK_VBUS | J1.A4, J1.A9, J1.B4, J1.B9, U2.5; left: F1.2; right: D22.A |
| HAND | U1.23 (GP0), R4.1 |
| +5V | U1.1; left: D22.A, R1.2, R2.2; right: D22.K, C1.1 |
| +3V3 | U1.3; left: R4.2 |
| GND | U1.2, J1.A1/A12/B1/B12/shell, U2.2; right: R1.2, R2.2, R4.2, C1.2 |
| VSRC (left) | D22.K, Q1.S, Q2.S |
| LINK_VBUS_SW (left) | Q1.D, Q2.D, F1.1 |

The SW → column and diode → row assignment per key is in `positions-b.csv` (the `R*x*C*` note column).

## 7. Placement, outline and stack

- **Positions:** [`../layout/positions-b.csv`](../layout/positions-b.csv), written by
  [`../layout/positions.py`](../layout/positions.py) from Ergogen's points.
  - KiCad frame: mm, y down. Origin at the left board's outer-bottom key centre.
  - Rows L and R. The right board is the exact mirror: x → −x, rotation → −r, same unmirrored footprints.
  - Thumbs rotate −12° and −24° on the left (+ on the right); each thumb's diode rotates with its key.
- **Edge.Cuts:** the `pcb` outline in `../layout/b-6col-corne-3fan.yaml`. Ergogen writes it to
  `_build/keyboard/layout/b-6col-corne-3fan/outlines/pcb.dxf`.
  - It is the key pitch boxes plus the controller zone, 1 mm past each: **132 × 88 mm per half**,
    x −10 … 122, y −52.5 … 35.6 in the CSV frame.
  - The case's 1 mm keycap clearance + 3 mm wall sit outside it (K-case).
  - Mirror the DXF in x for the right board.
- **Module:** U1 centre at (110, −34.75), USB end on the controller zone's rear edge (y = −46.5). The laptop
  cable leaves to the rear on the left board; on the right board that same rear opening is the flashing port.
- **Link port:** J1's mouth on the inner edge (x = 122, centred at y = −14), below the module and clear of the
  thumb fan. It points straight at the other half's J1.
- **Stack:** sockets (≈ 1.8 mm) and SOD-123 diodes (≈ 1.1 mm) hang under the PCB, together. The module
  (≈ 1 mm + parts) and J1 (≈ 3.2 mm) sit on top, in the controller zone where there are no keycaps. That is
  inside decision 0001's ≈ 13 mm with a 1.2 mm PCB. K-case owns the floor pockets.
- **Mounting holes:** not specified. Choc at 18 × 17 pitch leaves no room between keys, so holes would go in the
  thumb-to-matrix gap and the controller zone corners. That is K-case's decision; see question Q5 in the report.

## 8. Ergogen → KiCad: what 4.2.1 can do (tested)

- **Yes, Ergogen 4.2.1 emits a KiCad PCB with footprints placed.** Tested with the `kicad8` template; probe config
  in `_build/keyboard/pcb-probe/`. It writes `(kicad_pcb (version 20240108) …)`. KiCad 10.0.7's
  `kicad-cli pcb drc` loads it: 42 footprints at the CSV's coordinates, nets C*n*/R*n*/per-key assigned, 0
  violations, 137 unconnected items (unrouted, as expected).
- **Its footprints don't match the libraries above:**
  - The built-ins (`choc` → `PG1350`, `diode` → `ComboDiode`) are KiCad-5 `module`-syntax inline footprints with
    no library link and no 3D model.
  - `ComboDiode` is a THT + SMD dual-side pad set, not `D_SOD-123`.
  - There is no RP2040-Zero or USB-C footprint (built-ins: alps, button, choc, chocmini, diode, jstph, jumper,
    mx, oled, omron, pad, promicro, rgb, rotary, scrollwheel, slider, trrs, via).
  - Custom footprints can be supplied as `.js` files in a `footprints/` folder next to `config.yaml`.
  - Ergogen produces **no schematic**.
- **Two routes for the orchestrator** (question Q4 in the report):
  - **Schematic-first + position table (recommended).**
    - Draw §6's schematic in Konnect, which gives ERC, annotation and a real netlist.
    - Update the PCB and move each footprint to its CSV row.
    - Import `pcb.dxf` as Edge.Cuts.
    - Cost: 42+ placement moves per board. These are mechanical, and scripting them is a Konnect call per
      footprint or a batch.
  - **Import Ergogen's PCB as the seed.**
    - Placement and matrix nets arrive free.
    - But the board has no schematic (no ERC, and back-annotation breaks), its footprints are the inline ones
      above, and the module, link circuit and power nets would be drawn by hand on the PCB.
    - Fixing that later costs more MCP time than the placement saves.

## 9. To verify before drawing (no answer needed, just checks)

1. RP2040-Zero pad positions and edge assignment against Waveshare's drawing (§2, §6). **Partly done 2026-11-01:**
   `fw16kb:RP2040-Zero_Castellated` takes its hole positions from Waveshare's own STEP (`RP2040_ZERO_ASM`,
   2022-01-08, shipped in github.com/CountParadox/RP2040-Zero-Kicad, which has no licence and a "DO NOT USE" note on
   its footprint, so nothing else was copied): 9 + 5 + 9 plated Ø0.9 holes at 2.54 mm, side rows 15.24 apart, 1.39 mm
   inside an 18.02 × 23.52 mm board, the 5-hole row at the end opposite the USB-C. The pin *functions* per position
   agree between the tscircuit list above and that repo's symbol + footprint, but Waveshare's pinout drawing itself
   is still unchecked: compare against the module's silkscreen when it arrives, before ordering boards.
2. Pad pin maps of `Diode_SMD:D_SOD-123` (pad 1 = K) and of the AO3401A on `SOT-23` (G/S/D).
3. CPG135001S30 body extent against the diode position (§6), with calipers once parts arrive.
4. The HRO TYPE-C-31-M-12's mouth overhang relative to its footprint origin, so the mouth lands on the PCB edge.

## Sources

- QMK: [split keyboard](https://docs.qmk.fm/features/split_keyboard), [serial driver](https://docs.qmk.fm/drivers/serial),
  [RP2040 `serial_vendor.c`](https://github.com/qmk/qmk_firmware/blob/master/platforms/chibios/drivers/vendor/RP/RP2040/serial_vendor.c)
- RP2040-Zero: [Waveshare wiki](https://www.waveshare.com/wiki/RP2040-Zero),
  [pad list (tscircuit)](https://tscircuit.com/seveibar/rp2040-zero/blob/lib/PinOutCircuit.tsx),
  [specs (mischianti)](https://mischianti.org/waveshare-rp2040-zero-high-resolution-pinout-and-specs/)
- KiCad libraries: [kicad-footprints](https://gitlab.com/kicad/libraries/kicad-footprints) (`Button_Switch_Keyboard`,
  `Connector_USB`)
- Ergogen footprints: built-ins in the nix `ergogen-4.2.1` package; [ceoloide/ergogen-footprints](https://github.com/ceoloide/ergogen-footprints)
  (`switch_choc_v1_v2.js`, CC-BY-NC-SA-4.0)
- USB Type-C Cable and Connector Specification (USB-IF), Rp/Rd/Ra values and source-side attach states (§4.5)
