# The dev environment for the whole repo — the single source of truth for
# every tool: KiCad (the card), FreeCAD (bay keepout and the card envelope),
# poppler (reading Framework's PDF drawings), Python (fpga/bay/tools checks),
# and OxCaml + Hardcaml (fpga/gateware). Enter with `nix develop`; `shell.nix` is a
# shim onto this flake so `nix-shell` keeps working.
#
# Hardcaml is not in nixpkgs; it comes from oxcaml-nix, which packages the
# Jane Street 0.18-preview set on the OxCaml compiler. OCaml *libraries* the
# gateware needs are listed here because oxcaml-nix is their only source;
# there is no opam switch.
#
# Vivado is not provided by nix: it is the user's install under
# $FW16_FPGA_VIVADO (default ~/vivado/2025.2), put on PATH by the shellHook.
#
# Konnect (the KiCad MCP server) is not provided here; it runs from
# ~/bin/konnect and drives the kicad-cli from this same pin.
{
  description = "fw16 dev environment (FPGA bay module, split keyboard)";

  nixConfig = {
    extra-substituters = [ "https://hexadecimaldinosaur.cachix.org" ];
    extra-trusted-public-keys = [ "hexadecimaldinosaur.cachix.org-1:793qwEYh8UCmiTHMSFkz1/mu8mRDC7VCr7xI7e8M8s8=" ];
  };

  inputs = {
    # nixpkgs-unstable as of 2026-10-28: KiCad 10.0.7, the version Konnect
    # 0.13 drives — bump only together with Konnect's KiCad.
    nixpkgs.url = "github:NixOS/nixpkgs/58deea654ec5dd0644f31c840a07c29fa0592528";
    # Second pin, used ONLY for bay-python (the bay/tools CAD env). At the
    # main pin, pythonocc-core segfaults loading Framework's
    # FW_EXPANSION_BAY_PCB_REF__0807.stp (Extrema_ExtCC in ShapeFix_Solid);
    # this rev, the user's nixpkgs channel on 2026-10-29 (26.05pre958799),
    # loads it cleanly. Bump independently of the main pin, and only after
    # re-running bay/tools/step_rays_check.py against that STEP.
    nixpkgs-bay.url = "github:NixOS/nixpkgs/917fec990948658ef1ccd07cef2a1ef060786846";
    # Deliberately *not* following our nixpkgs: oxcaml-nix's own pin is what
    # its cachix cache was built against, and following ours would rebuild
    # the OxCaml compiler and every Jane Street package from source.
    oxcaml-nix.url = "git+https://git.underpowe.red/hexadecimalDinosaur/oxcaml-nix";
  };

  outputs = { nixpkgs, nixpkgs-bay, oxcaml-nix, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
      ox = oxcaml-nix.legacyPackages.${system}.oxcamlPackages;
      # oxcaml-nix pairs its compiler with its own nixpkgs' dune.
      oxDune = oxcaml-nix.inputs.nixpkgs.legacyPackages.${system}.dune;
      # bay/tools: pythonocc reads Framework's STEPs, ezdxf writes the card
      # DXFs. Exposed as `bay-python` so it never shadows the main python3.
      bayPython = nixpkgs-bay.legacyPackages.${system}.python3.withPackages
        (ps: [ ps.pythonocc-core ps.numpy ps.ezdxf ]);
      bayPythonBin = pkgs.writeShellScriptBin "bay-python" ''
        exec ${bayPython}/bin/python3 "$@"
      '';
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = (with pkgs; [
          # bay/ — card, keepout, drawings, checks
          kicad
          freecad
          poppler-utils
          # Plain python3 from the main pin. Its pythonocc segfaults on the
          # PCB_REF STEP — run bay/tools scripts with bay-python instead.
          (python3.withPackages (ps: [ ps.pythonocc-core ps.ezdxf ]))
          # host/ — the fw16-fpga crate; crates themselves come from Cargo.lock
          rustc
          cargo
          clippy
          rustfmt
          rust-analyzer
          # host/ — QEMU VM (vIOMMU + edu device) for the vfio layer's tests
          qemu_kvm
        ]) ++ [ bayPythonBin oxDune ] ++ (with ox; [
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

        # The kicad wrapper sets these only for its own binaries; Konnect (and
        # any script) resolves stock libraries like Device:R through them, so
        # export them for everything started from this shell — including
        # Claude Code.
        KICAD10_SYMBOL_DIR = "${pkgs.kicad.libraries.symbols}/share/kicad/symbols";
        KICAD10_FOOTPRINT_DIR = "${pkgs.kicad.libraries.footprints}/share/kicad/footprints";
        KICAD10_3DMODEL_DIR = "${pkgs.kicad.libraries.packages3d}/share/kicad/3dmodels";
        # host/: bindgen (fw16-fpga's build.rs) loads libclang through this.
        LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

        shellHook = ''
          export FW16_FPGA_VIVADO="''${FW16_FPGA_VIVADO:-$HOME/vivado/2025.2}"
          if [ -x "$FW16_FPGA_VIVADO/Vivado/bin/vivado" ]; then
            export PATH="$PATH:$FW16_FPGA_VIVADO/Vivado/bin"
          fi
        '';
      };
    };
}
