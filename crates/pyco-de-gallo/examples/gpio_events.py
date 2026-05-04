#!/usr/bin/env python3
"""Watch GPIO edges from a Pico de Gallo device.

Configures GPIO pin 8 as an input with pull-up, subscribes to falling
edges (e.g. a button pressed to ground), and prints each event as it
arrives. Press Ctrl+C to exit.

Wiring:
    - Connect a momentary push-button between GPIO8 and GND.
    - The internal pull-up holds the line high; pressing the button
      pulls it low and produces a falling edge.

Run:
    python examples/gpio_events.py
"""

from pyco_de_gallo import list_devices, open_with_serial_number, GpioDirection, GpioPull, GpioEdge

PIN = 0


def main() -> int:
    devices = list_devices()
    if not devices:
        print("No Pico de Gallo device found.")
        return 1

    # If only one device is present, pyco_de_gallo.open() is enough.
    gallo = open_with_serial_number(devices[0].serial_number)
    gallo.validate()

    gallo.gpio_set_config(
        PIN,
        GpioDirection.Input,
        GpioPull.Up,
    )

    print(f"Watching GPIO{PIN} for falling edges. Press Ctrl+C to quit.")

    with gallo.subscribe_gpio_events(depth=16) as events:
        gallo.gpio_subscribe(PIN, GpioEdge.Falling)
        try:
            for event in events:
                print(
                    f"pin={event.pin} edge={event.edge} "
                    f"state={'high' if event.state else 'low'} "
                    f"t={event.timestamp_us} us"
                )
        except KeyboardInterrupt:
            print("\nStopping.")
        finally:
            gallo.gpio_unsubscribe(PIN)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
