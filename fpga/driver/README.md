# driver

No kernel module ([decision 0009](../docs/decisions/0009-host-driver.md)): the card is bound to `vfio-pci` and driven from userspace by the `omni` crate in `host/`. This directory holds the NixOS module (IOMMU on, VID:DID → `vfio-pci`, device-node permissions). `third_party/` (git-ignored) keeps AMD's `dma_ip_drivers` only as a reference for XDMA descriptor layout.
