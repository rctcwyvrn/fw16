# `bay-python` from `nix develop` failed with "GLIBC_2.43 not found" (2026-11-01); measure STEPs in Fusion instead

Importing `OCC.Core.STEPControl` through `nix develop --accept-flake-config -c bay-python` failed:
`libm.so.6: version GLIBC_2.43 not found (required by …/opencascade-occt-7.9.3/lib/libTKService.so.7.9)`. The cause
was not investigated (the second nixpkgs pin vs. the main pin's glibc in the shell environment is the likely suspect).
Fusion's MCP measured the RP2040-Zero STEP fine: import into a throwaway assembly document, walk occurrences, read
cylinder faces, close without saving.
