# ppx_expect 0.18-preview needs `-source-tree-root .` under dune, or failing expect tests crash instead of diffing

The 0.18-preview runtime joins `-source-tree-root` with only the *basename* of the test file. Dune passes
`%{workspace_root}` (e.g. `../..`), so a failing test under `gateware/sim/` raised `Sys_error "../../test_x.ml"`
and never printed a diff. Fix (in `gateware/sim/dune`): `(inline_tests (flags (-source-tree-root .)))` — the last
flag wins and the runner's cwd holds the sources. Found by the Path 3 worker, 2026-10-29.
