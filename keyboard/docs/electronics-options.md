# Keyboard electronics — options (K step 2 research, no choice made)

Research for `keyboard/PLAN.md` step 2, written 2026-10-31 by the K-layout/K-elec worker. Nothing here is
decided. The user picks, then the choice goes to `keyboard/docs/decisions/` and is frozen at **KS2**
(MCU, GPIO map, inter-half link). Key counts come from the three layout candidates in
[`../layout/README.md`](../layout/README.md): **A 36, B 42, C 38 keys**.

Prices are rough single-hobbyist USD as of 2026-10 from the sources listed at the end. Check them on order day.

## 1. Controller per half

### What HandyWork actually is (corrects an assumption in the plan)

`keyboard/PLAN.md` step 2 reads as if HandyWork were a split-keyboard template we could fork. It is a
**Framework 16 Input Module**: a deck keyboard in the One Key Module form factor. Its stack is a 4-layer 0.8 mm PCB
glued to a 0.8 mm aluminium PCB, 3.65 mm overall, which matches the module. It is a unibody split
(5×3 + 3 mods + 3 thumbs per side + a nav cluster). It runs an RP2040 with an IS31FL3733B per-key LED driver
on "a recent QMK", branch `axel/handywork`. The repo is `AxelVoitier/keebs` (`Framework/Kicad templates`),
licensed CERN-OHL-W-2.0 / MPL-2.0. Its PCB template has a pre-routed RP2040 and LED driver, plus fanned-out
rows and columns for up to 5 rows × 13 columns.

At 3.65 mm it cannot hold Choc v1 switches (the switch alone is 8 mm tall). Its Framework-specific parts are the
deck connector and the module outline, and our halves stow in the tray rather than the deck, so we use neither.
**What transfers is the routed RP2040 block** (crystal, flash, regulator, USB), drawn for 4 layers at 0.8 mm.

### Options

