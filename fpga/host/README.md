# host

The `omni` Rust crate (cargo workspace): a `Transport` trait with `mock`, `vfio` (the card, via the Rust VFIO userspace driver — [decision 0009](../docs/decisions/0009-host-driver.md)) and later `sim` backends, plus CLI tools (`omni-loopback`, later `omni-cyto`, `omni-bat`, `omni sync`). `include/omni_regs.h` is generated from `gateware/regmap`; the crate reads it through `bindgen`.

- `vfio:<BDF>` opens the card bound to vfio-pci: registers, pinned DMA buffers, MSI-X interrupts, and DMA through the XDMA descriptor engine (`omni/src/xdma/`, written from PG195; not yet run on hardware). `mock:` runs the same engine against an XDMA model.
- `include/omni_patterns.txt` (from `gateware/shell/bin/patterns.exe`) is checked against the Rust pattern generators by `cargo test`.
- Interrupt messages are edge-triggered on a held level: service the shell's IRQ with `omni::irq::wait_pending` / `drain` (re-read `IRQ_PENDING` until 0), not one event per message.
- VM test of the vfio layer against QEMU's `edu` device behind a vIOMMU (from the dev shell): `bash host/vm/run-edu-check.sh [--suspend]`. Output lands in `host/target/vm/`.
