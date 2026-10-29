# Hardcaml is not in nixpkgs; get it from oxcaml-nix, which means OxCaml and the 0.18-preview API

Checked 2026-10-28: no `hardcaml*` attribute in nixos-25.05 or nixos-unstable (Jane Street 0.17 base libs are
there). The user pointed at https://git.underpowe.red/hexadecimalDinosaur/oxcaml-nix, which packages the whole
Jane Street set including hardcaml, ppx_hardcaml, hardcaml_waveterm/axi/xilinx/circuits at
`0.18.0-preview.130.106+341` on the OxCaml compiler, with a cachix cache (hexadecimaldinosaur.cachix.org).

- Do **not** make its `nixpkgs` input follow ours: the cache was built against its own pin, and following
  rebuilds the compiler and every package from source.
- Code is written against the 0.18-preview API, not the 0.17 docs most examples on the web use.
- The user prefers a root `flake.nix` over `shell.nix` here (asked 2026-10-28); `shell.nix` stays as a shim.
