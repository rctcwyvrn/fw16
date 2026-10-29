# 0009 — Host driver: Rust VFIO userspace, no kernel module

**Date:** 2026-10-28
**Status:** accepted

## Context

The card exposes AMD's XDMA PCIe IP (Path 3). The plan assumed AMD's out-of-tree C driver
(`Xilinx/dma_ip_drivers`, `XDMA/linux-kernel`) plus a C host library. Checked 2026-10-28 at commit
`a918c51` (2026-10-22): its readme lists kernels up to 6.8 and its version guards stop at 6.8, while the FW16
will run 7.x. The FW16 will run **NixOS**. The user prefers drivers in Rust wherever possible.

In the pinned nixpkgs (`58deea65`) rustc/cargo are 1.99 and both NixOS kernels (6.18.59, 7.3) are built with
`CONFIG_RUST=y`. Kernel 7.3's Rust bindings cover `pci::Driver`, BAR iomap, `alloc_irq_vectors`,
`dma::Coherent` and `miscdevice` (read/write/mmap/ioctl), but have no safe way to DMA into pinned user pages.

## Options

1. AMD's C module (patched for 7.x) for bring-up, a Rust VFIO backend after S5.
2. AMD's C module for bring-up, then an own Rust kernel module.
3. Own Rust kernel module from the start — copy-based DMA or unsafe raw bindings; Rust kernel APIs churn.
4. **Rust VFIO userspace driver from the start** — no kernel module; zero-copy through the IOMMU.

## Decision

Option 4. The card is bound to `vfio-pci`; the `omni` Rust crate opens the VFIO group, maps the shell's
AXI-Lite BAR and the XDMA config BAR (assignment fixed by Path 3, see the regmap v0.2 proposal), IOMMU-maps its own DMA buffers, programs the XDMA descriptor engine
itself (PG195 scatter-gather descriptors) and takes MSI-X through eventfds. The host library is a pure Rust
crate (no C ABI until a caller needs one); register constants come from `bindgen` on the generated
`host/include/omni_regs.h`, so the regmap generator stays the single source.

## Consequences

- No out-of-tree module to chase kernels with; the NixOS config only enables the IOMMU and binds the card's
  VID:DID to `vfio-pci` with device-node permissions.
- We own the XDMA descriptor engine (~800–1500 lines) and its suspend/resume handling (re-init after resume).
- S5 is the first time both the card and the driver meet real hardware — two unknowns at once, accepted
  knowingly. Mitigations before S4: the VFIO plumbing is tested in a QEMU VM against QEMU's `edu` DMA device
  (with a vIOMMU; `edu` has MSI, not MSI-X, so it proves the eventfd plumbing and MSI-X is first exercised on
  the card), and the descriptor engine is tested against an XDMA model in the mock backend. Both models
  are written from PG195 by the same hand as the driver, so they cannot catch a misreading of PG195.
- **Gate:** the bay root port must sit in its own IOMMU group (or behind ACS). If it shares a group with the
  NVMe or USB4 controller, VFIO would need the ACS-override patch (a tainted kernel) and this decision
  reopens. Checked on day one with the Dual M.2 reference (Path 4 step 1).
- IOMMU must be on on the FW16; this desktop has it off, so no VFIO test on bare metal before the laptop.
- `driver/third_party/dma_ip_drivers` stays only as a reference for descriptor layout and register offsets.
  Upstream Linux also has a clean XDMA engine, `drivers/dma/xilinx/xdma.c` + `xdma-regs.h` (~1,500 lines in
  7.3, dmaengine glue included) — a second reference. Both are GPL: read them, write ours from PG195.

## Viability check (2026-10-28)

- **Kernel:** the pinned NixOS 7.3 kernel has `VFIO=m`, `VFIO_PCI=m`, `VFIO_GROUP=y`, `VFIO_CONTAINER=y`
  (legacy `/dev/vfio/vfio`), `VFIO_DEVICE_CDEV=y` + `IOMMUFD=m` (the newer cdev/iommufd path), `AMD_IOMMU=y`.
- **Crate:** `vfio-ioctls` 0.10.0 (rust-vmm, released 2026-10-16, ~736 k downloads, Apache-2.0/BSD-3; the
  layer Cloud Hypervisor uses). `VfioContainer::new(None)` needs no KVM; it has `vfio_dma_map/unmap`
  (unsafe), `VfioDevice::enable_msix(eventfds)`, `reset`, `region_read/write` and a `VfioOps` trait covering
  both the legacy container and iommufd cdev. There is no BAR mmap helper, so BAR mmap is our own small unsafe
  `mmap` at `get_region_offset`. `pci-driver` (0.1.4, last released 2023) is unmaintained and was ruled out.
- **Precedent:** Rust userspace drivers exist — `ixy.rs` (10 GbE NIC), `vroom` (NVMe, proof of concept) —
  and VFIO is how DPDK and SPDK run in production. No public XDMA-over-VFIO driver was found; ours is the
  first we know of.
- **FW16 IOMMU groups:** a FW16 owner on the 7040 (Phoenix) board posted the bay's RTX 5070 module in **its
  own group** (GPU + its audio function only), passed through to a VM without ACS override. No report was
  found for the Ryzen AI 300 board — the gate below still stands.
- **Suspend:** in 7.3, `vfio-pci` has only runtime-PM callbacks, no system-sleep callbacks. The PCI core
  saves and restores config space (BARs, MSI-X) across suspend, but nothing tells the userspace driver. So
  `omni` must hold a logind *delay* inhibitor: on `PrepareForSleep(true)` stop DMA and release the lock, on
  `PrepareForSleep(false)` re-check `ID` and re-initialise the XDMA engine.
- **Non-root access:** the NixOS module gives the user the `/dev/vfio/<group>` node (udev) and raises
  `RLIMIT_MEMLOCK`, since `VFIO_IOMMU_MAP_DMA` pins buffer pages and counts them against it.
- Path 3 must fix a VID:DID for the XDMA IP and agree on it (goes in the regmap v0.2 proposal).
