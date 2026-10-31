# benchmarks/bay — bay reference tests

Path 4 step 1 ([PLAN](../../PLAN.md)): scripts run on the FW16 the day it arrives, first against the Dual M.2
adapter + NVMe, later against the card. Each writes a dated file to `results/` and prints a summary.
Run from the repo's dev shell (`nix develop`, for `lspci`/`setpci`), as root so capabilities are readable:

```sh
cd fpga
sudo -E bash benchmarks/bay/iommu_groups.sh <BDF>      # the 0009 gate: GATE PASS / GATE FAIL / SKIP
sudo -E bash benchmarks/bay/lspci_snapshot.sh <BDF>    # LnkCap/LnkSta of device + root port, lspci -vv, tree
sudo -E bash benchmarks/bay/enum_dmesg.sh <BDF>        # kernel log of enumeration (link, AER, IOMMU, vfio)
sudo -E bash benchmarks/bay/suspend_cycle.sh <BDF> --yes [secs]   # real suspend/resume; PASS if unchanged
```

`<BDF>` is the bay device (`lspci -D`); `--card` instead finds the card by its subsystem ID 10EE:4650. With no
argument a script lists the PCI devices and exits SKIP.

| Exit | Meaning |
| --- | --- |
| 0 | PASS / snapshot written |
| 1 | FAIL (`iommu_groups.sh`: another endpoint shares the group; `suspend_cycle.sh`: the device changed) |
| 2 | usage error, missing tool, or not root where root is required |
| 3 | SKIP — nothing to test (no device, no IOMMU groups, WSL); never a verdict on the 0009 gate |

`iommu_groups.sh` passes when every other member of the device's group is one of its own functions or a PCI
bridge (class 0x0604). `suspend_cycle.sh` suspends the machine it runs on (`systemctl suspend`, RTC wake) and
refuses to run without `--yes` or under WSL. `lib.sh` holds the shared pieces (device pick, root-port walk,
result file names).
