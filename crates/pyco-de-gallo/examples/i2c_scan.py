#!/usr/bin/env python3
"""Scan the I2C bus and print every responding 7-bit address.

Wiring (V1 landing board):
    - Connect any I2C device(s) to the SDA / SCL pads. Pull-ups are
      already populated on the board.
    - Bare Pico: SDA = GP2, SCL = GP3, plus 3V3 and GND. Add 4.7 kΩ
      pull-ups from SDA/SCL to 3V3 if no other pull-ups are present.

Run:
    python examples/i2c_scan.py
"""

import pyco_de_gallo


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    # If only one device is present, pyco_de_gallo.open() is enough.
    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    gallo.i2c_set_config(pyco_de_gallo.I2cFrequency.Standard)

    print("Scanning I2C bus...")
    found = gallo.i2c_scan(include_reserved=False)

    if not found:
        print("No devices responded.")
        return 0

    print(f"Found {len(found)} device(s):")
    for addr in found:
        print(f"  0x{addr:02x}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
