# Handoff: lane F3-gw (FPGA gateware) — written 2026-10-31

Continue `fpga/PLAN.md` **Path 3** in fw16. Read first: `CLAUDE.md`, `PARALLEL.md` (lane F3-gw, shared
resources), `fpga/PLAN.md` §Path 3, `fpga/gateware/README.md`, `fpga/gateware/regmap/map_v0.ml` and `REGMAP.md`,
`fpga/docs/decisions/0010-card-pin-plan-and-sheets.md` (pins, PPS input), `0011` (power estimate), and the
lessons `xdma-example-tb-hierarchical-refs`, `hardcaml-comes-from-oxcaml-nix`,
`hardcaml-comparison-binds-tighter-than-and`, `ppx-expect-preview-source-tree-root`,
`vivado-here-cannot-answer-au-plus-pinouts`, `jj-shared-checkout-gotchas`.

## Decisions already made (don't re-ask)
- Chip-down XCAU15P-2FFVB676E, PCIe Gen4 x4 on GTH Quad 225, XDMA 4.2, BAR0 = shell AXI-Lite, ID 0x46504741
  ("FPGA"), subsystem 0x10EE/0x4650 (0007, 0010).
- **One pin table:** `fpga/bay/card/fpga/pinplan.csv` (owned by F2/the PCB). `gateware/vivado/pins/pins_xdc.exe`
  turns it into the committed `gateware/vivado/fw16_fpga_pins.xdc`; `dune runtest` fails when that is stale.
  Never hand-edit the pin XDC.
- Top-level card I/O (`fw16_fpga_top.v`) exists with safe defaults: `clk100m` (100 MHz LVCMOS33 on AC13),
  heartbeat on `led_hb`, `led_link = user_lnk_up`, `th_alert`/`th_overt` driven 0, `sense_scl/sda` and `clkreq_n`
  released (z), `sense_alert_n` and `pps_in` inputs unused. All bank 84, LVCMOS33.
- The TS register block is already in the regmap (0x4000: `TS_LATCH` pulse, `TS_NOW_LO/HI`, `TS_PPS_LO/HI`,
  `TS_PPS_COUNT`; `_LO` read latches `_HI`) and the IRQ block has a `PPS` source bit. `shell_top.ml` stubs them:
  `ts_ro` wires read 0 and `pps = gnd`.
- Vivado: Windows 2025.2, driven from WSL through `gateware/vivado/run.sh project|synth|power|sim|all`; build dir
  `C:\Users\17786\fw16-fpga-build\vivado`, outside the repo.

## State (2026-10-31)
- `dune build`, `dune runtest`, `dune build @regmap` pass from `fpga/`.
- Synth with card pins: setup WNS +0.728 ns. Placement DRC clean (RTSTAT-13 only). Power 2.95 W.
- BFM sim PASS: link up 187.5 µs, BAR0 = 0x46504741 (needs the shims in `sim_bfm.tcl`).

## Next steps (this round)
1. **Timestamp unit (Path 3 step 4).** 64-bit free-running counter on the shell clock; `TS_LATCH` copies it into
   `TS_NOW`; a PPS rising edge captures `TS_PPS` and increments `TS_PPS_COUNT` and raises the IRQ `PPS` bit.
   `pps_in` is asynchronous (U.FL, 33 Ω + ESD, LVCMOS33): two-flop synchroniser and edge detect in the top or the
   shell (say which and why). Wire `pps_in` from `fw16_fpga_top.v` into the shell. Hardcaml cyclesim tests with a
   PPS stimulus in `gateware/sim/`; keep the expect output reviewable.
2. Regenerate `host/include/fw16_fpga_regs.h` / `REGMAP.md` only if the map changes (`dune build @regmap
   --auto-promote`). It should not need to; if it does, say exactly why in the report: F4 consumes it.
3. Re-run Vivado `all` (project, synth, power, sim) and record WNS, utilisation and the BFM result.
4. **Project-slot interface (sync point SP): proposal only, no code.** Write `fpga/docs/project-slot-proposal.md`:
   AXI-Stream in/out + an AXI-Lite window per `fpga/projects/PLAN.md` "Shell prerequisites" 4. Cover the address
   window, clock/reset domains, stream widths, how the slot sees the timestamp, and what changes between the
   rescan-reload route and partial reconfiguration. Give options with tradeoffs; the user picks.

Deliberately left out: the alive bitstream (step 6), implementation/bitstream, project gateware.

