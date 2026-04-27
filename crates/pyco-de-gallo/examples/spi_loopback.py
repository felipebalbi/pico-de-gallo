#!/usr/bin/env python3
"""Send bytes over SPI in a hardware loopback and verify they echo back.

Wiring:
    - Short MOSI (GP7) directly to MISO (GP4). No CS, CLK, or external
      device is required: the firmware drives CLK and we read whatever
      it shifts in, which — with MOSI tied to MISO — is exactly what
      we shifted out.
    - The V1 landing board exposes MOSI, MISO and CLK on the SPI
      header.

Run:
    python examples/spi_loopback.py
"""

import pyco_de_gallo

MESSAGE = b"Hello, Pico de Gallo!"


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    gallo.spi_set_config(
        frequency_hz=1_000_000,
        spi_phase=pyco_de_gallo.SpiPhase.CaptureOnFirstTransition,
        spi_polarity=pyco_de_gallo.SpiPolarity.IdleLow,
    )

    print(f"Transferring {len(MESSAGE)} bytes: {MESSAGE!r}")
    received = bytes(gallo.spi_transfer(list(MESSAGE)))
    print(f"Received: {received!r}")

    if received == MESSAGE:
        print("Loopback OK ✔")
        return 0

    print("Loopback MISMATCH ✘ — check that MOSI (GP7) is shorted to MISO (GP4).")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