| | **a. HandyWork-derived chip-down RP2040** | **b. RP2040 module per half (XIAO RP2040 / RP2040-Zero class)** | **c. Bare RP2040, our own layout** |
| --- | --- | --- | --- |
| What gets built | copy HandyWork's RP2040 block into our PCB, drop the LED driver, re-fit to 2 or 4 layers | a castellated module soldered flat on the PCB; the PCB carries only the matrix, sockets, diodes and the link jack | RP2040 + QSPI flash + 12 MHz crystal + 3.3 V LDO + USB-C + ESD, laid out from Raspberry Pi's "minimal design" |
| Parts per half | RP2040 ≈ $0.80, W25Q16/128 ≈ $0.30, crystal ≈ $0.20, LDO ≈ $0.20, USB-C ≈ $0.40, passives ≈ $0.50 → **≈ $2.5** | XIAO RP2040 **≈ $4–5** (21 × 17.5 mm, 11 GPIO) or RP2040-Zero **≈ $4–6** (23.5 × 18 mm, 20 GPIO on edge pads, 29 with bottom pads) | same as a: **≈ $2.5** |
| Assembly | QFN-56 at 0.4 mm pitch: needs PCBA (JLCPCB: setup ≈ $8 + ≈ $3 per extended part) or a stencil and hot air | hand-solderable with an iron; the module is pre-tested | same as a |
| PCB | 4-layer if the routing is reused as-is (≈ $10–20 more per 5 boards at JLCPCB than 2-layer); 2-layer means re-routing | 2-layer, simple; reversible (one board for both halves) is easy | 2-layer is feasible (Raspberry Pi's minimal design is 2-layer) |
| Height | everything flat on the PCB, about 1 mm, plus the USB-C receptacle ≈ 3.2 mm | module ≈ 1 mm PCB + parts; the XIAO's own USB-C ≈ 3.2 mm on top | as a |
| GPIO for our matrices | all 30 available | XIAO's 11 is **exactly enough for B** (6 cols × 4 rows = 10 + 1 half-duplex link) and **one short for full-duplex**; A and C (5 × 4 = 9) fit with full-duplex. The RP2040-Zero has room for anything | all 30 |
| Laptop USB port | our own USB-C receptacle, placed where the case wants it | the module's USB-C, so its position fixes where the cable leaves the case | our own receptacle |
| Risk / effort | most Konnect work (the MCP lane is the scarce resource); the copied block is unverified until boards arrive | least PCB work; the most proven path for hobby splits; RP2040-Zero bottom pads need reflow if used | most design work; nothing to copy from, only Raspberry Pi's reference |
| What it forecloses | — | some freedom in where the USB port goes; ≈ 1.5 mm extra height where the module sits | — |

The "≈ 130 × 100 mm" target and the templates both reserve a 22 × 40 mm controller zone at the inner column,
which holds any of the three.

## 2. Inter-half link

QMK's split serial driver on the RP2040 uses PIO. It runs half-duplex on 1 data wire or full-duplex on 2, on any
GPIO, with no pull-up needed. TRRS (4 conductors: VBUS, GND, 1–2 data) and USB-C both carry enough wires.

| | **TRRS (3.5 mm)** | **USB-C (receptacle on each half, C-to-C cable)** |
| --- | --- | --- |
| Parts | PJ-320A-class jack ≈ $0.20 per half; TRRS cable ≈ $5–10 | 16-pin USB-C receptacle ≈ $0.40 per half (+ ESD array ≈ $0.20); C-to-C cable ≈ $5–10 |
| Hot-plug | **unsafe**: the plug's tip and rings slide past each other, briefly shorting VBUS (5 V) to the data pin, and RP2040 GPIOs are not 5 V tolerant. Rule: never plug or unplug while powered. | no shorting while sliding in; safe to hot-plug |
| Wrong cable / wrong port | a TRRS jack can't be confused with the laptop port | the user can plug the laptop into the link port, or the link cable into the laptop. Needs mitigation: 5.1 k CC pull-downs only on the true USB port, the link carrying UART on SBU or D± with series resistors, and a label |
| Height on the PCB | jack body ≈ 5–6 mm tall: the tallest part on the board, sitting about level with the switch tops | receptacle ≈ 3.2 mm |
| Stowing in the tray | straight plug ≈ 15 mm proud of the case, right-angle ≈ 10 mm. Unplug and coil it in the spare depth (≥ 157 mm free in every candidate) | right-angle C plug ≈ 8 mm; same coiling. A short C-to-C cable can double as the laptop cable |
| Later wireless (nRF52840) | jack becomes dead weight | USB-C stays useful for charging each half |

The plan's note "USB-C if the cable must also nest in the tray" is a weak argument either way. All three
layouts leave so much tray depth spare that a coiled cable fits whichever plug it has. The real differences
are hot-plug safety and port confusion.

## 3. Hot-swap sockets and diodes

| Part | Choice | Price | Note |
| --- | --- | --- | --- |
| Hot-swap socket | **Kailh CPG135001S30** (Choc v1) | LCSC ≈ $0.07–0.15; retail ≈ $0.25–0.35 | The only common Choc v1 socket. It sits under the PCB: the body is ≈ 1.8 mm deep per common footprints, and one retail listing gives 3.0 mm overall. **Measure it with calipers before freezing the case floor.** |
| Alternative | Mill-Max 0305/7305 pin receptacles | ≈ $0.15–0.30 per pin, 2 per switch | Pressed into plated holes, nothing hangs below the PCB, so they save ≈ 1.8 mm of stack. Fiddlier to install. |
| Diode (SMD) | **1N4148W, SOD-123** | ≈ $0.01 at LCSC | Hand-solderable; JLCPCB basic part, so no extended fee if assembled. |
| Diode (smaller) | 1N4148WS, SOD-323 | ≈ $0.01 | Smaller and harder to hand-solder. |
| Diode (THT) | 1N4148 DO-35 | ≈ $0.02 | Leads poke through the PCB underside, which conflicts with the socket side. Avoid. |

## 4. Stack height vs the 10–12 mm stowed budget

**Likely problem:** a hot-swap Choc v1 half with a solid case floor comes out at about 13 mm, which is over the
10–12 mm stowed target. That target is also what the 12 mm-deep tray assumes. The numbers below are estimates;
Kailh lists the switch only as 15 × 15 × 8 mm.

| Layer | Hot-swap, solid floor | Hot-swap, floor pocketed under sockets | Mill-Max, solid floor |
| --- | --- | --- | --- |
| Case floor | 1.2 | 1.2 (0 under sockets) | 1.2 |
| Socket body below PCB | 1.8 | (in floor pockets) | 0 |
| PCB | 1.2 (or 1.6) | 1.2 | 1.2 |
| Switch + MBK cap above PCB top (est., verify) | 8.5–9 | 8.5–9 | 8.5–9 |
| **Total, legs folded flat inside the outline** | **≈ 12.7–13.2** | **≈ 10.9–11.4** | **≈ 10.9–11.4** |

For comparison, the Sofle Choc build quotes 15 mm from desk to keycap top, with feet and a full bottom plate.
Ways to get under 12 mm: pocket the floor under each socket (the floor becomes a ribbed grid), use Mill-Max
receptacles, a 1.0 mm or 0.8 mm PCB, or solder the switches directly (gives up hot-swap). Legs must fold
**into** the floor or flush to it, not add thickness. This is K-case's problem (KS3), but it constrains the
choices here: socket type and PCB thickness.

## 5. BOM per candidate

Both halves, rough USD. Switches and caps include about 10 % spares, rounded up to the pack size. Columns: (b) is
the module per half, (a/c) is chip-down. Choc v1 switches cost ≈ $0.55 each in AliExpress bulk and ≈ $1.20 each
in retail 10-packs (Adafruit $11.95 / 10). MBK blank caps cost ≈ $0.45 (AliExpress) to ≈ $0.90 (splitkb €0.82+)
each.

| Line | A (36 keys) | B (42) | C (38) | Source |
| --- | --- | --- | --- | --- |
| Choc v1 switches (40 / 50 / 40 bought) | $22–48 | $28–60 | $22–48 | AliExpress (Kailh store), Adafruit, splitkb, lowprokb |
| MBK caps (40 / 50 / 40) | $18–36 | $23–45 | $18–36 | AliExpress, splitkb |
| Hot-swap sockets (40 / 50 / 40) | $3–14 | $4–18 | $3–14 | LCSC C5333465, splitkb, Adafruit 5118 |
| Diodes 1N4148W (100 pcs) | $1 | $1 | $1 | LCSC |
| Controller ×2 | (b) $8–12 · (a/c) $5 + PCBA | same | same | Seeed, Waveshare, LCSC |
| Link: TRRS (2 jacks + cable) / USB-C (2 receptacles + ESD + cable) | $6–11 / $6–12 | same | same | LCSC, any retailer |
| Laptop cable (USB-C to C, short, right-angle) | $5–10 | same | same | — |
| PCB, 5 pcs, reversible 2-layer (b) / 4-layer (a) | $5–10 / $15–30 + shipping ≈ $15–25 | same | same | JLCPCB |
| PCBA for chip-down (a/c) | ≈ $25–40 (setup + extended parts + 2 boards) | same | same | JLCPCB |
| **Total (b, module)** | **≈ $85–170** | **≈ $95–195** | **≈ $85–170** | |
| **Total (a/c, chip-down + PCBA)** | **≈ $115–220** | **≈ $125–245** | **≈ $115–220** | |

The PLAN.md acquire row ("40–44 Choc switches … ≈ $90") holds for A and C at the low end. B needs 42 + spares.

## Questions this doc leaves for the user (KS2)

1. Controller: (a) HandyWork-derived chip-down, (b) module per half, or (c) bare RP2040?
2. Link: TRRS or USB-C?
3. Socket type: Kailh hot-swap with a pocketed floor, Mill-Max, or soldered switches? This choice sets
   whether 12 mm stowed is reachable.

## Sources

- HandyWork: [Framework Community thread](https://community.frame.work/t/handywork-an-ergo-keyboard-for-the-framework-16/80180),
  [AxelVoitier/keebs](https://github.com/AxelVoitier/keebs), [Framework/ folder](https://github.com/AxelVoitier/keebs/tree/main/Framework)
- Framework input-module reference (minimal RP2040 module in KiCad): [FrameworkComputer/InputModules](https://github.com/frameworkComputer/inputmodules)
- QMK: [split keyboard](https://docs.qmk.fm/features/split_keyboard), [serial driver](https://docs.qmk.fm/drivers/serial),
  [RP2040 platform](https://docs.qmk.fm/platformdev_rp2040)
- XIAO RP2040: [Seeed wiki](https://wiki.seeedstudio.com/XIAO-RP2040/), [The Pi Hut](https://thepihut.com/products/seeed-xiao-rp2040);
  RP2040-Zero: [Core Electronics](https://core-electronics.com.au/waveshare-rp2040-zero-dev-board-16-mb.html)
- XIAO GPIO in splits: [kbd.news](https://kbd.news/Seeeduino-Xiao-RP2040-1389.html)
- Sockets: [LCSC CPG135001S30](https://fat.lcsc.com/product-detail/image/CPG135001S30_C5333465.html),
  [Adafruit 5118](https://www.adafruit.com/products/5118), [splitkb Kailh sockets](https://splitkb.com/products/kailh-hotswap-sockets),
  [splitkb Mill-Max](https://splitkb.com/products/mill-max-hot-swap-sockets)
- Switches and caps: [Adafruit 5113](https://www.adafruit.com/product/5113), [splitkb switches](https://splitkb.com/collections/switches),
  [splitkb MBK blank](https://splitkb.com/products/blank-mbk-choc-low-profile-keycaps), [deskthority PG1350](https://deskthority.net/wiki/Kailh_PG1350_series)
- Stack reference: [Sofle Choc build guide](https://josefadamcik.github.io/SofleKeyboard/build_guide_choc.html)
