# The dev environment for the whole repo — the single source of truth for
# every tool: KiCad (the card), FreeCAD (bay keepout and the card envelope),
# poppler (reading Framework's PDF drawings), Python (bay/tools checks).
# Later phases add Vivado-adjacent tools, OCaml/opam and the driver build here.
#
# Konnect (the KiCad MCP server) is not provided here; it runs from
# ~/bin/konnect and drives the kicad-cli from this same pin.
{ pkgs ? import (builtins.fetchTarball {
    # nixpkgs-unstable as of 2026-10-28: KiCad 10.0.7, the version Konnect
    # 0.13 drives — bump only together with Konnect's KiCad.
    url = "https://github.com/NixOS/nixpkgs/archive/58deea654ec5dd0644f31c840a07c29fa0592528.tar.gz";
    sha256 = "1x14d54pa31qgvd655lw6szmhmpxda0nlqcbir587dgrcz0lpmll";
  }) {} }:

pkgs.mkShell {
  packages = with pkgs; [
    kicad
    freecad
    poppler-utils
    python3
  ];
}
