# The flake's pythonocc (nixpkgs 58deea65, OCCT 7.9.3) segfaults loading FW_EXPANSION_BAY_PCB_REF__0807.stp

Crash is in `Extrema_ExtCC` via `ShapeFix_Solid` during `TransferRoots`, on python 3.13 and 3.14 from that pin;
overriding `XSTEPResource/STEP` via `CSF_STEPDefaults` is ignored in 7.9. The user-channel
`nix-shell -p "python3.withPackages(ps: with ps; [pythonocc-core numpy ezdxf])"` (OCCT 7.9.3, different store
hash) loads it in 6 s, which is why earlier sessions never hit it. That env is unpinned — fixing the pin is an open
decision (Path 1 Q1, 2026-10-29). Found by the Path 1 worker.
