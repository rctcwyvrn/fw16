# A VFIO group is claimable iff no member is bound to a driver lacking `driver_managed_dma`

pcieport, pci-stub and vfio-pci set the flag, so bridges on pcieport don't block; any endpoint on a normal driver makes
`VFIO_GROUP_SET_CONTAINER` fail with EPERM. Judge the IOMMU-group gate (0009) by the members' drivers, not their PCI
class. `vfio-pci.ids=` accepts `ffffffff` (PCI_ANY_ID) as a device wildcard, not `*`, and its dynids also bind a
rescanned device (`pci-driver.c:203-213`). From the pinned 7.3 source, F4-sw worker, 2026-10-31.
