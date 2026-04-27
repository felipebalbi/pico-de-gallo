#!/usr/bin/env python3
"""Read all four ADC channels and print millivolts.

Wiring:
    - ADC channels are on RP2350 GP26 (ADC0), GP27 (ADC1), GP28 (ADC2),
      and GP29 (ADC3). The V1 landing board does NOT route any ADC
      pin, so wire directly to the Pico's castellated pads.
    - With pins floating, the readings are noisy and meaningless.
    - For a meaningful test: connect a 10 kΩ potentiometer between
      3V3 and GND with the wiper on ADC0 (GP26). Turning the pot
      sweeps the reading from ~0 mV to ~3300 mV.

Run:
    python examples/adc_read.py
"""

import time

import pyco_de_gallo


def main() -> int:
    devices = pyco_de_gallo.list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    gallo = pyco_de_gallo.open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    cfg = gallo.adc_get_config()
    n = cfg.num_gpio_channels
    full_scale = (1 << cfg.resolution_bits) - 1
    vref_mv = cfg.nominal_reference_mv

    print(
        f"ADC: {n} channels, {cfg.resolution_bits}-bit, "
        f"vref ≈ {vref_mv} mV. Press Ctrl+C to stop."
    )
    print("ch | raw  | mV")
    print("---+------+-----")

    try:
        while True:
            for ch in range(n):
                raw = gallo.adc_read(ch)
                mv = (raw * vref_mv) // full_scale
                print(f" {ch} | {raw:4d} | {mv:4d}")
            print("---+------+-----")
            time.sleep(0.5)
    except KeyboardInterrupt:
        print("\nStopped.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
