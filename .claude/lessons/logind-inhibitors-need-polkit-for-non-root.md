# logind grants a delay inhibitor to a non-root user only through polkit

Headless NixOS (the test VMs) has `security.polkit.enable = false`, so logind answers `AccessDenied`. With polkit on,
the stock policy grants it, even under `su`, and no rule is needed (F4-sw, 2026-10-31). The `fw16-fpga` crate's
suspend handling (0009) depends on this.
- The fpga-module NixOS module asserts `security.polkit.enable` (F4-sw round 2).