## Rules for this lane
- You are the **only dune-running worker** and the **only Vivado user** this round.
- Commit only `fpga/gateware/`, `fpga/host/include/` (generated) and `fpga/docs/project-slot-proposal.md`.

## Judgment calls to surface in the report
- Where the PPS synchroniser lives; whether `TS_PPS_COUNT` saturates or wraps; reset value of the counter.

---

# Round 2 (written 2026-10-31)

## State after round 1
- Timestamp unit landed (bd02e15c); `fpga/docs/project-slot-proposal.md` (54923514) is decided in
  **`fpga/docs/decisions/0013-project-slot-interface.md`**. Read 0013 first: it differs from the proposal's
  recommendation on the register space (**bypass BAR**, not a BAR0 window).
- **PCIe lanes swapped** on the card (0010 amendment, lesson `xdma-gt-lane-order-is-reversed-on-quad-225`): lane n is
  on Quad 225 channel 3−n. `power.tcl` now fails if any port is off its pin-plan ball. Keep that check green.
- The orchestrator's Vivado rerun: 0 critical warnings, WNS +0.728 ns, 2.94 W, BFM PASS.
- You may now run `fpga/gateware/vivado/run.sh <step>` yourself (exactly that form, from `fpga/`'s parent or with the
  repo path; ~35 min for `sim`). One Vivado run at a time.

## Next steps (round 2)
1. **Regmap v0.3** in `map_v0.ml` per 0013: `RING_CTL.SRC_SEL` (2 bits, `LOOPBACK` stays bit 0), IRQ bit 3 `SLOT`,
   a `SLOT` block at 0x6000 with `SLOT_CTL.RESET` (and `DECOUPLE` reserved), the stale TS/IRQ-PPS doc strings fixed,
   VERSION bump. Regenerate with `dune build @regmap --auto-promote`. Commit the regmap + generated header **alone and
   first** so F4 can pick it up, with "regmap v0.3" in the message.
2. **Slot boundary in the shell:** a `Slot` interface (`axi_aclk`, `slot_clk`, `slot_clear`, 256-bit s_axis/m_axis,
   `ts_now`, `pps_edge`, `slot_irq`, the AXI4 bypass port) behind one `keep_hierarchy` boundary, with a default
   project (e.g. a stream echo + `SLOT_ID`/`SLOT_VERSION` at bypass offsets 0x0/0x4). Cyclesim tests through the ring
   with `SRC_SEL` = 2.
3. **XDMA bypass BAR:** enable the AXI-MM bypass master on BAR2 in `xdma_0.tcl` and wire it through
   `fw16_fpga_top.v` to the slot. Run `run.sh all`, and in the BFM test also read `SLOT_ID` through BAR2.
4. Record the results in `fpga/docs/decisions/0013` (a Consequences note, numbers and commit ids).

Checkpoint: stop after step 4. Out of scope: the CDC library for `slot_clk` (say what it needs), DFX, project gateware.
Commit only `fpga/gateware/`, `fpga/host/include/` and 0013.

---

# Round 3 (written 2026-11-01)

## State after round 2
- Regmap v0.3 (c9949a39), slot boundary + `Slot_echo` (2ed87fcc), and the bypass BAR (01c6a03d, finished by the
  orchestrator) are on main. Vivado `all`: 0 critical warnings, WNS +0.728 ns, BFM PASS including BAR2
  `SLOT_ID` "ECHO". See 0013's "Verified" note.
- **Your round-2 Vivado run died** because it ran in the background and the headless turn ended. **Run Vivado in the
  foreground** with a timeout (`timeout 3600 fpga/gateware/vivado/run.sh all`); never as a background task.
- Lesson: `sim_bfm.tcl` escapes `&`/`\` before its `regsub` splice (`xdma-example-tb-hierarchical-refs`).

## Next steps (round 3)
1. **`SLOT_CTL.RESET` gates the slot's streams** (0013, user 2026-11-01): AND `s_axis.tvalid` into the slot and the
   slot's `tready` back to the shell with `~slot_clear`. Cyclesim test: with RESET held, nothing moves in either
   direction and H2C is back-pressured; releasing RESET resumes without loss.
2. Keep `dune build`, `dune runtest`, `dune build @regmap` green; the regmap should not change.
3. `timeout 3600 fpga/gateware/vivado/run.sh all` in the foreground; record WNS and the BFM result in 0013.

Checkpoint: stop after step 3. Commit only `fpga/gateware/` and 0013.
