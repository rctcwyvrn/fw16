# A udev `ATTR{driver_override}="vfio-pci"` + `drivers_probe` rule binds every unbound device that matches it

In QEMU every device carries subsystem 1af4:1100, so a subsystem-only rule also bound the host bridge, the VGA and the ISA
bridge to vfio-pci (F4-sw VM test, 2026-10-31). Match on something unique (vendor 0x10ee + subsystem 10ee:4650 for the
card) and have the test assert that nothing else ended up on vfio-pci.
