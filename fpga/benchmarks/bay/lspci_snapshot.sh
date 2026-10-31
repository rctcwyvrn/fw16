#!/usr/bin/env bash
# Bay reference test (fpga/PLAN.md Path 4 step 1): record the device under
# test and its root port — BDF, IDs, driver, LnkCap/LnkSta, full `lspci -vv`
# — plus the whole tree, into a dated file under results/.
#
#   sudo -E bash benchmarks/bay/lspci_snapshot.sh <BDF>|--card
#
# Run as root: without it lspci hides every capability (link, ACS, AER).
set -euo pipefail
. "$(dirname "$0")/lib.sh"

bay_pick "${1:-}"
out=$(bay_result_file lspci "$bdf")
rp=$(bay_root_port "$bdf")

{
  bay_header
  bay_root_note
  echo "== device $bdf, root port ${rp:-<none: on a root bus>}"
  echo "== link (device, then root port):"
  for d in "$bdf" $rp; do
    echo "-- $d: $(lspci -s "$d" -nn)"
    echo "   driver: $(basename "$(readlink "/sys/bus/pci/devices/$d/driver" 2>/dev/null)" 2>/dev/null || true)"
    echo "   sysfs speed/width: $(cat "/sys/bus/pci/devices/$d/current_link_speed" 2>/dev/null || echo ?) x$(cat "/sys/bus/pci/devices/$d/current_link_width" 2>/dev/null || echo ?)" \
         "(max $(cat "/sys/bus/pci/devices/$d/max_link_speed" 2>/dev/null || echo ?) x$(cat "/sys/bus/pci/devices/$d/max_link_width" 2>/dev/null || echo ?))"
    lspci -s "$d" -vv | grep -E 'LnkCap:|LnkSta:|LnkCtl2:|LnkSta2:' | sed 's/^/   /' || true
  done
  for d in "$bdf" $rp; do
    echo "== lspci -vv -nn -s $d"
    lspci -vv -nn -s "$d"
  done
  echo "== lspci -tv"
  lspci -tv
  echo "== lspci -D -nnk"
  lspci -D -nnk
} > "$out" 2>&1

grep -E '^(NOTE|== device|-- |   (driver|sysfs|LnkCap|LnkSta))' "$out"
echo "wrote $out"
