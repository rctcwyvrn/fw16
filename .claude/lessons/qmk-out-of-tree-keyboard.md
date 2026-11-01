# QMK builds an out-of-tree keyboard only through a symlink into `qmk_firmware/keyboards/`

QMK's external userspace holds keymaps, not keyboard definitions, so `keyboard/firmware/build.sh` symlinks
`_build/qmk_firmware/keyboards/fw16kb` → `keyboard/firmware/fw16kb` with `QMK_HOME` set (QMK 0.34.7 compile and lint
follow it). On ChibiOS, `SPLIT_USB_DETECT` is forced on whenever `USB_VBUS_PIN` is undefined
(`platforms/chibios/chibios_config.h`), so master detection needs no config. `qmk lint` fails without a `readme.md` in the
keyboard directory. A worker shell started before a flake edit doesn't see the new tools; use `nix develop --command`.
K-fw worker, 2026-11-01.
