#!/usr/bin/env bash
# Run fw16-fpga's vfio layer against QEMU's edu device in a NixOS VM with a
# vIOMMU (Path 4 step 4). Run from the dev shell (`nix develop`):
#
#   bash host/vm/run-edu-check.sh [--suspend]
#
# Builds examples/edu_check, builds the VM from the flake's nixpkgs pin
# (host/vm/default.nix), boots it headless and prints the guest's log.
# `--suspend` adds a real suspend/resume cycle to the inhibitor check.
# Everything it writes lives under host/target/vm/ (git- and jj-ignored).
# Exit 0 only if the guest printed EDU-CHECK PASS.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
host=$(dirname "$here")
work="$host/target/vm"
timeout_s=${FW16_FPGA_VM_TIMEOUT:-1200}

mkdir -p "$work/shared" "$work/tmp"
rm -f "$work/shared/edu-check.log"

(cd "$host" && cargo build --example edu_check)
cp -f "$host/target/debug/examples/edu_check" "$work/shared/edu_check"
printf '%s\n' "$*" > "$work/shared/edu-check.args"

nix build -f "$here" -o "$work/result"
runner=$(echo "$work"/result/bin/run-*-vm)

# The NixOS runner shares $SHARED_DIR as /tmp/shared and keeps its own
# scratch under $TMPDIR (only if USE_TMPDIR is set; otherwise a mktemp
# under /tmp). No KVM (e.g. WSL2) means TCG: slow, but it works.
echo "booting the VM (log: $work/console.log, timeout ${timeout_s}s)"
rc=0
SHARED_DIR="$work/shared" TMPDIR="$work/tmp" USE_TMPDIR=1 \
  timeout "$timeout_s" "$runner" </dev/null >"$work/console.log" 2>&1 || rc=$?
if [ "$rc" -ne 0 ]; then
  echo "VM exited with $rc (124 = timeout); tail of console.log:"
  tail -n 40 "$work/console.log"
fi

if [ ! -f "$work/shared/edu-check.log" ]; then
  echo "no edu-check.log: the check never ran"
  exit 1
fi
cat "$work/shared/edu-check.log"
grep -q '^EDU-CHECK PASS' "$work/shared/edu-check.log"
