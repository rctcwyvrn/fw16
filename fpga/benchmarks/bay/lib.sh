# Shared by the bay reference tests (sourced, not run): picking the device
# under test, walking to its root port, and the dated result file.
#
# Exit codes every script uses:
#   0 PASS (or: snapshot written)   1 FAIL   2 usage error
#   3 SKIP — nothing to test here (no device given or found, no IOMMU groups).
#        A SKIP is never a verdict on decision 0009's gate.

bay_here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
bay_results="${FW16_BAY_RESULTS:-$bay_here/results}"

# The card's XDMA subsystem ID (gateware/vivado/xdma_0.tcl, regmap v0.2 §3).
bay_card_subsys="10ee:4650"

bay_skip() { echo "SKIP: $*"; exit 3; }
bay_usage() { echo "usage: $(basename "$0") <BDF>|--card $*" >&2; exit 2; }

bay_need() {
  local t
  for t in "$@"; do
    command -v "$t" >/dev/null || { echo "missing tool: $t (run from the dev shell: nix develop)" >&2; exit 2; }
  done
}

# Sets $bdf from "$1": a BDF (DDDD:BB:DD.F or BB:DD.F), or --card for the
# device with the card's subsystem ID. With no argument, list candidates
# and SKIP.
bay_pick() {
  bay_need lspci
  local arg="${1:-}"
  if [ -z "$arg" ]; then
    echo "no device given. PCI devices here (pass one as <BDF>, or --card):"
    lspci -D -nn | sed 's/^/  /'
    [ -n "$(lspci -D)" ] || echo "  (none: no PCI bus visible, e.g. WSL)"
    bay_skip "no device under test"
  fi
  if [ "$arg" = "--card" ]; then
    bdf=$(lspci -D -vmm -n | awk -v want="$bay_card_subsys" '
      /^Slot:/ { slot = $2 } /^SVendor:/ { sv = $2 } /^SDevice:/ { sd = $2 }
      /^$/ { if (sv ":" sd == want) print slot; sv = sd = "" }' | head -n 1)
    [ -n "$bdf" ] || bay_skip "no device with subsystem $bay_card_subsys (is the card in the bay?)"
    return
  fi
  case "$arg" in
    *:*:*.*) bdf="$arg" ;;
    *:*.*) bdf="0000:$arg" ;;
    *) bay_usage ;;
  esac
  [ -e "/sys/bus/pci/devices/$bdf" ] || bay_skip "no PCI device $bdf"
}

# The root port above $1: walk sysfs parents while they are PCI devices and
# keep the topmost one. Empty when $1 sits directly on a root bus (e.g. an
# integrated endpoint, or QEMU's pcie.0).
bay_root_port() {
  local d top="" p
  d=$(readlink -f "/sys/bus/pci/devices/$1")
  while p=$(dirname "$d"); [ -e "$p/config" ]; do
    top=$(basename "$p")
    d="$p"
  done
  echo "$top"
}

# Capabilities (ACS, link, AER) are only readable as root.
bay_root_note() {
  if [ "$(id -u)" -ne 0 ]; then
    echo "NOTE: not root — lspci shows 'Capabilities: <access denied>'; rerun with sudo for LnkCap/ACS."
  fi
}

# A fresh dated result file for this script and device; prints its path.
bay_result_file() {
  local name="$1" dev="${2:-none}"
  mkdir -p "$bay_results"
  echo "$bay_results/$(date +%Y-%m-%d-%H%M%S)-$(hostname)-$name-${dev//:/_}.txt"
}

bay_header() {
  echo "== $(basename "$0") $(date -Is)"
  echo "== host $(hostname), kernel $(uname -r)"
  echo "== cmdline $(cat /proc/cmdline 2>/dev/null)"
}
