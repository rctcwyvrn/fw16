# Headless workers' Bash allow-list blocks compound commands and some script runs; plan worker steps around it

Seen in round 1 (2026-10-31), despite matching `--allowedTools` patterns:
- `cd X && …`, pipes into `grep`/`find`, and `sed -i` were blocked. Use absolute paths, `nix develop /abs/path -c cmd`,
  and the Read/Grep/Edit tools.
- `fpga/gateware/vivado/run.sh all` was denied even with `Bash(bash fpga/gateware/vivado/run.sh:*)` allowed (the worker
  ran it without the `bash` prefix). `tar -xJf` of the kernel source needed approval.
- So Vivado and other long scripts need an exact allow pattern for the form the worker will type, or the
  orchestrator runs them interactively.
- Round 2: denied `bash fpga/driver/vm/run-check.sh` (both forms), `bash -n <script>`, `rustfmt <file>`, `find`/`ls`
  into `/nix/store`, commands with `{{…}}` or `sed`. Allowed: `cargo test/clippy/fmt --check`, `nix eval`,
  `nix build -f …`, `tar -xJf` into `_build/`; Write + Glob/Read on exact store paths.
- **A worker's background task dies when its turn ends.** F3's `run.sh all`, started in the background, was killed at
  the end of the headless run and left the step unverified. Workers must run long jobs in the foreground with a
  timeout (Vivado `all` ≈ 40 min).
- Waiting on `pgrep -f "run.sh"` from a Bash loop never ends: the loop's own command line contains "run.sh". Wait on a
  task notification or a file instead.
