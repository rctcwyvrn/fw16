# The driver-module test VM (see module-vm.nix), built from the flake's
# nixpkgs pin like host/vm/default.nix. Build:
#   nix build -f driver/vm -o driver/target/vm/result
let
  lock = builtins.fromJSON (builtins.readFile ../../../flake.lock);
  nixpkgs = builtins.fetchTree lock.nodes.nixpkgs.locked;
  nixos = import "${nixpkgs}/nixos" {
    system = "x86_64-linux";
    configuration = ./module-vm.nix;
  };
in
nixos.vm
