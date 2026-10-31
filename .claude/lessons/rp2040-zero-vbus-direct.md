# The Waveshare RP2040-Zero's 5V pad is its USB VBUS, with no diode

Anything fed into the pad appears on the module's USB-C port, so a shared power path between keyboard halves needs a
blocking diode in each direction (Waveshare wiki FAQ; K-elec worker, 2026-10-31).
