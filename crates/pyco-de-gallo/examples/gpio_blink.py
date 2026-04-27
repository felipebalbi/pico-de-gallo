#!/usr/bin/env python3
"""Blink an LED on GPIO0 every 500 ms.

Wiring:
    - LED anode (long leg) -> 330 Ω resistor -> GPIO0 pad on the
      landing board (or directly to RP2350 GP8 on the bare Pico).
    - LED cathode (short leg) -> GND.

Press Ctrl+C to exit.

Run:
    python examples/gpio_blink.py
"""

import time

import pyco_de_gallo

PIN = 0
PERIOD_S = 0.5


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    gallo.gpio_set_config(
        PIN,
        pyco_de_gallo.GpioDirection.Output,
        pyco_de_gallo.GpioPull.Disabled,
    )

    print(f"Blinking GPIO{PIN} at {1.0 / (2 * PERIOD_S):.1f} Hz. Press Ctrl+C to stop.")
    state = False
    try:
        while True:
            state = not state
            gallo.gpio_put(PIN, state)
            time.sleep(PERIOD_S)
    except KeyboardInterrupt:
        gallo.gpio_put(PIN, False)
        print("\nStopped.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
