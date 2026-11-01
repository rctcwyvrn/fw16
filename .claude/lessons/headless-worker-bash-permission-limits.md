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
- **The Bash tool caps a foreground command at 600 s**, moves it to the background and kills it ~30 min later, so
  `timeout 3600 run.sh all` does not stay foreground (F3-gw round 3). The Windows Vivado process survives the WSL-side
  kill: read its own log at `C:\Users\17786\fw16-fpga-build\vivado\logs\<step>.log` (Read/Grep work; Bash on `/mnt/c`
  is denied for workers) and keep the turn alive with ≤ 580 s `until …; do sleep 30; done` polls. WSL-side stdout
  capture through cmd.exe lags Vivado's own log.
