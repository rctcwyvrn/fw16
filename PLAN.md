# fw16 — build plan

Implements the design doc [FW16 Split Keyboard and FPGA Bay Module](https://claude.ai/artifact/AnCbHXtriDXSs3vPVNYoFu)
(rev 11, 2026-10-29) as two independent tracks. Where the doc and the repo's decision records disagree, **the
decision records win** (user, 2026-10-29): the doc's NiteFury → SoM → chip-down ladder and its "19 W on the
low-voltage rails" budget are superseded by [0001](fpga/docs/decisions/0001-tier1-board.md),
[0007](fpga/docs/decisions/0007-card-architecture.md) (branch A, minimal chip-down AU15P spin 1) and
[0008](fpga/docs/decisions/0008-card-power-input.md) (VSYS_GPU primary).

No hardware is in hand yet (2026-10-29), so every track opens with work that needs none — CAD from STEP
files, schematic, simulation, layout templates — while step 0 orders parts.

## Tracks and gates

| Track | Plan | Ends with | Gate |
| --- | --- | --- | --- |
| 0 · Acquire | below | laptop, bay reference parts, keyboard parts on the desk | everything in the order list received |
| K · Split keyboard | [keyboard/PLAN.md](keyboard/PLAN.md) | two wired Choc halves on QMK, printed cases; tray halves tent them | both halves type over USB-C; 20° tent holds while typing |
| T · Belly tray | [keyboard/tray/PLAN.md](keyboard/tray/PLAN.md) | printed tray, then a machined one, latched under the laptop | 30-min load within 3 °C of no-tray; fits a 16" sleeve |
| F · FPGA bay module | [fpga/PLAN.md](fpga/PLAN.md) (paths 1–4, sync S1–S6) | custom AU15P card doing sustained DMA in the bay | 1 h loopback, zero errors, Tj < 85 °C |
| P · FPGA projects | [fpga/projects/PLAN.md](fpga/projects/PLAN.md) | logic analyzer and RISC-V soft CPU in the project slot, then the rest of the doc's list | each project's own acceptance |

How the tracks and the lanes inside them run at once: [PARALLEL.md](PARALLEL.md).

K and F share nothing but the laptop and run in parallel. T needs K's stowed-half dimensions (K step 4) before
its insert is final. P needs F's shell gateware and host crate, and the card in the bay (S5) before any
project leaves simulation.

## 0 · Acquire

| Part | Track | Price | Note |
| --- | --- | --- | --- |
| Framework Laptop 16 + spare Expansion Bay Shell | all | — | the spare shell is cut for the rear slot (F path 1.6) |
| Dual M.2 Adapter + cheap 2230/2280 NVMe | F | $39 + ~$30 | bay reference: `lspci`, IOMMU group (gates 0009), suspend |
| Graphics Module Interposer ×2 | F | $29 ea | one for the reference, one for the card |
| Threaded inserts ×4, SMT standoffs | F | free (Framework forms) | request on order day; lead time unknown |
| AU15P quote (`XCAU15P-2FFVB676E` ×2) | F | TBD | decision 0006 reopens above ~$300/unit |
| Inline USB-C PD meter (POWER-Z KM003C), USB-serial multi-channel K-type thermocouple logger | F, T | ~$80–140 | S6 soak and the tray thermal test share them; user picked 2026-10-31, logger model still to choose |
| 46 Choc v1 switches (42 + spares), 42 hot-swap sockets, diodes, keycaps | K | ~$90 | layout B, 42 keys (keyboard decision 0001) |
| 2× RP2040-Zero | K | ~$10 | module per half (keyboard decision 0001) |
| Calipers (if not owned) | F, T | ~$25 | KEEPOUT v2 and tray fit |

## Order of work (no hardware yet)

1. **This week, no parts:** F path 2 schematic (empty sheets `fpga`, `core_power`, `config`, `pcie_sideband`;
   `POWER.md` part picks), F path 3 Vivado flow first run, K step 1 layout template, T step 1 STEP-based outline.
2. **Laptop arrives:** F path 4.1 bay reference tests on the Dual M.2 + NVMe; F path 1.2 calipers → KEEPOUT v2;
   T step 2 measure the underside and intakes.
3. **Keyboard PCBs back:** K bring-up while F waits on S2/S3.
4. **Card boards back (S4):** F bench bring-up; P starts on the card once S5 passes.

## Repo layout

```
PLAN.md            this file
fpga/              bay module: PLAN.md (Phase 2 paths), bay/, gateware/, host/, driver/, docs/
fpga/projects/     PLAN.md for the doc's personal FPGA projects; one directory per project later
keyboard/          PLAN.md, pcb/ (KiCad), firmware/ (QMK), case/ (Fusion exports, STL)
keyboard/tray/     PLAN.md, cad/ (Fusion exports, STL, DXF)
```

## Doc follow-ups

- The design doc's FPGA bay section, roadmap card "1 · Shell and NiteFury" and the "Bay power and heat" risk
  row still describe the superseded path; update them to match 0001/0007/0008 when convenient.
- `fpga/PLAN.md` still names omnimeter phases (Phase 3 tray, Phase 4 cytometer, Phase 7 CPU checks); rewrite
  those references against this file.
