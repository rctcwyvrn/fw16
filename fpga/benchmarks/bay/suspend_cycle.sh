#!/usr/bin/env bash
# Bay reference test: one real suspend-to-RAM cycle with the bay device in,
# comparing it before and after — still present, config space readable, same
# driver, same link speed/width, AER counters — plus the kernel log of the
# cycle, into a dated file under results/. PASS only if nothing changed.
#
#   sudo -E bash benchmarks/bay/suspend_cycle.sh <BDF>|--card --yes [seconds]
#
# Suspends THIS machine (wakes by RTC alarm after `seconds`, default 20).
# Goes through `systemctl suspend`, so logind's PrepareForSleep reaches
# fw16-fpga's delay inhibitor just as a lid close would. Refuses to run
# without --yes, and always refuses under WSL.
set -euo pipefail
. "$(dirname "$0")/lib.sh"

if grep -qi microsoft /proc/version 2>/dev/null; then
  bay_skip "WSL: no suspend to test here"
fi
target="${1:-}"
[ "${2:-}" = "--yes" ] || bay_usage "--yes [seconds]   (suspends this machine)"
secs="${3:-20}"
bay_need rtcwake systemctl
[ "$(id -u)" -eq 0 ] || { echo "run as root (rtcwake, AER and link registers)" >&2; exit 2; }
bay_pick "$target"
out=$(bay_result_file suspend "$bdf")
rp=$(bay_root_port "$bdf")

state() {
  local d s=/sys/bus/pci/devices/$1
  [ -e "$s" ] || { echo "$1 GONE"; return; }
  d=$(basename "$(readlink "$s/driver" 2>/dev/null)" 2>/dev/null || true)
  echo "$1 vendor=$(setpci -s "$1" VENDOR_ID 2>/dev/null || echo ?)" \
       "driver=${d:-none}" \
       "link=$(cat "$s/current_link_speed" 2>/dev/null || echo ?)/x$(cat "$s/current_link_width" 2>/dev/null || echo ?)"
}
aer() {
  local f
  for f in /sys/bus/pci/devices/"$1"/aer_dev_{correctable,nonfatal,fatal}; do
    [ -e "$f" ] && echo "-- $(basename "$f"):" && grep -v ' 0$' "$f" | sed 's/^/   /'
  done
  true
}

{
  bay_header
  echo "== device $bdf, root port ${rp:-<none>}; suspending for ${secs}s"
  before=$( { state "$bdf"; [ -z "$rp" ] || state "$rp"; } )
  echo "== before:"; echo "$before" | sed 's/^/   /'
  aer "$bdf"
  t0=$(date -Is)
  rtcwake -m no -s "$secs"
  systemctl suspend
  # systemctl returns at once; wait until the machine has slept and woken.
  sleep $((secs + 15))
  after=$( { state "$bdf"; [ -z "$rp" ] || state "$rp"; } )
  echo "== after:"; echo "$after" | sed 's/^/   /'
  aer "$bdf"
  echo "== kernel log since $t0 (PM, PCI, link, vfio):"
  journalctl -k --since "$t0" -o short-monotonic --no-pager 2>/dev/null \
    | grep -iE "PM:|$bdf${rp:+|$rp}|pcieport|AER|link|vfio|ACPI: (PM|EC)" || echo "   (none)"
  if [ "$before" = "$after" ]; then
    echo "SUSPEND PASS: $bdf unchanged across the cycle"
  else
    echo "SUSPEND FAIL: $bdf changed across the cycle (see before/after)"
  fi
} > "$out" 2>&1

grep -E '^(== (device|before|after)|   [0-9a-f]{4}:|SUSPEND)' "$out"
echo "wrote $out"
grep -q '^SUSPEND PASS' "$out"
