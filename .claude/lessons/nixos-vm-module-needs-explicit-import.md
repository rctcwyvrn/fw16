# At nixpkgs 58deea65, a NixOS config built with `.vm` must import qemu-vm.nix itself

`import <nixpkgs/nixos> { configuration = …; }`'s `.vm` no longer pulls in `qemu-vm.nix`; add
`imports = [ (modulesPath + "/virtualisation/qemu-vm.nix") ]` or every `virtualisation.*` option "does not exist".
The generated runner also ignores `$TMPDIR` unless `USE_TMPDIR` is set. WSL2 has no `/dev/kvm`, so the VM runs
under TCG (minutes per boot). Found by the Path 4b worker, 2026-10-29 (`host/vm/`).
