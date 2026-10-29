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
# Absolute: the nix-shell PATH drops the Windows interop dirs.
cmd_exe=/mnt/c/Windows/System32/cmd.exe
build=${OMNI_VIVADO_BUILD:-/mnt/c/Users/17786/omnimeter-build/vivado}
# /mnt/c/x/y -> C:\x\y by hand, so wslpath (not on every distro's PATH) isn't needed.
[[ $build =~ ^/mnt/([a-z])/(.*)$ ]] || { echo "OMNI_VIVADO_BUILD must be under /mnt/<drive>/" >&2; exit 2; }
win="${BASH_REMATCH[1]^^}:\\${BASH_REMATCH[2]//\//\\}"
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)

stage() {
  (cd "$root" && dune build gateware/vivado/omni_shell.v)
  mkdir -p "$build/src" "$build/logs"
  cp "$here"/*.tcl "$here"/*.v "$here"/*.xdc "$build/src/"
  cp -f "$root/_build/default/gateware/vivado/omni_shell.v" "$build/src/"
}

vivado() {
  local step=$1
  # cmd.exe refuses a UNC cwd; the build dir is drive-local and takes Vivado's .Xil/ scratch.
  (cd "$build" && "$cmd_exe" /c "$vivado_bat -mode batch -nojournal -log $win\\logs\\$step.log -source $win\\src\\$step.tcl")
}

case ${1:-} in
  project) stage; vivado create_project ;;
  synth)   stage; vivado synth ;;
  sim)     stage; vivado sim_bfm ;;
  all)     stage; vivado create_project; vivado synth; vivado sim_bfm ;;
  *) sed -n '2,8p' "$0"; exit 2 ;;
esac
