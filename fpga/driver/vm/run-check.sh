#!/usr/bin/env bash
# Prove driver/fw16-fpga.nix in a NixOS VM (Path 4 step 2). Run from the dev
# shell (`nix develop`), from fpga/:
#
#   bash driver/vm/run-check.sh
#
# Builds host/'s edu_check example, builds the VM (driver/vm/default.nix),
# copies edu_check, module-check.sh and the bay scripts into the share, boots
# headless and prints the guest's log. Everything it writes lives under
# driver/target/vm/ (git-ignored). Exit 0 only if the guest printed
# MODULE-CHECK PASS. No KVM (WSL2) means TCG: minutes per boot.
set -euo pipefail

here=$(cd "$(dirname "$0")" && pwd)
fpga=$(cd "$here/../.." && pwd)
work="$fpga/driver/target/vm"
timeout_s=${FW16_FPGA_VM_TIMEOUT:-1200}

rm -rf "$work/shared"
mkdir -p "$work/shared/bay" "$work/tmp"

(cd "$fpga/host" && cargo build --example edu_check)
cp -f "$fpga/host/target/debug/examples/edu_check" "$work/shared/edu_check"
cp -f "$here/module-check.sh" "$work/shared/"
cp -f "$fpga"/benchmarks/bay/*.sh "$work/shared/bay/"

nix build -f "$here" -o "$work/result"
runner=$(echo "$work"/result/bin/run-*-vm)

echo "booting the VM (log: $work/console.log, timeout ${timeout_s}s)"
rc=0
SHARED_DIR="$work/shared" TMPDIR="$work/tmp" USE_TMPDIR=1 \
  timeout "$timeout_s" "$runner" </dev/null >"$work/console.log" 2>&1 || rc=$?
if [ "$rc" -ne 0 ]; then
  echo "VM exited with $rc (124 = timeout); tail of console.log:"
  tail -n 40 "$work/console.log"
fi

if [ ! -f "$work/shared/module-check.log" ]; then
  echo "no module-check.log: the check never ran"
  exit 1
fi
cat "$work/shared/module-check.log"
grep -q '^MODULE-CHECK PASS' "$work/shared/module-check.log"
