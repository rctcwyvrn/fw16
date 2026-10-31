# driver

No kernel module ([decision 0009](../docs/decisions/0009-host-driver.md)): the card is bound to `vfio-pci` and driven from userspace by the `fw16-fpga` crate in `host/`. This directory holds the NixOS module that sets that up. `third_party/` (git-ignored) keeps AMD's `dma_ip_drivers` only as a reference for XDMA descriptor layout.

## `fw16-fpga.nix`

```nix
imports = [ "${fw16}/fpga/driver/fw16-fpga.nix" ];   # fw16 = this repo
hardware.fw16-fpga.enable = true;
users.users.<you>.extraGroups = [ "fw16-fpga" ];
```

- **IOMMU:** adds `amd_iommu=on iommu=pt` to the kernel command line (`kernelParams` option). The IOMMU must also be on in the BIOS.
- **Binding:** loads `vfio-pci` at boot and binds the card with a udev rule that sets `driver_override=vfio-pci` on any PCI device with vendor 10EE and **subsystem** ID 10EE:4650 (`vendor`/`subsystemVendor`/`subsystemDevice` options), then asks the PCI core to probe it. The match must be unique to the card: it claims every *unbound* device that matches (in the test VM, matching QEMU's shared subsystem ID alone grabbed the host bridge, VGA and ISA bridge). The subsystem ID is ours ([regmap v0.2 §3](../docs/regmap-v0.2-proposal.md)); XDMA's VID:DID (10EE:9xxx) is the IP's default and names every XDMA design, so it is not matched on. `vfio-pci.ids=` was not used: its format is `vendor:device[:subvendor:subdevice]`, so it needs the device ID in front of the subsystem ID (or a wildcard device, untested), and the device ID is whatever the XDMA config produces. The `drivers_probe` in the rule also binds the card when it re-appears after a reprogram + rescan.
- **Device node:** the card's `/dev/vfio/<iommu group>` gets group `fw16-fpga`, mode 0660 (`group` option). A udev `PROGRAM` checks the IOMMU group holds the card, so other vfio groups (a GPU passed to a VM) stay root-only. `/dev/vfio/vfio` (the container) is world-accessible by default.
- **Memlock:** `VFIO_IOMMU_MAP_DMA` pins every DMA buffer against `RLIMIT_MEMLOCK`; the module sets it to `unlimited` (`memlock` option) for the group's PAM sessions. A systemd service using the card needs `LimitMEMLOCK=` itself.
- **Not covered:** the crate's logind *delay* inhibitor (suspend handling, 0009). logind grants it to non-root users only through polkit: in the VM test a group member got `AccessDenied` with polkit off (the headless default), and the lock with `security.polkit.enable = true` (no extra rule needed). Desktop NixOS configs normally have polkit on; the module does not enable it.

## Testing

`bash driver/vm/run-check.sh` (dev shell, from `fpga/`) boots a NixOS VM with a vIOMMU and QEMU's `edu` device, with the module pointed at edu's vendor 1234 and subsystem ID (1AF4:1100, QEMU's default), VGA off so that pair is unique. `vm/module-check.sh` checks the kernel parameters, that the udev rule alone bound edu (and nothing else) to `vfio-pci`, the node's owner and mode, the group's memlock limit, runs `host/`'s `edu_check` as an unprivileged group member, and runs the bay reference scripts against edu. TCG under WSL2: minutes per boot. The real card is first seen on the FW16.
