# In Hardcaml 0.18-preview, build a CDC synchroniser with `pipeline ~attributes:[Rtl_attribute.Vivado.async_reg true] spec ~n:2 x`

The attribute is emitted as `(* ASYNC_REG="TRUE" *)` on each register (checked in the emitted `fw16_fpga_shell.v`,
2026-10-31, F3-gw worker). `async_reg` applies to registers only, so attach it to the regs, not to a wire.
