# `builtins.getFlake "<repo path>"` copies git-ignored files and fails on a socket; read the pin from flake.lock instead

It failed with "has an unsupported type" while a NixOS VM's socket sat under git-ignored `fpga/driver/target/vm/tmp/`.
`nix develop <repo>` uses the git tree and was fine. Read the pin with
`builtins.fetchTree (fromJSON (readFile flake.lock)).nodes.nixpkgs.locked`, as `fpga/host/vm/default.nix` does (F4-sw, 2026-10-31).
