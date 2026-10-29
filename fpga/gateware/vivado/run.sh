#!/usr/bin/env bash
# run.sh: drive the Windows Vivado 2025.2 from WSL (run inside nix-shell, from anywhere in the repo).
#
#   gateware/vivado/run.sh project   # emit omni_shell.v, create <build>/omni with xdma_0
#   gateware/vivado/run.sh synth     # synthesis + reports/synth_*.txt
#   gateware/vivado/run.sh sim       # XDMA example + root-port BFM sim, BAR0 ID check
#   gateware/vivado/run.sh all       # the three in order
#
# <build> is Windows-local (default C:\Users\17786\omnimeter-build\vivado, override with
# OMNI_VIVADO_BUILD as a WSL path) and outside the repo. Every step re-copies the sources into
# <build>/src first, so Vivado never touches the \\wsl.localhost share.
set -euo pipefail

vivado_bat='F:\AMDDesignTools\2025.2\Vivado\bin\vivado.bat'
build=${OMNI_VIVADO_BUILD:-/mnt/c/Users/17786/omnimeter-build/vivado}
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)

stage() {
  (cd "$root" && dune build gateware/vivado/omni_shell.v)
  mkdir -p "$build/src" "$build/logs"
  cp "$here"/*.tcl "$here"/*.v "$here"/*.xdc "$build/src/"
  cp -f "$root/_build/default/gateware/vivado/omni_shell.v" "$build/src/"
}

vivado() {
  local step=$1 win
  win=$(wslpath -w "$build")
  # cmd.exe refuses a UNC cwd, hence /mnt/c.
  (cd /mnt/c && cmd.exe /c "$vivado_bat -mode batch -nojournal -log $win\\logs\\$step.log -source $win\\src\\$step.tcl")
}

case ${1:-} in
  project) stage; vivado create_project ;;
  synth)   stage; vivado synth ;;
  sim)     stage; vivado sim_bfm ;;
  all)     stage; vivado create_project; vivado synth; vivado sim_bfm ;;
  *) sed -n '2,8p' "$0"; exit 2 ;;
esac
