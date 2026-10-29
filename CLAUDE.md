# fw16 — project notes for Claude

Two independent Framework Laptop 16 builds, per the doc
[FW16 Split Keyboard and FPGA Bay Module](https://claude.ai/artifact/AnCbHXtriDXSs3vPVNYoFu):
- `fpga/` — the FPGA expansion bay module: card KiCad project, Hardcaml shell gateware, Rust VFIO host, docs.
  Imported with history from `~/code/omnimeter` (rev 17b8cf9, 2026-10-29); omnimeter keeps its own copy and
  is not kept in sync. `fpga/` is its own dune root (`fpga/dune-project`); run dune and `gateware/...` paths from `fpga/`.
- `keyboard/` — split Choc keyboard and its own belly tray (not omnimeter's Phase 3 tray).

- **No sidecar docs in this repo**, mirroring omnimeter; do not add `.claude/sidecar.json` or sidecars. Files
  still keep one purpose each.
- Tools come from `flake.nix` (`nix develop`); `shell.nix` is a shim onto it so `nix-shell` also works. Add tools to the flake only.
- KiCad files change only through Konnect MCP tools.
- VCS is jj: use `jj` for every VCS operation it can do; fall back to `git` only for what jj cannot.
  Remote: `git@github.com:rctcwyvrn/fw16.git`.
