#!/usr/bin/env bash
# Bay reference test — the gate of decision 0009: is the bay device in an
# IOMMU group VFIO can take whole? Records its group's members, every group
# on the machine, and ACSCap/ACSCtl on the root port, then prints one line:
#
#   GATE PASS  the group holds only the device's own functions and PCI
#              bridges on pcieport (or unbound): those do not stop vfio
#              taking it (driver/README.md, kernel findings)
#   GATE FAIL  another endpoint shares the group (NVMe, USB4, ...): VFIO
#              would need the ACS-override patch; 0009 reopens
#   SKIP       no device, or no IOMMU groups (IOMMU off, or WSL): no verdict
#
#   sudo -E bash benchmarks/bay/iommu_groups.sh <BDF>|--card
set -euo pipefail
. "$(dirname "$0")/lib.sh"

if [ -z "$(ls -A /sys/kernel/iommu_groups 2>/dev/null)" ]; then
  bay_skip "no IOMMU groups in /sys/kernel/iommu_groups — the IOMMU is off or absent (WSL has none). On the FW16: IOMMU on in BIOS and the fw16-fpga NixOS module (iommu=pt; no amd_iommu=off)."
fi
bay_pick "${1:-}"
out=$(bay_result_file iommu "$bdf")
rp=$(bay_root_port "$bdf")
dev="/sys/bus/pci/devices/$bdf"
[ -e "$dev/iommu_group" ] || bay_skip "$bdf has no IOMMU group (not behind an IOMMU)"
group=$(basename "$(readlink -f "$dev/iommu_group")")
slot=${bdf%.*}

verdict=PASS
{
  bay_header
  bay_root_note
  echo "== device $bdf in IOMMU group $group; root port ${rp:-<none>}"
  echo "== members of group $group:"
  for m in "$dev"/iommu_group/devices/*; do
    m=$(basename "$m")
    class=$(cat "/sys/bus/pci/devices/$m/class" 2>/dev/null || echo "?")
    drv=$(basename "$(readlink "/sys/bus/pci/devices/$m/driver" 2>/dev/null)" 2>/dev/null || true)
    if [ "${m%.*}" = "$slot" ]; then why="the device's own function"
    # Only drivers with driver_managed_dma leave the group claimable
    # (driver/README.md, kernel findings): pcieport, pci-stub, vfio-pci.
    elif [ "${class:0:6}" = "0x0604" ]; then
      case "${drv:-none}" in
        none|pcieport|pci-stub|vfio-pci) why="PCI bridge" ;;
        *) why="PCI BRIDGE ON A DMA-API DRIVER"; verdict=FAIL ;;
      esac
    else why="OTHER ENDPOINT"; verdict=FAIL
    fi
    echo "   $m class=$class driver=${drv:-none}  [$why]  $(lspci -s "$m" -nn 2>/dev/null)"
  done
  if [ -n "$rp" ]; then
    echo "== root port $rp ACS (needs root):"
    lspci -vv -s "$rp" | grep -E 'Access Control Services|ACSCap|ACSCtl' | sed 's/^/   /' \
      || echo "   no ACS capability listed (or capabilities hidden: not root)"
  fi
  echo "== all IOMMU groups:"
  for g in /sys/kernel/iommu_groups/*; do
    for m in "$g"/devices/*; do
      echo "   group $(basename "$g"): $(lspci -s "$(basename "$m")" -nn 2>/dev/null || basename "$m")"
    done
  done | sort -V
  echo "GATE $verdict (decision 0009): $bdf in group $group"
} > "$out" 2>&1

grep -E '^(NOTE|== device|   [0-9a-f]{4}:|   (ACS|no ACS)|GATE)' "$out"
echo "wrote $out"
[ "$verdict" = PASS ]
