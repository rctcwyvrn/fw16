# Konnect only finds KiCad's stock libraries (Device:, power:, Resistor_SMD…) when started from `nix-shell`

The nix `kicad` wrapper sets `KICAD10_SYMBOL_DIR` / `KICAD10_FOOTPRINT_DIR` only for KiCad's own binaries;
the global `sym-lib-table` just points at `${KICAD10_TEMPLATE_DIR}`. Konnect (`~/bin/konnect`, launched by
Claude Code) inherits Claude Code's environment, so placing `Device:R` fails with "Library 'Device' not found"
unless Claude Code itself was started inside this repo's `nix-shell`, whose `shell.nix` exports those vars.
`reload_server` does not help — the environment comes from the parent process.
Related: subagents (e.g. kicad-schematic-build-agent) could not call Konnect's deferred schematic tools at all
in this setup; drive schematic builds from the main session.
