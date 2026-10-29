# host

The `omni` Rust crate (cargo workspace): a `Transport` trait with `mock`, `vfio` (the card, via the Rust VFIO userspace driver — [decision 0009](../docs/decisions/0009-host-driver.md)) and later `sim` backends, plus CLI tools (`omni-loopback`, later `omni-cyto`, `omni-bat`, `omni sync`). `include/omni_regs.h` is generated from `gateware/regmap`; the crate reads it through `bindgen`.
