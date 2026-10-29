# bay

Expansion Bay work: `KEEPOUT.md` and `POWER.md` (Phase 2 section B), the rear-slot shell modification, and the custom AU15P card KiCad project under `card/` with its `BRINGUP.md`. Community interposer footprints are cloned into `lib/` (git-ignored) and must be pin-checked against the ExpansionBay Electrical README before use.

## FXBeam footprint check (2026-10-28)

Checked by script, pad by pad, against two Framework sources: the pin table in
`ExpansionBay/Electrical/README.md` (pin → signal → column/row) and the Neoconix drawing
`Dual-channel 74P FXBeam, length 27.75mm _Rev 2_` sheet 2, "Recommend DB footprint" (DWG FXBM-X148F027A).
Clones: FrameworkComputer/ExpansionBay `c9b8eef`, XPModder/Framework-ExpansionBay `2ec6368`,
jyancat/ExpansionBay `15efffe`. Framework's README says neither community footprint was tested by them.

Geometry from the drawing (origin at hole E, y up): pads 0.70 × 0.40 mm, column pitch 0.86 mm within a pair,
row pitch 0.60 mm (11 rows A–L, no I); power field on the left, signal field 16.65 mm to its right; columns
numbered 8 → 1 left to right in both fields; signal rows sit 0.25 mm above power rows. Holes: 2 × Ø2.70 NPTH
(interposer screws) with Ø4.50/Ø3.20 SMT rings on the back side, Ø1.50 NPTH × 4 and Ø1.40 NPTH × 2
(alignment). Solder-mask opening ≥ 0.5 × 0.8 mm per pad, i.e. ≥ 0.05 mm mask margin.

| | XPModder `Framework-FX-Beam` | jyancat `Expansion Bay FXBeam` |
| --- | --- | --- |
| Pad numbering | own geometric order (`P-1..74`, `D-1..74`), signal names carried by its symbol | row+column (`PA1`, `SL8`) |
| 148 pad positions | match the drawing | match the drawing |
| Pad → signal | **all 148 match** the README (after name normalisation: `…N`=`…#`, `SDA`=`DAT`) | **PJ6/PJ7/PJ8 are mis-named PJ1/PJ2/PJ3** — as shipped it shorts three GND pads to ID0, Fan1_SPEED and USB+ |
| NPTH diameters | match (drill 1.5 / 1.4 / 2.7; pad sizes 1.6 / 3.2 are mask openings) | match (1.5 / 1.4 / 2.7) |
| SMT rings | present, B.Cu, Ø4.5 pad over the 3.2 hole | **missing** |
| Mask margin | board default (must be ≥ 0.05) | board default (must be ≥ 0.05) |

XPModder's only gap is the mask margin; jyancat's naming bug makes it unsafe. Re-run with `bay/tools/fxbeam_check.py`. Pin-table facts the plan's step-1 list did not have: the power field also carries
**VSYS_GPU ×14** (7–20 V, "up to 10.5 A" on a high-power adapter, 15.2 V min on battery), **PWR_EN** (in, from
the PCH) and **PWR_GOOD** (out, power-sequencing sideband), plus EXT_SSD1/2_RST# and EXT_SSD2_CLK_REQ# for the
second PCIe port; the signal field has TH_OVERT# (PROCHOT#, drive low to throttle the system) and TH_ALERT#.
The "lane crossover" reported by the community was not found in either repo or the README; P/N pad order
does alternate between lanes (e.g. TX0 P below N, TX2 P above N), which is a layout matter, not a swap.

## How the EC powers the bay (research 2026-10-28)

From the FW16 ("lotus") EC source, `FrameworkComputer/EmbeddedController` branch `fwk-lotus-azalea-19573`,
`zephyr/program/framework/lotus/src/{gpu.c,gpu_configuration.c,power_sequence.c,adc.c}`, and
`FrameworkComputer/ExpansionBayConfigurationGenerator`. Not yet verified on hardware.

