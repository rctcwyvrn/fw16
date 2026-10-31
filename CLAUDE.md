# fw16 — project notes for Claude

Two independent Framework Laptop 16 builds, per the doc
[FW16 Split Keyboard and FPGA Bay Module](https://claude.ai/artifact/AnCbHXtriDXSs3vPVNYoFu):
- `fpga/` — the FPGA expansion bay module: card KiCad project, Hardcaml shell gateware, Rust VFIO host, docs.
  Imported with history from `~/code/omnimeter` (rev 17b8cf9, 2026-10-29); omnimeter keeps its own copy and
  is not kept in sync. `fpga/` is its own dune root (`fpga/dune-project`); run dune and `gateware/...` paths from `fpga/`.
- `keyboard/` — split Choc keyboard and its own belly tray (`keyboard/tray/`, not omnimeter's Phase 3 tray).

## Plans
- `PLAN.md` — tracks (0 acquire, K keyboard, T tray, F FPGA bay, P FPGA projects), gates, order list.
  Per-track plans: `keyboard/PLAN.md`, `keyboard/tray/PLAN.md`, `fpga/PLAN.md` (Phase 2 paths 1–4, sync
  S1–S6), `fpga/projects/PLAN.md`.
- `PARALLEL.md` — lanes, what each lane owns, shared-resource rules, sync points, rounds.
- **Where the design doc and `fpga/docs/decisions/` disagree, the decision records win** (user, 2026-10-29):
  no NiteFury/SoM tier, chip-down AU15P branch A (0001, 0007), VSYS_GPU power (0008).

## Conventions
- **No sidecar docs in this repo**, mirroring omnimeter; do not add `.claude/sidecar.json` or sidecars. Files
  still keep one purpose each.
- Tools come from `flake.nix` (`nix develop`); `shell.nix` is a shim onto it so `nix-shell` also works. Add tools
  to the flake only; a flake edit touches only the tool list or `nixosModules` (user, 2026-10-31: the driver module
  is exported as `nixosModules.fw16-fpga`) and is committed alone.
- KiCad files change only through Konnect MCP tools; Fusion work goes through the fusion360 MCP. Both run as
  Windows processes and reach only the interactive session, never subagents or headless workers.
- Handoffs are repo markdown: `.claude/handoff-<lane>.md` (read-first list, decisions made, state, next steps,
  judgment calls), never a Claude Doc.

## VCS
- jj: use `jj` for every VCS operation it can do; fall back to `git` only for what jj cannot.
  Remote: `git@github.com:rctcwyvrn/fw16.git`.
- **Commit and push without asking.** Several sessions share this checkout (`PARALLEL.md`), so commit each
  lane's work as soon as it stands: small commits of only that lane's files (`jj commit <paths> -m …`, run from
  the repo root — paths are cwd-relative), `jj bookmark set main -r <change id>`, `jj git push`. `jj git fetch`
  before starting new work.
- Ignore build outputs (`target/`, `_build/`) before the first build: jj snapshots untracked files and refuses
  files over 1 MiB, which blocks every session.

## Workers (see `PARALLEL.md`)
- Headless workers run as `claude -p` under `systemd-run --user --unit=fw16-<lane>` in the dev shell, launched by
  `_build/agents/launch.sh` (git-ignored); `setsid`/`nohup` children die when `wsl.exe` exits.
- Workers cannot write under `.claude/`: they put log entries, lessons and questions in
  `_build/agents/<lane>.report.md`, and the orchestrator copies them into `.claude/` and commits.
- Workers stop at a stated checkpoint and never ask mid-run. Workers must not spawn subagents (a headless
  run ends with its turn and kills them).
- Max 4 workers at once: the WSL account has its own API limit. A dead worker: check
  `tail -c 600 _build/agents/<lane>.log` for `api_error_status` before assuming a crash.
- One dune-running worker at a time in `fpga/` (shared `_build/` lock); one Vivado run at a time.
