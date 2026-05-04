# pyco-de-gallo examples

Runnable Python scripts that demonstrate how to drive a Pico de Gallo
device from the `pyco_de_gallo` Python module.

## Setup

From the `crates/pyco-de-gallo` directory:

```sh
python -m venv .env
. .env/bin/activate          # Windows: .env\Scripts\Activate.ps1
pip install maturin
maturin develop --release
```

This builds the native extension and installs it into the active
virtual environment.

## Running

With the device plugged in:

```sh
python examples/tmp108_read.py
```

## Examples

| Script            | Peripheral | What it does                                                                              |
|-------------------|------------|-------------------------------------------------------------------------------------------|
| `device_info.py`  | none       | Lists every attached device and prints firmware/schema versions plus a capability matrix. |
| `i2c_scan.py`     | I2C        | Scans the I2C bus and prints every responding 7-bit address.                              |
| `tmp108_read.py`  | I2C        | Reads ambient temperature from a TMP108 sensor at `0x48`.                                 |
| `tmp108_live.py`  | I2C        | Live scrolling temperature graph (Rich + braille). Needs `pip install rich`.              |
| `gpio_blink.py`   | GPIO       | Blinks an LED on GPIO0.                                                                   |
| `gpio_events.py`  | GPIO       | Subscribes to falling edges on GPIO0 and prints each event.                               |
| `pwm_fade.py`     | PWM        | Smoothly fades an LED with a sine wave on PWM channel 0.                                  |
| `adc_read.py`     | ADC        | Reads all 4 ADC channels and prints millivolts (needs hw-rev2 firmware).                  |
| `spi_loopback.py` | SPI        | Verifies SPI by short-circuiting MOSI to MISO.                                            |
| `uart_echo.py`    | UART       | Verifies UART by short-circuiting TX to RX (needs hw-rev2 firmware).                      |
| `onewire_scan.py` | 1-Wire     | Discovers ROM IDs of every device on the 1-Wire bus (needs hw-rev2 firmware).             |

Each script has a wiring comment at the top describing exactly how to
connect the parts. On the **V1 landing board**, only I2C, SPI (without
CS), GPIO 0–3, and PWM 0–3 are routed to connectors — for ADC, UART,
1-Wire, or SPI-CS you need to wire to the bare Pico's castellated
pads.
