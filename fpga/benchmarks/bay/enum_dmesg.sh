#!/usr/bin/env bash
# Bay reference test: the kernel's account of enumerating the bay device —
# every kernel-log line naming it or its root port, plus PCIe link, AER,
# IOMMU and vfio messages since boot — into a dated file under results/.
#
#   sudo -E bash benchmarks/bay/enum_dmesg.sh <BDF>|--card
#
# Reads `dmesg`, falling back to `journalctl -k -b` when dmesg is restricted
# (kernel.dmesg_restrict) for a non-root user.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

bay_pick "${1:-}"
out=$(bay_result_file dmesg "$bdf")
rp=$(bay_root_port "$bdf")

klog() {
  dmesg 2>/dev/null && return
  journalctl -k -b -o short-monotonic --no-pager 2>/dev/null && return
  echo "NOTE: kernel log unreadable (dmesg_restrict, no journal access): rerun with sudo"
}

pat="$bdf${rp:+|$rp}"
{
  bay_header
  bay_root_note
  echo "== device $bdf, root port ${rp:-<none>}"
  log=$(klog)
  named=$(grep -E "$pat" <<<"$log" || true)
  echo "== $(grep -c . <<<"$named" || true) lines naming the device or its root port:"
  echo "${named:-   (none)}"
  echo "== link / AER / hotplug / IOMMU / vfio:"
  grep -iE 'pcieport|link (up|down)|bandwidth|AER|pciehp|AMD-Vi|DMAR|iommu|vfio' <<<"$log" | tail -n 200 \
    || echo "   (none)"
} > "$out" 2>&1

grep -E '^(NOTE|== device|== [0-9]+ lines)' "$out"
echo "wrote $out"
