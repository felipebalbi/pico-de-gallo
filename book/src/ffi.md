# FFI / C Bindings

## Overview

The [`pico-de-gallo-ffi`](https://docs.rs/pico-de-gallo-ffi) crate provides
C-compatible bindings for all Pico de Gallo functionality. It wraps
[`pico-de-gallo-lib`](https://docs.rs/pico-de-gallo-lib) in a C-compatible API
using **opaque pointers** and **integer status codes**.

Key design decisions:

- **Opaque handle** — C code never sees the internal layout of `PicoDeGallo`.
  All interaction goes through `gallo_*` functions.
- **cbindgen** — a C header (`pico_de_gallo.h`) is generated automatically at
  build time. You never need to write or maintain the header by hand.
- **Thread safety** — the context handle is `Send + Sync`. Each function call
  creates its own async executor via `futures::executor::block_on`, so
  concurrent calls from different threads are safe.
- **Shared library** — the crate compiles as a `cdylib`, producing
  `libpico_de_gallo_ffi.so` (Linux), `pico_de_gallo_ffi.dll` (Windows), or
  `libpico_de_gallo_ffi.dylib` (macOS).

## Lifecycle

Every program using the FFI follows the same three-step pattern:

1. **Create** a device handle.
2. **Use** `gallo_*` functions, passing the handle.
3. **Free** the handle when done.

```c
#include "pico_de_gallo.h"

/* 1. Connect to the first Pico de Gallo device */
const PicoDeGallo *gallo = gallo_init();

/* 2. Use it */
uint32_t id = 42;
Status s = gallo_ping(gallo, &id);

/* 3. Release */
gallo_free(gallo);
```

### Initialisation Functions

| Function | Description |
|---|---|
| `const PicoDeGallo *gallo_init(void)` | Connect to the first available device. Returns an opaque pointer, or `NULL` on failure. |
| `const PicoDeGallo *gallo_init_with_serial_number(const char *serial)` | Connect to a device with a specific serial number. `serial` must be a null-terminated UTF-8 string. Returns `NULL` if the serial is invalid or no matching device is found. |
| `void gallo_free(const PicoDeGallo *gallo)` | Release the device handle created by `gallo_init` or `gallo_init_with_serial_number`. Passing `NULL` is a safe no-op. |

## Status Codes

Every `gallo_*` function (except the lifecycle functions above) returns a
`Status` value. `Status` is a C `enum` backed by `int32_t`.

- **`Ok` (0)** — success.
- **Negative values** — errors, grouped by peripheral.

### Complete Status Table

| Name | Value | Description |
|---|--:|---|
| `Ok` | 0 | Operation successful |
| `I2cReadFailed` | −1 | I2C read failed |
| `I2cWriteFailed` | −2 | I2C write failed |
| `InvalidResponse` | −3 | Firmware produced an invalid response |
| `Uninitialized` | −4 | Library was not initialised (NULL context) |
| `InvalidArgument` | −5 | Caller passed an invalid argument |
| `PingFailed` | −6 | Ping round-trip failed |
| `SpiReadFailed` | −7 | SPI read failed |
| `SpiWriteFailed` | −8 | SPI write failed |
| `SpiFlushFailed` | −9 | SPI flush failed |
| `GpioGetFailed` | −10 | GPIO get failed |
| `GpioPutFailed` | −11 | GPIO put failed |
| `GpioWaitFailed` | −12 | GPIO wait failed |
| `SetConfigFailed` | −13 | Set config failed |
| `VersionFailed` | −14 | Version query failed |
| `I2cWriteReadFailed` | −15 | I2C write-read failed |
| `I2cSetConfigFailed` | −16 | I2C set config failed |
| `SpiSetConfigFailed` | −17 | SPI set config failed |
| `I2cNack` | −18 | I2C target did not acknowledge |
| `I2cBusError` | −19 | I2C bus error |
| `I2cArbitrationLoss` | −20 | I2C arbitration loss |
| `I2cOverrun` | −21 | I2C data overrun |
| `BufferTooLong` | −22 | Buffer exceeds firmware transfer limit |
| `I2cAddressOutOfRange` | −23 | I2C address out of valid range |
| `GpioInvalidPin` | −24 | Invalid GPIO pin number |
| `CommsFailed` | −25 | USB communication failure |
| `I2cScanFailed` | −26 | I2C bus scan failed |
| `GpioSetConfigFailed` | −27 | GPIO set config failed |
| `GpioWrongDirection` | −28 | GPIO pin direction mismatch |
| `I2cGetConfigFailed` | −29 | I2C get config failed |
| `SpiGetConfigFailed` | −30 | SPI get config failed |
| `UartReadFailed` | −31 | UART read failed |
| `UartWriteFailed` | −32 | UART write failed |
| `UartFlushFailed` | −33 | UART flush failed |
| `UartOverrun` | −34 | UART receiver overrun |
| `UartBreak` | −35 | UART break condition |
| `UartParity` | −36 | UART parity error |
| `UartFraming` | −37 | UART framing error |
| `UartInvalidBaudRate` | −38 | Invalid baud rate |
| `UartSetConfigFailed` | −39 | UART set config failed |
| `UartGetConfigFailed` | −40 | UART get config failed |
| `PwmSetDutyCycleFailed` | −41 | PWM set duty cycle failed |
| `PwmGetDutyCycleFailed` | −42 | PWM get duty cycle failed |
| `PwmEnableFailed` | −43 | PWM enable failed |
| `PwmDisableFailed` | −44 | PWM disable failed |
| `PwmSetConfigFailed` | −45 | PWM set config failed |
| `PwmGetConfigFailed` | −46 | PWM get config failed |
| `PwmInvalidChannel` | −47 | Invalid PWM channel |
| `PwmInvalidDutyCycle` | −48 | Invalid PWM duty cycle |
| `PwmInvalidConfiguration` | −49 | Invalid PWM configuration |
| `AdcReadFailed` | −50 | ADC read failed |
| `AdcGetConfigFailed` | −51 | ADC get config failed |
| `AdcConversionFailed` | −52 | ADC conversion error |
| `GpioPinMonitored` | −53 | Pin is currently subscribed |
| `GpioPinNotMonitored` | −54 | Pin is not subscribed |
| `GpioSubscribeFailed` | −55 | GPIO subscribe failed |
| `GpioUnsubscribeFailed` | −56 | GPIO unsubscribe failed |
| `OneWireNoPresence` | −57 | 1-Wire: no device responded to reset |
| `OneWireBusError` | −58 | 1-Wire: bus communication error |
| `OneWireReadFailed` | −59 | 1-Wire: read failed |
| `OneWireWriteFailed` | −60 | 1-Wire: write failed |
| `OneWireSearchFailed` | −61 | 1-Wire: ROM search failed |
| `CaptureInvalidPin` | −62 | Capture: pin outside valid range |
| `CaptureInvalidRate` | −63 | Capture: invalid sample rate |
| `CaptureTooManyChannels` | −64 | Capture: too many channels |
| `CaptureAlreadyRunning` | −65 | Capture: session already running |
| `CaptureNotRunning` | −66 | Capture: no session running |
| `CaptureNoPins` | −67 | Capture: no pins specified |

## Function Reference

All functions below take the opaque `PicoDeGallo *gallo` as their first
argument. Output values are written through pointer parameters. The return
value is always `Status`.

### Ping

```c
Status gallo_ping(PicoDeGallo *gallo, uint32_t *id);
```

Round-trips `*id` through the firmware. On success `*id` contains the echoed
value. Useful for verifying connectivity.

### Version

```c
Status gallo_version(PicoDeGallo *gallo,
                     uint16_t *major, uint16_t *minor, uint32_t *patch);
```

Queries the firmware version and writes the semver components to the output
pointers.

### I2C

```c
Status gallo_i2c_read(PicoDeGallo *gallo,
                      uint8_t address, uint8_t *buf, size_t len);

Status gallo_i2c_write(PicoDeGallo *gallo,
                       uint8_t address, const uint8_t *buf, size_t len);

Status gallo_i2c_write_read(PicoDeGallo *gallo,
                            uint8_t address,
                            const uint8_t *txbuf, size_t txlen,
                            uint8_t *rxbuf, size_t rxlen);

Status gallo_i2c_scan(PicoDeGallo *gallo,
                      bool include_reserved,
                      uint8_t *buf, size_t buf_len, size_t *found);

Status gallo_i2c_set_config(PicoDeGallo *gallo, uint8_t frequency);

Status gallo_i2c_get_config(PicoDeGallo *gallo, uint8_t *out_frequency);
```

**I2C frequency values:** `0` = Standard 100 kHz, `1` = Fast 400 kHz,
`2` = Fast+ 1 MHz.

`gallo_i2c_scan` probes the bus and fills `buf` with the addresses that ACK.
When `include_reserved` is `false`, only addresses 0x08–0x77 are probed.
`*found` always reflects the total device count even if `buf_len` is smaller.

### SPI

```c
Status gallo_spi_read(PicoDeGallo *gallo, uint8_t *buf, size_t len);

Status gallo_spi_write(PicoDeGallo *gallo, const uint8_t *buf, size_t len);

Status gallo_spi_flush(PicoDeGallo *gallo);

Status gallo_spi_set_config(PicoDeGallo *gallo,
                            uint32_t frequency,
                            bool spi_phase, bool spi_polarity);

Status gallo_spi_get_config(PicoDeGallo *gallo,
                            uint32_t *out_frequency,
                            bool *out_phase, bool *out_polarity);
```

**SPI phase/polarity:** `spi_phase` — `false` = CPHA=0 (capture on first
transition), `true` = CPHA=1. `spi_polarity` — `false` = CPOL=0 (idle low),
`true` = CPOL=1 (idle high).

### GPIO

```c
Status gallo_gpio_get(PicoDeGallo *gallo, uint8_t pin, bool *state);

Status gallo_gpio_put(PicoDeGallo *gallo, uint8_t pin, bool state);

Status gallo_gpio_wait_for_high(PicoDeGallo *gallo, uint8_t pin);
Status gallo_gpio_wait_for_low(PicoDeGallo *gallo, uint8_t pin);
Status gallo_gpio_wait_for_rising_edge(PicoDeGallo *gallo, uint8_t pin);
Status gallo_gpio_wait_for_falling_edge(PicoDeGallo *gallo, uint8_t pin);
Status gallo_gpio_wait_for_any_edge(PicoDeGallo *gallo, uint8_t pin);

Status gallo_gpio_set_config(PicoDeGallo *gallo,
                             uint8_t pin, uint8_t direction, uint8_t pull);

Status gallo_gpio_subscribe(PicoDeGallo *gallo, uint8_t pin, uint8_t edge);

Status gallo_gpio_unsubscribe(PicoDeGallo *gallo, uint8_t pin);
```

**Direction values:** `0` = Input, `1` = Output.

**Pull values:** `0` = None, `1` = Pull-up, `2` = Pull-down.

**Edge values (subscribe):** `0` = Rising, `1` = Falling, `2` = Any.

After calling `gallo_gpio_set_config`, the pin enters explicit-direction mode.
Calling `gallo_gpio_put` on an input pin (or `gallo_gpio_get`/wait on an
output pin) returns `GpioWrongDirection`.

While a pin is subscribed, other GPIO operations on that pin return
`GpioPinMonitored`. Call `gallo_gpio_unsubscribe` to release it.

### UART

```c
Status gallo_uart_read(PicoDeGallo *gallo,
                       uint8_t *buf, uint16_t count,
                       uint32_t timeout_ms, uint16_t *out_len);

Status gallo_uart_write(PicoDeGallo *gallo,
                        const uint8_t *buf, uint16_t len);

Status gallo_uart_flush(PicoDeGallo *gallo);

Status gallo_uart_set_config(PicoDeGallo *gallo, uint32_t baud_rate);

Status gallo_uart_get_config(PicoDeGallo *gallo, uint32_t *out_baud_rate);
```

`gallo_uart_read` reads up to `count` bytes. If no data arrives within
`timeout_ms` milliseconds, `*out_len` is set to `0` and the function returns
`Ok`. Use `gallo_uart_flush` to block until all pending bytes have been
transmitted on the wire.

### PWM

```c
Status gallo_pwm_set_duty_cycle(PicoDeGallo *gallo,
                                uint8_t channel, uint16_t duty);

Status gallo_pwm_get_duty_cycle(PicoDeGallo *gallo,
                                uint8_t channel,
                                uint16_t *out_duty, uint16_t *out_max_duty);

Status gallo_pwm_enable(PicoDeGallo *gallo, uint8_t channel);

Status gallo_pwm_disable(PicoDeGallo *gallo, uint8_t channel);

Status gallo_pwm_set_config(PicoDeGallo *gallo,
                            uint8_t channel,
                            uint32_t frequency_hz, bool phase_correct);

Status gallo_pwm_get_config(PicoDeGallo *gallo,
                            uint8_t channel,
                            uint32_t *out_frequency_hz,
                            bool *out_phase_correct, bool *out_enabled);
```

Channels 0–3 are available. Channels 0–1 share one hardware PWM slice;
channels 2–3 share another. `duty` is the raw compare value (0 to `top`).
Use `gallo_pwm_get_duty_cycle` to discover `out_max_duty` (= top + 1).

### ADC

```c
Status gallo_adc_read(PicoDeGallo *gallo,
                      uint8_t channel, uint16_t *out_value);

Status gallo_adc_get_config(PicoDeGallo *gallo,
                            uint8_t *out_resolution_bits,
                            uint16_t *out_nominal_reference_mv,
                            uint8_t *out_num_gpio_channels);
```

`channel` selects the ADC input: 0–3 map to GPIO 26–29. The raw 12-bit value
(0–4095) is written to `*out_value`.

### 1-Wire

```c
Status gallo_onewire_reset(PicoDeGallo *gallo, bool *out_present);

Status gallo_onewire_read(PicoDeGallo *gallo,
                          uint8_t *buf, uint16_t len, uint16_t *out_len);

Status gallo_onewire_write(PicoDeGallo *gallo,
                           const uint8_t *buf, uint16_t len);

Status gallo_onewire_write_pullup(PicoDeGallo *gallo,
                                  const uint8_t *buf, uint16_t len,
                                  uint16_t pullup_duration_ms);

Status gallo_onewire_search(PicoDeGallo *gallo,
                            uint64_t *out_rom_ids, uint16_t max_count,
                            uint16_t *out_count);
```

`gallo_onewire_reset` issues a bus reset; `*out_present` indicates whether any
device responded with a presence pulse.

`gallo_onewire_write_pullup` writes data and then holds the bus high for
`pullup_duration_ms` milliseconds to power parasitic-power devices.

`gallo_onewire_search` discovers up to `max_count` ROM IDs and writes them to
`out_rom_ids`. The actual count is stored in `*out_count`.

### Logic Capture

```c
typedef struct {
    uint32_t actual_sample_rate_hz;
    uint8_t  num_channels;
} GalloCaptureStartInfo;

typedef struct {
    uint64_t total_samples;
    uint64_t duration_us;
    uint32_t chunks_sent;
    uint32_t drops;
} GalloCaptureStopInfo;

Status gallo_capture_start(PicoDeGallo *gallo,
                           const uint8_t *pins, uint32_t num_pins,
                           uint32_t sample_rate_hz,
                           GalloCaptureStartInfo *out_info);

Status gallo_capture_stop(PicoDeGallo *gallo,
                          GalloCaptureStopInfo *out_info);
```

`pins` contains capture channel indices (0–3, corresponding to GPIO 8–11).
On success, `out_info->actual_sample_rate_hz` reflects the rate achieved after
clock divider quantisation. Call `gallo_capture_stop` to end the session and
retrieve statistics.

## Building and Linking

### Build the shared library

```sh
cd crates/pico-de-gallo-ffi
cargo build --release
```

This produces:

| Platform | Output |
|---|---|
| Linux | `target/release/libpico_de_gallo_ffi.so` |
| macOS | `target/release/libpico_de_gallo_ffi.dylib` |
| Windows | `target/release/pico_de_gallo_ffi.dll` + `pico_de_gallo_ffi.dll.lib` |

### Locate the generated header

The C header is generated during the build into the cargo `OUT_DIR`:

```
target/release/build/pico-de-gallo-ffi-<hash>/out/include/pico_de_gallo.h
```

Copy this header to your project's include path.

### Compile and link a C program

```sh
# Linux / macOS
gcc -o my_tool my_tool.c \
    -I/path/to/include \
    -L/path/to/target/release \
    -lpico_de_gallo_ffi

# Windows (MSVC)
cl my_tool.c /I path\to\include ^
    /link /LIBPATH:path\to\target\release pico_de_gallo_ffi.dll.lib
```

At runtime, ensure the shared library is in the library search path
(`LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS, or the same
directory as the executable on Windows).

### cbindgen configuration

The header generation is controlled by `cbindgen.toml` in the FFI crate root.
Notable settings:

- **Language:** C
- **Include guard:** `PICO_DE_GALLO_H`
- **Style:** `"both"` — generates both `typedef` and tagged-struct forms.
- **Line endings:** LF

## Complete Example

Below is a full C program that initialises the library, reads two bytes from
an I2C device at address `0x50`, prints the result, and cleans up.

```c
#include <stdio.h>
#include <stdint.h>
#include "pico_de_gallo.h"

int main(void) {
    /* Connect to the first available Pico de Gallo */
    const PicoDeGallo *gallo = gallo_init();
    if (!gallo) {
        fprintf(stderr, "Failed to connect to device\n");
        return 1;
    }

    /* Verify connectivity with a ping */
    uint32_t id = 0xDEADBEEF;
    Status s = gallo_ping((PicoDeGallo *)gallo, &id);
    if (s != Ok) {
        fprintf(stderr, "Ping failed: %d\n", s);
        gallo_free(gallo);
        return 1;
    }
    printf("Ping OK, got back: 0x%08X\n", id);

    /* Query firmware version */
    uint16_t major, minor;
    uint32_t patch;
    s = gallo_version((PicoDeGallo *)gallo, &major, &minor, &patch);
    if (s == Ok) {
        printf("Firmware v%u.%u.%u\n", major, minor, patch);
    }

    /* Read 2 bytes from I2C address 0x50 */
    uint8_t buf[2] = {0};
    s = gallo_i2c_read((PicoDeGallo *)gallo, 0x50, buf, sizeof(buf));
    if (s != Ok) {
        fprintf(stderr, "I2C read failed: %d\n", s);
        gallo_free(gallo);
        return 1;
    }

    printf("Read: 0x%02X 0x%02X\n", buf[0], buf[1]);

    /* Clean up */
    gallo_free(gallo);
    return 0;
}
```

Compile and run:

```sh
gcc -o i2c_demo i2c_demo.c -I./include -L./lib -lpico_de_gallo_ffi
LD_LIBRARY_PATH=./lib ./i2c_demo
```
