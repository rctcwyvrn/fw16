# QMK split on RP2040 takes one serial pin and one hand pin for both halves; only the matrix pins can differ per side

`SERIAL_USART_TX_PIN` and `SPLIT_HAND_PIN` have no per-side variant; only `MATRIX_*_PINS_RIGHT` do. Pick the serial and
hand GPIOs free and routable on both boards before assigning the matrix (keyboard: GP11 and GP0). RP2040 half-duplex
serial idles on weak pull-ups and drives edges high (`serial_vendor.c`). K-elec worker, 2026-10-31.
