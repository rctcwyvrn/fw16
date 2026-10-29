# The FW16 bay supplies no power at all — not even 5V_ALW/3V3_ALW — until the card presents valid ID resistors and an EEPROM descriptor

The pin names suggest "ALW" rails are always on; the EC (`lotus/src/gpu.c`) actually keeps every bay rail off
unless ID0/ID1 read a valid ADC code and an I2C EEPROM at 0x50/0x52 holds a Framework descriptor (or is
blank, which falls back to the dual-SSD config). So the EEPROM + ID resistors are spin-1 mandatory, and a
bench bring-up must emulate neither — just power the card directly. Details: `bay/README.md` § How the EC
powers the bay.
