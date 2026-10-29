# host

The `omni` Rust crate (cargo workspace): a `Transport` trait with `mock`, `vfio` (the card, via the Rust VFIO userspace driver — [decision 0009](../docs/decisions/0009-host-driver.md)) and later `sim` backends, plus CLI tools (`omni-loopback`, later `omni-cyto`, `omni-bat`, `omni sync`). `include/omni_regs.h` is generated from `gateware/regmap`; the crate reads it through `bindgen`.

- `vfio:<BDF>` opens the card bound to vfio-pci: registers, pinned DMA buffers and MSI-X interrupts work; DMA transfers wait for the XDMA engine (Path 4 step 5).
- VM test of the vfio layer against QEMU's `edu` device behind a vIOMMU (from the dev shell): `bash host/vm/run-edu-check.sh [--suspend]`. Output lands in `host/target/vm/`.
