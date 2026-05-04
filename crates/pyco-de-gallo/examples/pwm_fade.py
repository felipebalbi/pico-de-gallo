#!/usr/bin/env python3
"""Smoothly fade an LED on PWM channel 0 with a sine wave.

Wiring:
    - PWM channel 0 lives on RP2350 GP12. The V1 landing board does
      NOT route GP12, so wire directly to the Pico's castellated GP12
      pad.
    - LED anode -> 330 Ω resistor -> GP12.
    - LED cathode -> GND.

Press Ctrl+C to exit.

Run:
    python examples/pwm_fade.py
"""

import math
import time

import pyco_de_gallo

CHANNEL = 0
FREQUENCY_HZ = 1000
UPDATE_HZ = 60
CYCLE_S = 3.0


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    gallo.pwm_set_config(CHANNEL, FREQUENCY_HZ, phase_correct=False)
    gallo.pwm_enable(CHANNEL)

    info = gallo.pwm_get_duty_cycle(CHANNEL)
    max_duty = info.max_duty
    print(
        f"Fading PWM ch{CHANNEL} ({FREQUENCY_HZ} Hz, max duty {max_duty}). "
        "Press Ctrl+C to stop."
    )

    period_s = 1.0 / UPDATE_HZ
    try:
        start = time.monotonic()
        while True:
            t = time.monotonic() - start
            phase = (t / CYCLE_S) * 2.0 * math.pi
            # Map sin (-1..1) -> (0..1) -> raw compare value.
            duty = int((0.5 - 0.5 * math.cos(phase)) * max_duty)
            gallo.pwm_set_duty_cycle(CHANNEL, duty)
            time.sleep(period_s)
    except KeyboardInterrupt:
        gallo.pwm_set_duty_cycle(CHANNEL, 0)
        gallo.pwm_disable(CHANNEL)
        print("\nStopped.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
