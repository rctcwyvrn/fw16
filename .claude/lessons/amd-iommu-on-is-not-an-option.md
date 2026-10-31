# `amd_iommu=on` is not a Linux option; it is ignored ("Unknown option"), and the AMD IOMMU is on by default with IVRS

Linux 7.3 `amd_iommu=` takes only fullflush, force_enable, off, force_isolation, pgtbl_v1, pgtbl_v2, irtcachedis,
nohugepages and v2_pgsizes_only (`drivers/iommu/amd/init.c:3781`). Use `iommu=pt` and leave `amd_iommu` unset.
Found by the F4-sw worker from the pinned kernel source, 2026-10-31; the driver module dropped it.
