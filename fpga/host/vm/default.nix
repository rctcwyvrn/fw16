# The edu VM (see edu-vm.nix), built from the same nixpkgs the dev shell
# uses: the pin is read from the repo's flake.lock, so this file needs no
# flake output and never drifts from the shell. Build:
#   nix build -f host/vm -o host/target/vm/result
let
  lock = builtins.fromJSON (builtins.readFile ../../../flake.lock);
  nixpkgs = builtins.fetchTree lock.nodes.nixpkgs.locked;
  nixos = import "${nixpkgs}/nixos" {
    system = "x86_64-linux";
    configuration = ./edu-vm.nix;
  };
in
nixos.vm
