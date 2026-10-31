# Headless workers' Bash allow-list blocks compound commands and some script runs; plan worker steps around it

Seen in round 1 (2026-10-31), despite matching `--allowedTools` patterns:
- `cd X && …`, pipes into `grep`/`find`, and `sed -i` were blocked. Use absolute paths, `nix develop /abs/path -c cmd`,
  and the Read/Grep/Edit tools.
- `fpga/gateware/vivado/run.sh all` was denied even with `Bash(bash fpga/gateware/vivado/run.sh:*)` allowed (the worker
  ran it without the `bash` prefix). `tar -xJf` of the kernel source needed approval.
- So Vivado and other long scripts need an exact allow pattern for the form the worker will type, or the
  orchestrator runs them interactively.