- **The bay is dead without two things on the card**, 5V_ALW and 3V3_ALW included ("ALW" is a misnomer):
  - ID0 and ID1 must read a valid ADC code (`gpu.c:127-183`): (12,12) = dual-interposer device, ≈2.135 V on
    the EC's 100 k pull-up. Inferred, medium confidence: **330 k to GND on each**, as Input Modules and
    XPModder's design use.
  - An EEPROM descriptor on I2C_*_ALW at 0x50 (or 0x52), magic `32 ac 00 00` + CRC32 + typed blocks
    (PCIE, FAN, VENDOR, GPIO). A blank EEPROM that ACKs falls back to the dual-SSD config. No EEPROM means
    nothing is enabled. XPModder uses a 24LC32.
  - The bay hatch/latch switch must also read closed.
- **The generator's "PCIe accessory" descriptor turns VSYS_GPU on from S3**, 3V/5V from S5, and GPIO0/1_EC high
  from S3. It sets `PCIE_8X1` and PCIE_MUX_SEL high. Write it with `framework_tool` or the EC `gpucfg`
  command. So VSYS_GPU *is* available to a non-GPU card.
- **Suspend:** VSYS stays live in S3/S0ix. The card must drop its own load on PERST# / D3.
- **PWR_EN** is driven by the APU (dGPU BOCO/D3cold) and only watched by the EC. Whether BIOS toggles it
  for an accessory is unknown, so PEX_RST# alone must be enough to gate the link. With PCIE_MUX_SEL high,
  **PWR_GOOD becomes the 2nd CLKREQ# and DP_A_AUX the 2nd REFCLK.**
- **Open:** whether PEX_TX/RX are named from the host or the module side (README marks PEX_TX as a module
  *input*; community thread t/75786 unresolved). Settle it from the GPU board PDFs on ExpansionBay branch
  `gpu_design` before S3. Also open: BIOS bifurcation for the accessory vendor, and Dual M.2 power tree.
  **PEX naming resolved 2026-10-29: host side** — see the next section.

## PEX_TX/RX direction (research 2026-10-29)

**PEX_TX/RX are named from the host side.** PEX_TXn is the laptop's transmitter and enters the card;
PEX_RXn is the card's transmitter. Confidence: medium-high (three independent sources agree, none is a
Framework schematic).

For the fpga sheet and the Path 3 XDC:

| Interposer net | Card end | AC coupling on the card |
| --- | --- | --- |
| PEX_TXn / PEX_TXn# | GTH **RX** (MGTHRXP/N) of lane n | none (the host's TX caps sit on the mainboard) |
| PEX_RXn / PEX_RXn# | GTH **TX** (MGTHTXP/N) of lane n | series cap, placed near the interposer |

Evidence:
- **The GPU board PDFs cannot answer it.** `Graphics Module Reference Design/AMD_7700S_GPU_Board.pdf` and
  `Nvidia_5070_GPU_Board.pdf` (6 sheets each) are mechanical drawings: part list, height zones, outline
  dimensions, FXBeam pad detail (sheet 4, Detail A) and paint lines. They show no components and no net
  names, so AC-cap placement is not visible. `Dual SSD Reference Design/fw_ssd_pcb_ref_design_0.pdf` is
  also mechanical; its sheet 1 has only a pin table.
- **Electrical README I/O column**, which is written from the module's side throughout (VSYS_GPU, Fan*_PWM,
  PWR_EN = I; Fan*_SPEED, PWR_GOOD, PEX_CLK_REQ# = O): PEX_TX0–7 = **I** (signal pins 2–24),
  PEX_RX0–7 = **O** (pins 26–48). The one row that breaks the convention is PEX_REFCLK = O (pins 50/51),
  which must be a typo, since the clock comes from the host.
