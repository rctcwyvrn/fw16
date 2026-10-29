# The dev environment for the whole repo — the single source of truth for
# every tool: KiCad (the card), FreeCAD (bay keepout and the card envelope),
# poppler (reading Framework's PDF drawings), Python (bay/tools checks), and
# OxCaml + Hardcaml (gateware). Enter with `nix develop`; `shell.nix` is a
# shim onto this flake so `nix-shell` keeps working.
#
# Hardcaml is not in nixpkgs; it comes from oxcaml-nix, which packages the
# Jane Street 0.18-preview set on the OxCaml compiler. OCaml *libraries* the
# gateware needs are listed here because oxcaml-nix is their only source;
# there is no opam switch.
#
# Vivado is not provided by nix: it is the user's install under
# $OMNI_VIVADO (default ~/vivado/2025.2), put on PATH by the shellHook.
{
  description = "Omnimeter dev environment";

  nixConfig = {
    extra-substituters = [ "https://hexadecimaldinosaur.cachix.org" ];
    extra-trusted-public-keys = [ "hexadecimaldinosaur.cachix.org-1:793qwEYh8UCmiTHMSFkz1/mu8mRDC7VCr7xI7e8M8s8=" ];
  };

  inputs = {
    # nixpkgs-unstable as of 2026-10-28: KiCad 10.0.7, the version Konnect
    # 0.13 drives — bump only together with Konnect's KiCad.
    nixpkgs.url = "github:NixOS/nixpkgs/58deea654ec5dd0644f31c840a07c29fa0592528";
    # Deliberately *not* following our nixpkgs: oxcaml-nix's own pin is what
    # its cachix cache was built against, and following ours would rebuild
    # the OxCaml compiler and every Jane Street package from source.
    oxcaml-nix.url = "git+https://git.underpowe.red/hexadecimalDinosaur/oxcaml-nix";
  };

  outputs = { nixpkgs, oxcaml-nix, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      ox = oxcaml-nix.legacyPackages.${system}.oxcamlPackages;
      # oxcaml-nix pairs its compiler with its own nixpkgs' dune.
      oxDune = oxcaml-nix.inputs.nixpkgs.legacyPackages.${system}.dune;
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = (with pkgs; [
          # bay/ — card, keepout, drawings, checks
          kicad
          freecad
          poppler-utils
          python3
        ]) ++ [ oxDune ] ++ (with ox; [
          # gateware/ — compiler and editor tooling
          oxcaml
          findlib
          ocaml-lsp-server
          ocamlformat
          merlin
          # gateware/ — libraries
          core
          ppx_jane
          ppx_expect
          hardcaml
          ppx_hardcaml
          hardcaml_waveterm
          hardcaml_axi
          hardcaml_circuits
          hardcaml_xilinx
        ]);

        shellHook = ''
          export OMNI_VIVADO="''${OMNI_VIVADO:-$HOME/vivado/2025.2}"
          if [ -x "$OMNI_VIVADO/Vivado/bin/vivado" ]; then
            export PATH="$PATH:$OMNI_VIVADO/Vivado/bin"
          fi
        '';
      };
    };
}
