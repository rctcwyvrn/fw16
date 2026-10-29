# Hardcaml is not in nixpkgs; get it from oxcaml-nix, which means OxCaml and the 0.18-preview API

Checked 2026-10-28: no `hardcaml*` attribute in nixos-25.05 or nixos-unstable (Jane Street 0.17 base libs are
there). The user pointed at https://git.underpowe.red/hexadecimalDinosaur/oxcaml-nix, which packages the whole
Jane Street set including hardcaml, ppx_hardcaml, hardcaml_waveterm/axi/xilinx/circuits at
`0.18.0-preview.130.106+341` on the OxCaml compiler, with a cachix cache (hexadecimaldinosaur.cachix.org).

- Do **not** make its `nixpkgs` input follow ours: the cache was built against its own pin, and following
  rebuilds the compiler and every package from source.
- Code is written against the 0.18-preview API, not the 0.17 docs most examples on the web use.
- The user prefers a root `flake.nix` over `shell.nix` here (asked 2026-10-28); `shell.nix` stays as a shim.

## 0.18-preview renames (found porting the shell, 2026-10-29)
- `<==` → `<--`; widths/shifts are labelled (`sll ~by`, `sel_bottom ~width`, `drop_bottom ~width`, `select ~high ~low`).
- `of_int`/`to_int` → `of_unsigned_int`/`to_unsigned_int` (raises on overflow instead of truncating — this caught a real decoder aliasing bug).
- `Int.ceil_log2` → `address_bits_for`; `Reg_spec.override ~clear_to` → `reg`/`reg_fb ~clear_to`; `Fifo.create` needs `~scope`; `Waveform.create` → `Cyclesim.Waveform.create`.
- Signatures live in `…hardcaml-0.18.0-preview…/site-lib/hardcaml/kernel/` (`comb_intf.ml`, `signal_intf.ml`, `reg_spec_intf.ml`), not the top-level dir.
- Nested interface ports are named `<rtlprefix><field>$<sub>` (e.g. `s_axil_axil$awvalid`) unless `rtlmangle` is set.
