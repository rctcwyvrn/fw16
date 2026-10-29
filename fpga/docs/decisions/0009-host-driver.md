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
- Path 3 must fix a VID:DID for the XDMA IP and agree on it (goes in the regmap v0.2 proposal).
