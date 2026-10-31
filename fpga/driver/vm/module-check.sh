#!/usr/bin/env bash
# Runs inside the module VM (module-vm.nix) as root, from /tmp/shared.
# Checks what driver/fw16-fpga.nix promises, with edu as the card:
# kernel params, edu bound to vfio-pci by the udev rule, its /dev/vfio node
# root:fw16-fpga 0660, the group's memlock limit, and the crate's vfio layer
# working as the unprivileged user (edu_check). Then runs the bay reference
# scripts against edu. Last line: MODULE-CHECK PASS|FAIL.
set -uo pipefail
shared=/tmp/shared
group=fw16-fpga
failed=0
check() { # name, ok(0/1), note
  if [ "$2" -eq 0 ]; then echo "PASS  $1  $3"; else echo "FAIL  $1  $3"; failed=1; fi
}

udevadm settle
echo "== kernel $(uname -r)"
echo "== cmdline $(cat /proc/cmdline)"

cl=$(cat /proc/cmdline)
[[ " $cl " == *" amd_iommu=on "* && " $cl " == *" iommu=pt "* ]]
check params $? "amd_iommu=on iommu=pt on the cmdline"

bdf=$(lspci -D -n -d 1234:11e8 | awk '{print $1; exit}')
dev=/sys/bus/pci/devices/$bdf
echo "== edu: $(lspci -nnk -s "$bdf")"
echo "   subsystem $(cat "$dev/subsystem_vendor"):$(cat "$dev/subsystem_device")," \
     "driver_override=$(cat "$dev/driver_override")"
drv=$(basename "$(readlink "$dev/driver")" 2>/dev/null)
[ "$drv" = vfio-pci ]
check bind $? "edu $bdf driver=${drv:-none} (bound by the module's udev rule, no vfio-pci ids=)"

others=$(for d in /sys/bus/pci/drivers/vfio-pci/*:*; do [ "$(basename "$d")" = "$bdf" ] || basename "$d"; done)
[ -z "$others" ]
check only-edu $? "devices on vfio-pci besides edu: ${others:-none}"

grp=$(basename "$(readlink -f "$dev/iommu_group")")
nodes=$(ls /dev/vfio | grep -E '^[0-9]+$' | tr '\n' ' ')
[ "$nodes" = "$grp " ]
check one-node $? "group nodes in /dev/vfio: $nodes(want only $grp)"
echo "== /dev/vfio:"; ls -l /dev/vfio | sed 's/^/   /'
perm=$(stat -c '%U:%G %a' "/dev/vfio/$grp" 2>/dev/null)
[ "$perm" = "root:$group 660" ]
check node $? "/dev/vfio/$grp is '$perm'"

lim=$(/run/wrappers/bin/su - tester -c 'ulimit -l' 2>&1)
[ "$lim" = unlimited ]
check memlock $? "tester's ulimit -l via su - = $lim"

echo "== edu_check as tester (non-root):"
/run/wrappers/bin/su - tester -c "$shared/edu_check" 2>&1 | tee /tmp/edu.log | sed 's/^/   /'
for c in open mmio msi dma iommu; do
  grep -qE "^PASS +$c " /tmp/edu.log
  check "user-$c" $? "edu_check '$c' as tester"
done
grep -qE '^PASS +inhibitor ' /tmp/edu.log \
  && echo "INFO  user-inhibitor passed as tester" \
  || echo "INFO  user-inhibitor did not pass as tester (logind delay lock outside a seat session; see the log above)"

for s in iommu_groups lspci_snapshot enum_dmesg; do
  echo "== bay script $s.sh $bdf (root):"
  FW16_BAY_RESULTS=$shared/bay-results bash "$shared/bay/$s.sh" "$bdf" 2>&1 | sed 's/^/   /'
  echo "   exit ${PIPESTATUS[0]}"
done

echo "MODULE-CHECK $([ "$failed" -eq 0 ] && echo PASS || echo FAIL)"
