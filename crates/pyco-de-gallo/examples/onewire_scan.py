#!/usr/bin/env python3
"""Discover and print the ROM IDs of every device on the 1-Wire bus.

Wiring:
    - 1-Wire data pin is on RP2350 GP16. The V1 landing board does
      NOT route GP16, so wire directly to the Pico's castellated GP16
      pad.
    - DQ pin of each 1-Wire device -> GP16.
    - 4.7 kΩ pull-up resistor between GP16 and 3V3.
    - VDD pin of each device -> 3V3 (or GND for parasitic power).
    - GND pin -> GND.

A common 1-Wire device is the DS18B20 temperature sensor. Multiple
devices can share the same bus.

Run:
    python examples/onewire_scan.py
"""

import pyco_de_gallo


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    if not gallo.onewire_reset():
        print("No 1-Wire devices detected (no presence pulse).")
        print("Check wiring and 4.7 kΩ pull-up to 3V3.")
        return 1

    print("Scanning 1-Wire bus...")
    rom = gallo.onewire_search()
    found = []
    while rom is not None:
        found.append(rom)
        # Family code is the lowest byte; serial is the next 6, CRC is top byte.
        family = rom & 0xFF
        serial = (rom >> 8) & 0xFFFF_FFFF_FFFF
        crc = (rom >> 56) & 0xFF
        print(f"  ROM 0x{rom:016x}  family=0x{family:02x}  serial=0x{serial:012x}  crc=0x{crc:02x}")
        rom = gallo.onewire_search_next()

    print(f"Total devices: {len(found)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
