# fw16kb

Split 42-key Choc keyboard for the Framework Laptop 16 (layout B, Corne `LAYOUT_split_3x6_3` matrix), one
Waveshare RP2040-Zero per half, half-duplex serial over the USB-C link's D+.

* Keyboard maintainer: [rctcwyvrn](https://github.com/rctcwyvrn)
* Hardware: `keyboard/` in [rctcwyvrn/fw16](https://github.com/rctcwyvrn/fw16)

Built out of tree; see `keyboard/firmware/README.md` in that repo. Bootloader: hold BOOT while plugging the half's
own USB-C in, hold the outer top key while plugging in (bootmagic), or press `QK_BOOT` on the ADJUST layer.