- **XPModder's IO module** (`XPModder/Framework-ExpansionBay`, `ExpansionBay ExtensionCard
  Module/Framework-ExpansionBay-IO-Module/usb3.kicad_sch`, re-cloned 2026-10-29 at HEAD): PEX_TX0± goes
  straight to the uPD720201's **PERXP/PERXN** (receive, pins 10/11). PEX_RX0± goes through 100 nF caps
  C1/C2 to its transmitter. Traced by wire connectivity, not by eye. Whether that board was built and
  works is unknown.
- **Community thread t/75786 "PCIe data lines":** Filip reports that the M.2 reference board wires interposer
  RX to M.2 RX and TX to TX. M.2 pin names (PETp0 / PERp0) are host-side, so this matches. One community
  report, not from Framework staff.

Consequence: the card puts PCIe Gen4 AC caps (base spec 176–265 nF, i.e. 220 nF 0201/0402) on PEX_RX0–3
only. XPModder's 100 nF suits its Gen2 controller, not Gen4. A TX/RX swap cannot be fixed by lane
polarity inversion, so S3 must keep this mapping.

## Reference PCB vs README (2026-10-28)

`PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.pdf` sheet 2 has its own pin definition table. It agrees with
the Electrical README on every pad except three in the power field: 3PF / 3PG / 2PG are **I2C_CLK_VS /
I2C_DAT_VS / I2C_INT_VS** there, but **EXT_SSD1_RST# / EXT_SSD2_RST# / EXT_SSD2_CLK_REQ#** in the README. The
README is newer (Dual-SSD era), so we follow it; these pads only go to test pads anyway. The same sheet gives
the shell's fan connector: **AFC58-S04FIA-R1 × 2**, pin 1 +12V, 2 FANx_SPEED, 3 FANx_PWM, 4 GND.

## XCAU15P-FFVB676 package facts (2026-10-28)

- **Height:** lidded flip-chip. A (seated, balls included) = 3.12 / 3.31 / 3.52 mm min/nom/max; lid ≤ 22.10 mm
  square on a 27 × 27 mm body; 1.0 mm pitch, 26 × 26. UG575 v1.22 Fig 4-10 (p. 454).
- **Transceivers:** all 12 GTH (Quads 224, 225, 226) sit on the **column 1–7 edge** (the pin-1 side in top
  view): Quad 226 rows F–L, Quad 225 rows M–W, Quad 224 rows Y–AF. UG575 Fig 3-83 (p. 311, top view).
- **PCIe:** a single hard block, PCIE4C_X0Y0 (Gen4 up to x4). Lane-0 quad options per PG213 v1.3 Table 94:
  GTH_Quad_224 (x4), GTH_Quad_225 (x8). REFCLK goes to GTREFCLK0 of the lane-0 quad.

## AFC58-S04FIA-R1 fan connector footprint (research 2026-10-29)

Source: `PCB Reference Design/Datasheets/J200_J201-SP010036Q00_LTCX00A2N00_AFC58-S04FIA-R1.pdf` in the
`gpu_design` clone. Page 1 is the Hefeng customer drawing for the whole AFC58-S series, 0.5 mm pitch,
H = 1.5 mm, FPC/FFC "Easy-on" right-angle SMT, rev A 2020-09-11. Page 2 is the tape and reel. The drawing
is generic, so the CKTS = 4 row of its table applies. Part-number decode (note 4): 04 circuits, **F = lower
contacts**, I = gold flash, A = tape and reel, R1 = natural housing with a black latch. A web search on
2026-10-29 found no LCSC/JLCPCB listing under this number (LCSC itself not queried directly), so plan for
consignment or hand placement.

**Verify before creating the footprint.** The page is a raster image. It could only be read at thumbnail
resolution here, because `pdftoppm`, `pdftocairo` and `pdfimages` needed approval that a headless run
could not get. Rows marked **(low-res)** need a check against a zoomed render.

| Item | Value | Drawing callout |
| --- | --- | --- |
| Signal pads | 4, numbered 1–4 from the "Circuit 1" end | "Recommended P.C.B pattern layout" |
| Pad pitch | 0.50 ± 0.05 mm | pattern, top-left |
| Signal pad width | 0.30 ± 0.05 mm | pattern, top-right |
| Signal pad length | 1.10 ± 0.05 mm **(low-res)** | pattern, right vertical callout |
| Pad 1 → pad 4 centre span | DIM A = 1.50 ± 0.05 mm | pattern "DIM A", table CKTS 4 |
| Fitting-nail pads | 2, mechanical (unnumbered, or "MP") | pattern, both ends |
| Nail pad size | 1.00 ± 0.05 (X) × 1.60 ± 0.05 (Y) mm **(low-res)** | pattern, bottom-left |
| Nail pad inner edge → pad 1 centre (X) | 1.60 ± 0.05 mm **(low-res)**. Working reading: nail centres at X = ±2.85 mm from the connector centre, giving an outer copper span of 6.70 mm. | pattern, bottom "1.60 ± 0.05". The "(0.15)" reference by Circuit 1 suggests the callout ends on the pad centreline. |
| Signal row → nail pads (Y) | 2.00 ± 0.05 mm **(low-res)**. Unclear whether it runs from the signal-pad outer edge to the nail-pad near edge or to the far edge. | pattern, right "2.00 ± 0.05" |
| Housing width | DIM A + 2.56 = 4.06 mm | top view |
| Overall width incl. nails | DIM B = 6.40 ± 0.15 mm | front view, table |
| DIM C / DIM D | 2.60 ± 0.05 / 5.40 ± 0.10 mm | front view, table |
| Depth | 3.83 mm body; 4.45 mm overall; terminal tail 1.10 mm | side view |
| Height | 1.50 mm (datum Z); latch opens to 105° | side view, section A-A |
| Mating FPC | 0.30 ± 0.03 mm thick, 2.50 ± 0.08 mm wide (DIM A + 1.00), conductors 0.30 ± 0.03 mm on 0.50 pitch, ≥ 2.70 mm exposed, ≥ 3.5 mm stiffener | "Recommended FPC/FFC dimension" |

Sanity check: the 6.70 mm working-reading pad span is wider than the 6.40 mm part (nail pads overhanging the
nails), which is how such land patterns usually look. The other reading, 1.60 mm nail-centre to pad-1-centre,
gives a 5.70 mm span, narrower than the part, which is unlikely. Pinout (FW_EXPANSION_BAY_PCB_0807.pdf
sheet 2): 1 +12V_FAN, 2 FANx_SPEED, 3 FANx_PWM, 4 GND. **Open:** which side of the fan FPC has its contacts
exposed. A lower-contact connector needs the conductors facing the board. The UMA fan spec PDFs have no
extractable text.

**Verified 2026-10-29 against a 300 dpi render, and built as `fw16-fpga-module:AFC58-S04FIA-R1`.** The low-res rows hold.
The 2.00 mm callout runs from the signal-pad edge nearest the nails to the far edge of the nail pad. The
top view's 3.10 mm (terminal tip to the nail edge) equals 1.10 + 2.00, and the nail metal sits 1.93–3.10 mm behind
the terminal tip. The pad-1 centre is 1.60 mm from the nail pad's inner edge.

Footprint frame: origin at the signal-pad centre. Pads 1–4 are 0.30 × 1.10 mm at x = −0.75…+0.75, y = 0. The two `MP`
nail pads are 1.00 × 1.60 mm at x = ±2.85, y = +1.75. The terminal tip is at y −0.70, and the housing runs to y +3.75.
The courtyard is ±3.6 × (−0.95…+4.0). The MP pads have no net, because the symbol is a plain `Conn_01x04`. Tie them
to GND in the PCB if the reference board does. The connectors go on the **bottom** side (CAD), so flip them at
placement.
