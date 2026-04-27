#!/usr/bin/env python3
"""Send a string over UART and read it back via a hardware loopback.

Wiring:
    - Short TX (GP0) directly to RX (GP1).
    - The V1 landing board does NOT route UART, so wire on the bare
      Pico's castellated GP0 / GP1 pads.

Run:
    python examples/uart_echo.py
"""

import pyco_de_gallo

MESSAGE = b"Hello, UART!\r\n"
BAUD_RATE = 115_200
TIMEOUT_MS = 500


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    gallo.uart_set_config(BAUD_RATE)

    print(f"UART @ {BAUD_RATE} baud. Sending: {MESSAGE!r}")
    gallo.uart_write(list(MESSAGE))
    gallo.uart_flush()

    received = bytes(gallo.uart_read(len(MESSAGE), TIMEOUT_MS))
    print(f"Received: {received!r}")

    if received == MESSAGE:
        print("Loopback OK ✔")
        return 0

    print("Loopback MISMATCH ✘ — check that TX (GP0) is shorted to RX (GP1).")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
