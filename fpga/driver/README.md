# driver

No kernel module ([decision 0009](../docs/decisions/0009-host-driver.md)): the card is bound to `vfio-pci` and driven from userspace by the `fw16-fpga` crate in `host/`. This directory holds the NixOS module that sets that up. `third_party/` (git-ignored) keeps AMD's `dma_ip_drivers` only as a reference for XDMA descriptor layout.

## `fw16-fpga.nix`

```nix
imports = [ fw16.nixosModules.fw16-fpga ];   # fw16 = this repo's flake
hardware.fw16-fpga.enable = true;
security.polkit.enable = true;               # asserted by the module
users.users.<you>.extraGroups = [ "fw16-fpga" ];
```

Without flakes, import the file directly: `imports = [ "${fw16}/fpga/driver/fw16-fpga.nix" ];`.

- **IOMMU:** adds `iommu=pt` to the kernel command line (`kernelParams` option). There is no `amd_iommu=on`: the AMD driver comes up by default whenever the firmware publishes an IVRS table, and `on` is not one of its options (see the kernel findings below). The IOMMU must be on in the BIOS, or there is no IVRS table.
- **Binding:** loads `vfio-pci` at boot and binds the card with a udev rule that sets `driver_override=vfio-pci` on any PCI device with vendor 10EE and **subsystem** ID 10EE:4650 (`vendor`/`subsystemVendor`/`subsystemDevice` options), then asks the PCI core to probe it. The match must be unique to the card: it claims every *unbound* device that matches (in the test VM, matching QEMU's shared subsystem ID alone grabbed the host bridge, VGA and ISA bridge). The subsystem ID is ours ([regmap v0.2 §3](../docs/regmap-v0.2-proposal.md)); XDMA's VID:DID (10EE:9xxx) is the IP's default and names every XDMA design, so it is not matched on. `vfio-pci.ids=10ee:ffffffff:10ee:4650` would also work: the device field takes `ffffffff` as a wildcard (see below), and the IDs become vfio-pci dynids that every later probe checks (`drivers/pci/pci-driver.c:203-213`), so a card that re-appears after a reprogram + rescan binds too. The udev rule was kept because it is proven in the VM test and `driver_override` also keeps any other driver off the card; the rule's `drivers_probe` does the rebinding after a rescan.
- **Device node:** the card's `/dev/vfio/<iommu group>` gets group `fw16-fpga`, mode 0660 (`group` option). A udev `PROGRAM` checks the IOMMU group holds the card, so other vfio groups (a GPU passed to a VM) stay root-only. `/dev/vfio/vfio` (the container) is world-accessible by default.
- **Memlock:** `VFIO_IOMMU_MAP_DMA` pins every DMA buffer against `RLIMIT_MEMLOCK`; the module sets it to `unlimited` (`memlock` option) for the group's PAM sessions. A systemd service using the card needs `LimitMEMLOCK=` itself.
- **Polkit:** the crate's logind *delay* inhibitor (suspend handling, 0009) is granted to non-root users only through polkit: in the VM test a group member got `AccessDenied` with polkit off (the headless default), and the lock with `security.polkit.enable = true` (no extra rule needed). The module does not turn polkit on; it **asserts** `security.polkit.enable`, so a config without it fails to evaluate.

## Kernel findings (Linux 7.3, the pin's `linuxPackages_latest`)

Checked against the source of `/nix/store/c6pm40ggi8yfbs2rgd371m3fnqzbia35-linux-7.3.tar.xz` (extracted under `_build/`, not in the tree). Paths are relative to the kernel tree.

- **`amd_iommu=on` is not an option.** `parse_amd_iommu_options` (`drivers/iommu/amd/init.c:3753`) knows `fullflush`, `force_enable`, `off`, `force_isolation`, `pgtbl_v1`, `pgtbl_v2`, `irtcachedis`, `nohugepages`, `v2_pgsizes_only`; anything else hits `pr_notice("Unknown option - '%s'\n")` (`init.c:3781`) and is ignored. The documented list matches (`Documentation/admin-guide/kernel-parameters.txt:438`). The driver is on by default: `amd_iommu_disabled` starts false (`init.c:167`), and once `detect_ivrs()` finds the firmware's IVRS table the only thing that stops init is that flag (`init.c:3490-3505`). Dropped from the module.
- **`iommu=pt` is real.** The x86 `iommu=` early param (`arch/x86/kernel/pci-dma.c:109`, registered at `:168`) calls `iommu_set_default_passthrough(true)` on `pt` (`:155`). It sets the *default* domain only; vfio replaces the group's domain when it claims the group (next points).
- **`vfio-pci.ids=` wildcards.** The format is `vendor:device[:subvendor[:subdevice[:class[:class_mask]]]]` (`drivers/vfio/pci/vfio_pci.c:35`), parsed with `sscanf("%x:%x:%x:%x:%x:%x")` (`:255`); fewer than two fields is rejected (`:259`). Omitted subvendor/subdevice default to `PCI_ANY_ID` (`:248`), which is `~0` (`include/linux/device-id/pci.h:10`), and the matcher treats `PCI_ANY_ID` in any of the four ID fields as "any" (`drivers/pci/pci.h:474-483`). So `10ee:ffffffff:10ee:4650` matches any 10EE device with our subsystem; `*` is not accepted (`%x` stops, too few fields, "invalid id string").
- **Bridges in the card's group do not block vfio.** With the container API (what `vfio-ioctls` uses), `VFIO_GROUP_SET_CONTAINER` calls `iommu_group_claim_dma_owner` (`drivers/vfio/container.c:437`), which fails with `EPERM` if the group's `owner_cnt` is non-zero (`drivers/iommu/iommu.c:3431`). `owner_cnt` is raised at bind time by `iommu_device_use_default_domain` (`iommu.c:3316-3339`), which the PCI core calls only for drivers **without** `driver_managed_dma` (`drivers/pci/pci-driver.c:1720`). The group is viable iff `owner_cnt` is zero (`drivers/vfio/group.c:356`).
- **`pcieport` and `driver_managed_dma`.** `pcieport` sets `driver_managed_dma = true` (`drivers/pci/pcie/portdrv.c:801`), as do `pci-stub` (`drivers/pci/pci-stub.c:39`) and `vfio-pci` (`vfio_pci.c:233`). So a root port or switch port bound to `pcieport`, an unbound bridge, or a `pci-stub` device in the group leaves it viable; any other endpoint in the group bound to an ordinary driver makes `SET_CONTAINER` fail with `EPERM`. The reverse also holds: while the card is open, an ordinary driver binding to a group member gets `EBUSY` (`iommu.c:3331-3335`). This is what `benchmarks/bay/iommu_groups.sh`'s PASS/FAIL must mean on the FW16.

## Testing

`bash driver/vm/run-check.sh` (dev shell, from `fpga/`) boots a NixOS VM with a vIOMMU and QEMU's `edu` device, with the module pointed at edu's vendor 1234 and subsystem ID (1AF4:1100, QEMU's default), VGA off so that pair is unique. `vm/module-check.sh` checks the kernel parameters, that the udev rule alone bound edu (and nothing else) to `vfio-pci`, the node's owner and mode, the group's memlock limit, runs `host/`'s `edu_check` as an unprivileged group member, and runs the bay reference scripts against edu. TCG under WSL2: minutes per boot. The real card is first seen on the FW16.

VM check with the round-2 module (polkit asserted and on in the VM, `iommu=pt` without `amd_iommu=on`), booted
2026-11-01: `MODULE-CHECK PASS`, including `user-inhibitor` as the unprivileged `tester` and `GATE PASS`.
