# fw16 — parallel work plan

How the work in [PLAN.md](PLAN.md) splits into lanes that run at once, what each lane may touch, and where they
meet. Plan as of 2026-10-29, no hardware in hand. The execution model is omnimeter's Phase 2 setup: one
interactive **orchestrator** session plus headless **workers**, with the user doing the physical work.

## Who runs what

| Runner | What it can do | What it cannot |
| --- | --- | --- |
| **O · orchestrator** (interactive Claude session, Windows-side MCPs) | Konnect (KiCad), Fusion 360, launching and reviewing workers, asking the user, writing `.claude/` | run long jobs without blocking the conversation |
| **W · workers** (`claude -p` under `systemd-run --user`, in the dev shell) | code, scripts, STEP analysis via `bay-python`, Vivado from WSL, research, docs | Konnect or Fusion (MCP tools don't reach subagents or headless runs); write under `.claude/` (report file instead); ask questions mid-run |
| **U · user** | order parts, print, measure with calipers, type on templates, pick parts, approve checkpoints | — |

The **MCP lane is single-threaded**: one KiCad, one Fusion, one orchestrator. Every KiCad and Fusion edit for the
card, the keyboard and the tray queues through it, so it is the critical resource. Everything that can be done
as text, scripts or research goes to a worker so the MCP lane only does what only it can.

## Lanes

| Lane | Track | Runner | Owns (may commit) | Reads |
| --- | --- | --- | --- | --- |
| F1-cad | FPGA path 1 | W scripts, O Fusion | `fpga/bay/cad/`, `fpga/bay/tools/`, `fpga/bay/KEEPOUT.md` | STEP files |
| F2-pcb | FPGA path 2 | O Konnect, W research | `fpga/bay/card/`, `fpga/bay/POWER.md`, `fpga/bay/README.md` | KEEPOUT, regmap |
| F3-gw | FPGA path 3 | W | `fpga/gateware/`, `fpga/host/include/` (generated) | pin table (S3) |
| F4-sw | FPGA path 4 | W | `fpga/host/` (except `include/`), `fpga/driver/`, `fpga/benchmarks/` | regmap header |
| P-proj | FPGA projects | W | `fpga/projects/` | shell interface, `fw16-fpga` crate |
| K-layout | keyboard 1 | W, U types on templates | `keyboard/layout/` | tray inner area |
| K-elec | keyboard 2–3 | W research, O Konnect | `keyboard/pcb/`, `keyboard/docs/` | layout |
| K-case | keyboard 4 | O Fusion, U prints | `keyboard/case/` | PCB outline |
| K-fw | keyboard 5 | W | `keyboard/firmware/` | matrix pinout |
| T-tray | tray | W STEP scripts, O Fusion, U prints | `keyboard/tray/` | laptop STEP, keyboard envelope |
| X-shared | all | one lane at a time | `flake.nix`/`flake.lock` (committed alone), `PLAN.md`, `PARALLEL.md` | — |

Lanes commit only their own paths, in small commits pushed right away. Run `jj git fetch` before starting work and
run jj from the repo root (omnimeter lesson `jj-shared-checkout-gotchas`).

## Shared resources that serialize work

| Resource | Contended by | Rule |
| --- | --- | --- |
| MCP lane (Konnect + Fusion) | F2 schematic, F1 envelope/heatsink, K-elec PCB, K-case, T-tray | priority: F2 until S3, then F1 for S2 heatsink, then K/T; one tool's work batch at a time |
| dune `_build/` lock (`fpga/` dune root) | F3, P-proj, F4's golden-file tests | at most one dune-running worker at a time; P-proj gateware rides in F3's worker until the shell is frozen |
| Windows Vivado install + build dir | F3 synth/sim, P-proj bitstreams | one Vivado run at a time; separate build dirs per lane |
| `flake.nix` | any lane adding a tool | edit only the tool list, commit it alone, re-enter the shell |
| regmap ABI | F3 (owner) → F4, P-proj | F3 lands, the others `jj git fetch` and re-run tests before building on top |
| WSL account API limit | all workers | ≤ 4 workers at once; check `api_error_status` in a log before assuming a crash |
| user review | every checkpoint | workers stop at a stated checkpoint; O batches questions per round |
| 3D printer | K-case, T-tray, F1 dummy card | user schedules; small test parts before full-size tray tiles |
| the laptop (once here) | F4 bay tests, F1 calipers, T measure, K bring-up | F4.1 IOMMU-group test first (it gates decision 0009) |

## Sync points

FPGA keeps `fpga/PLAN.md`'s **S1–S6**; S1 is passed (branch A, KEEPOUT v1). New ones for the other tracks:

| Sync | What crosses | From → to |
| --- | --- | --- |
| **KS1 — layout frozen** | key count, positions, outline per half | K-layout → K-elec, K-case, T-tray insert, step-0 order |
| **KS2 — matrix frozen** | MCU, GPIO map, inter-half link | K-elec → K-fw |
| **KS3 — stowed envelope** | per-half footprint and thickness with legs folded | K-case → T-tray step 3 |
| **KS4 — keyboard PCBs back** | boards to bring up | fab → K-fw, K-case fit check |
| **TS1 — underside from CAD** | bottom-cover outline, feet, intakes | T-tray (W) → T-tray (O Fusion) |
| **X1 — laptop arrives** | real hardware for all tracks | U → F4.1, F1.2, T step 2, K bring-up |
| **SP — shell slot interface** | project-slot ports + reload flow | F3/F4 → P-proj |

## Rounds

A round is one batch of workers launched together plus the MCP-lane work the orchestrator does meanwhile. Each
worker stops at its checkpoint; the round ends when all have reported and the user has answered the batch.

### Round 0 — set-up (orchestrator only)
- Port the omnimeter worker lessons into `.claude/lessons/` (headless workers can't write `.claude/`,
  `systemd-run` launch, separate API limit, jj shared-checkout gotchas, handoffs are repo markdown).
- Add the "several sessions commit aggressively" rule to `CLAUDE.md`.
- Write `.claude/handoff-<lane>.md` per lane and `_build/agents/launch.sh` (git-ignored) with fw16 paths and
  `fw16-*` unit names.

### Round 1 — now, no hardware

| Lane | Work | Checkpoint |
| --- | --- | --- |
| O · F2-pcb | `core_power` sheet once the user picks parts from `POWER.md`; then `pcie_sideband`, `config` | ERC clean per sheet |
| W · F3-gw | re-run the Vivado synth + BFM sim after the ID rename; alive-bitstream design with a placeholder XDC; timestamp/PPS | synth report + BFM pass |
| W · F4-sw | `driver/` NixOS module; bay reference-test scripts (`lspci`, IOMMU group, ACS, suspend); S6 measurement tooling | VM/mock tests pass |
| W · K-layout + K-elec research | Ergogen (or KLE) layout configs with printable 1:1 templates; HandyWork-fork vs RP2040-module comparison; BOM with sources | templates printed (U) |
| W · T-tray | raycast the FW16 underside from `Framework_16_all_UMA_asm_0408` with `bay-python`: outline, feet, intake areas → `keyboard/tray/KEEPOUT.md` v1 | TS1 |
| U | order step-0 parts; pick core_power parts; type on the paper templates | KS1 decision |

### Round 2 — after KS1 and the `fpga` sheet

| Lane | Work | Checkpoint |
| --- | --- | --- |
| O · F2-pcb | `fpga` sheet → pin table → **S3**; board setup | S3 |
| O · F1-cad (Fusion) | card envelope, heatsink, outline DXF → **S2** | S2 |
| W · F3-gw | XDC generated from the S3 pin table; project-slot interface (**SP**) | synth with real pins |
| W · F4-sw | reload flow (remove → reprogram → rescan); `sim` transport | mock + sim tests |
| W · K-fw | QMK split skeleton + keymap against the KS2 matrix | `qmk compile` passes |
| O · K-elec | keyboard PCB (Konnect, or Ergogen output — see open questions) | DRC clean, KS2 |
| W · P-proj | logic-analyzer capture core in Hardcaml sim | cyclesim tests |

### Round 3 — fabs out, laptop in
- F2 layout and fab order (path 2.5–2.6); keyboard PCB fab order.
- O · K-case and T-tray in Fusion (cases + legs → KS3; tray v1 around KS3); U prints.
- X1: F4.1 bay reference tests and IOMMU group; F1.2 calipers → KEEPOUT v2; T step 2.

### Round 4 — boards back
- KS4: keyboard bring-up (U solders, K-fw flashes).
- S4–S6: card bench bring-up, PCIe in the bay, thermal soak; T step 6 thermal test shares the logger after S6.
- P-proj leaves simulation after S5.

## Critical path

Card schematic (F2, MCP lane) → S3 → layout → fab (~2–3 months total, branch A) → S4–S6. Everything else
has slack against it, so the MCP lane gives F2 priority until S3. The keyboard is the quickest usable result
(KS1 → PCB fab ≈ 2–3 weeks) and fills MCP-lane gaps while F2 waits on part picks.
