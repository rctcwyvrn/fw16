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

## Reference PCB vs README (2026-10-28)

`PCB Reference Design/FW_EXPANSION_BAY_PCB_0807.pdf` sheet 2 has its own pin definition table. It agrees with
the Electrical README on every pad except three in the power field: 3PF / 3PG / 2PG are **I2C_CLK_VS /
I2C_DAT_VS / I2C_INT_VS** there, but **EXT_SSD1_RST# / EXT_SSD2_RST# / EXT_SSD2_CLK_REQ#** in the README. The
README is newer (Dual-SSD era), so we follow it; these pads only go to test pads anyway. The same sheet gives
the shell's fan connector: **AFC58-S04FIA-R1 × 2**, pin 1 +12V, 2 FANx_SPEED, 3 FANx_PWM, 4 GND.
