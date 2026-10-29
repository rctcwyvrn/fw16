# Shim: the dev environment lives in flake.nix (`nix develop`). This keeps
# `nix-shell` working by loading that flake's default devShell, so there is
# one list of tools. Add tools to flake.nix, never here.
#
# getFlake does not apply the flake's nixConfig, so the OxCaml cachix cache
# is only used by `nix develop --accept-flake-config`; run that once first or
# `nix-shell` will build OxCaml from source.
# git+file (not a bare path) so only tracked files are copied to the store —
# a path flake would copy bay/lib/ and every build product too.
(builtins.getFlake ("git+file://" + toString ./.)).devShells.${builtins.currentSystem}.default
