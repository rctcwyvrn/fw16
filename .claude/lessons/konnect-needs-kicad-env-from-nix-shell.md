# Konnect only finds KiCad's stock libraries (Device:, power:, Resistor_SMD…) when started from `nix-shell`

The nix `kicad` wrapper sets `KICAD10_SYMBOL_DIR` / `KICAD10_FOOTPRINT_DIR` only for KiCad's own binaries;
the global `sym-lib-table` just points at `${KICAD10_TEMPLATE_DIR}`. Konnect (`~/bin/konnect`, launched by
Claude Code) inherits Claude Code's environment, so placing `Device:R` fails with "Library 'Device' not found"
unless Claude Code itself was started inside this repo's `nix-shell`, whose `shell.nix` exports those vars.
`reload_server` does not help — the environment comes from the parent process.
Related: subagents (e.g. kicad-schematic-build-agent) could not call Konnect's deferred schematic tools at all
in this setup; drive schematic builds from the main session.

**Scope (2026-10-29):** this applies to the Linux Konnect inside WSL. The Windows build
(`\wsl.localhost\NixOS\home\lily\code\momentum\pcb\konnect.exe`, registered for omnimeter at local scope in
`C:\Users\17786\.claude.json`) uses the Windows KiCad 10.0.7 install and resolves `Device:`/`power:`/stock
footprints with no extra env. Run KiCad work from the Windows Claude Code session.
