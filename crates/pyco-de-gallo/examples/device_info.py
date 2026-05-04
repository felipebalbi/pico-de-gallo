#!/usr/bin/env python3
"""Print firmware/schema versions and capabilities of all attached devices.

No hardware wiring is required beyond plugging in the device.

Run:
    python examples/device_info.py
"""

import pyco_de_gallo

CAPABILITIES = [
    ("I2C", 1 << 0),
    ("SPI", 1 << 1),
    ("UART", 1 << 2),
    ("GPIO", 1 << 3),
    ("PWM", 1 << 4),
    ("ADC", 1 << 5),
    ("1-Wire", 1 << 6),
]


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    for desc in devices:
        print(f"USB serial:   {desc.serial_number}")
        print(f"  Manufacturer: {desc.manufacturer}")
        print(f"  Product:      {desc.product}")

        gallo = pyco_de_gallo.open_with_serial_number(desc.serial_number)
        gallo.validate()

        version = gallo.version()
        info = gallo.device_info()

        print(f"  Firmware:     v{version.major}.{version.minor}.{version.patch}")
        print(
            f"  Schema:       {info.schema_major}.{info.schema_minor}.{info.schema_patch}"
        )
        print(f"  Hardware rev: {info.hw_version}")
        print(f"  Capabilities (0x{info.capabilities:016x}):")
        for name, bit in CAPABILITIES:
            mark = "✓" if info.capabilities & bit else " "
            print(f"    [{mark}] {name}")
        print()

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
