# Pico de Gallo HAL — AI Agent Guide

Audience: AI coding agents writing host-side examples or HIL tests that
exercise an `embedded-hal` driver against real hardware via a Pico de
Gallo USB bridge.

**Source of truth:** `crates/pico-de-gallo-hal/src/lib.rs` on `main`.
If this file contradicts that file, the source wins — file an issue at
<https://github.com/OpenDevicePartnership/pico-de-gallo/issues>.

## 1. TL;DR

You are generating either a host-side **example binary** at
`examples/<chip>.rs` or a **hardware-in-the-loop test** gated behind
`#[cfg(feature = "hil")]`. Both shapes exercise an `embedded-hal`
driver against real hardware through a Pico de Gallo USB bridge.

Decide which shape to produce (§2), pick blocking or async (§3), set
up `Cargo.toml` (§4), look up which HAL accessor your device needs
(§5 decision tree → §6 per-peripheral reference), drop a fixed-format
decision-log header at the top of the generated file (§10.7 below),
and verify against the checklist in §8.

The HAL exposes I²C, SPI, GPIO, PWM, ADC, 1-Wire, UART, and Delay.
Pin range for any GPIO is **0..=3** (the firmware only exposes four
pins). The HAL is `pico-de-gallo-hal` on crates.io.

## 2. Output-shape rule (binary vs. HIL test)

**Default: binary at `examples/<chip>.rs`.**

Pick **HIL test** (`#[cfg(feature = "hil")] #[test]` inside a driver
crate, gated behind a `hil` Cargo feature) only when **at least one**
of these is true:

- The user explicitly says "test", "validate the driver", "regression
  test", "CI", or "hardware-in-the-loop".
- The user names an existing driver crate they want regression
  coverage for.

Otherwise produce the binary. "Show me X works", "make X blink",
"read a value from X", and bring-up scripts are all binary requests.

If both shapes seem to fit, default to binary; binaries are easier to
run by hand and the user can always wrap one in a test later.

The HIL-test shape pattern is:

```rust
#[cfg(feature = "hil")]
#[test]
fn <chip>_<assertion>() {
    let hal = pico_de_gallo_hal::Hal::new();
    // ... assert something the user would actually want to know
    //     is true for their hardware ...
}
```

Gate it in the driver crate's `Cargo.toml`:

```toml
[features]
default = []
hil = ["dep:pico-de-gallo-hal"]

[dev-dependencies]
pico-de-gallo-hal = { version = "*", optional = true }
```

## 3. Sync-vs-async rule

Selection algorithm, in order:

1. If the driver crate's primary API is `async fn` → **async**
   (`#[tokio::main]`).
2. If the driver crate's primary API is blocking → **blocking**.
3. If the driver crate offers both → **blocking** (simpler).
4. If the example needs GPIO edge waits (the `Wait` trait:
   `wait_for_falling_edge`, etc.) → **async** regardless of (1)–(3).
5. If both async edge waits **and** blocking driver methods are
   needed → **async**. The HAL's `block_in_place` plumbing lets
   blocking trait calls run inside a tokio task, **but read the
   warning below first**.

> ### ⚠ Mandatory: never use `current_thread` tokio
>
> If you pick async, you MUST use the **default multi-thread**
> tokio runtime:
>
> - `#[tokio::main]` — correct.
> - `#[tokio::main(flavor = "current_thread")]` — **will panic**
>   the first time a driver issues a blocking I²C, SPI, GPIO,
>   UART, PWM, ADC, or 1-Wire call.
>
> The HAL detects "in async context" but cannot detect runtime
> flavor (see `Hal::in_async_context` in `lib.rs:463`), so it
> unconditionally calls `tokio::task::block_in_place`, which tokio
> documents as panicking on single-threaded runtimes.

Default tokio dep when async is chosen:

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
```

Do **not** add `"current_thread"` to the feature list and do **not**
override the flavor.

## 4. Cargo setup

This file does not pin versions. Resolve current versions from
crates.io when you edit `Cargo.toml`. The crates you need are:

- `pico-de-gallo-hal` — the HAL itself. Goes in `[dependencies]` for
  binaries, `[dev-dependencies]` (optional, gated by the `hil`
  feature) for HIL tests.
- `embedded-hal` — required if your driver returns
  `embedded_hal::*::Error` types.
- `embedded-hal-async` — required only if you use async traits
  (`embedded_hal_async::digital::Wait`, etc.).
- `tokio` — required only if async, with features
  `["rt-multi-thread", "macros", "time"]`. **Do not** pick
  `current_thread` — see §3.
- `pico-de-gallo-lib` — required directly when you need
  `AdcChannel`, `GpioDirection`, `GpioPull`, `GpioEdge`,
  `AdcConfigurationInfo`, or `PicoDeGallo::uart_set_config` (none
  of these are re-exported by the HAL).
- The driver crate(s) for the device you're exercising.

Re-exported by `pico_de_gallo_hal` (no extra dependency needed):
`GpioEvent`, `I2cFrequency`, `SpiConfigurationInfo`, `SpiPhase`,
`SpiPolarity`, `UartConfigurationInfo`.

## 5. Peripheral decision tree

| Your device…                                          | Jump to       |
|-------------------------------------------------------|---------------|
| Talks I²C (sensor, EEPROM, expander)                  | §6.1          |
| Talks SPI, driver manages CS itself                   | §6.2          |
| Talks SPI, you provide a CS pin (typical)             | §6.3          |
| Is an LED, relay, or reset line (drives a level)      | §6.4          |
| Is a button or switch you poll                        | §6.5          |
| Is a button you wait on (edge-triggered)              | §6.6          |
| Needs streamed edge events (rotary encoder, etc.)     | §6.7          |
| Is a motor or LED needing a duty cycle                | §6.8          |
| Is an analog input (potentiometer, thermistor)        | §6.9          |
| Is a 1-Wire device (DS18B20, DS2401)                  | §6.10         |
| Talks UART or serial                                  | §6.11         |
| The driver needs `&mut impl DelayNs`                  | §6.12         |

Most drivers combine 2–3 of these (e.g. SPI device + GPIO output for
reset + Delay). Pull each one in from its subsection independently.

## 6. Per-peripheral reference

### 6.1 I²C

**When to use:** the device is on an I²C bus and uses 7-bit addressing.

**HAL accessor:** `hal.i2c()` → returns `I2c`.

**Traits implemented:** `embedded_hal::i2c::I2c`,
`embedded_hal_async::i2c::I2c`.

#### Snippet — binary form

```rust
// examples/<chip>.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   sync (reason: driver is blocking)
//   peripherals:  i2c
//   hal version:  <crate version observed at generation time>

use embedded_hal::i2c::I2c;
use pico_de_gallo_hal::Hal;

fn main() {
    let hal = Hal::new();
    let mut i2c = hal.i2c();

    let mut buf = [0u8; 2];
    i2c.write_read(0x48, &[0x00], &mut buf).unwrap();
    println!("raw: {:02x?}", buf);
}
```

#### Snippet — HIL-test form

```rust
#[cfg(feature = "hil")]
#[test]
fn tmp102_responds_on_default_address() {
    use embedded_hal::i2c::I2c;
    let hal = pico_de_gallo_hal::Hal::new();
    let mut i2c = hal.i2c();
    let mut buf = [0u8; 2];
    // A successful write_read with an ACK proves the device is wired.
    i2c.write_read(0x48, &[0x00], &mut buf).unwrap();
}
```

#### Gotchas

- 7-bit addressing only. Pass `0x48`, not `0x90`.
- `I2c::transaction()` (and `write_read`) is batched as a single USB
  round-trip by the HAL — prefer it over separate `write` + `read`.
- Default bus frequency is 100 kHz. Bump with
  `hal.i2c_set_config(I2cFrequency::Fast)?` for 400 kHz or
  `I2cFrequency::FastPlus` for 1 MHz.
- Async usage: see §3 for the mandatory `current_thread` warning.

#### Config knobs

- `hal.i2c_set_config(I2cFrequency)` — bus frequency. Default
  `Standard` (100 kHz). Variants: `Standard`, `Fast`, `FastPlus`.
- `hal.i2c_get_config() -> Result<I2cFrequency, _>` — read current
  frequency.
- `hal.i2c_scan(include_reserved: bool) -> Result<Vec<u8>, _>` —
  scan for devices. `false` scans `0x08..=0x77`, `true` scans the
  full `0x00..=0x7F`.

### 6.2 SPI bus (no CS)

<!-- filled in Task 11 -->

### 6.3 SPI device (with CS)

**When to use:** the device is on an SPI bus and you provide a GPIO
pin as chip-select. This is the common case — most drivers expect an
`SpiDevice` that wraps CS handling for them.

**HAL accessor:** `hal.spi_device(cs_pin)` → returns
`Result<SpiDev, SpiHalError>`. CS pin range: `0..=3`.

**Traits implemented:** `embedded_hal::spi::SpiDevice`,
`embedded_hal_async::spi::SpiDevice`.

#### Snippet — binary form

```rust
// examples/<chip>.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   sync (reason: driver is blocking)
//   peripherals:  spi_device(cs=0)
//   hal version:  <crate version observed at generation time>

use pico_de_gallo_hal::Hal;
use <driver_crate>::Driver;

fn main() {
    let hal = Hal::new();
    let spi = hal.spi_device(0).expect("spi_device failed");
    let mut driver = Driver::new(spi);
    // ... use driver ...
}
```

#### Snippet — HIL-test form

```rust
#[cfg(feature = "hil")]
#[test]
fn driver_who_am_i_matches_datasheet() {
    let hal = pico_de_gallo_hal::Hal::new();
    let spi = hal.spi_device(0).unwrap();
    let mut driver = <DriverCrate>::new(spi);
    assert_eq!(driver.who_am_i().unwrap(), 0xAB);
}
```

#### Gotchas

- CS pin range is `0..=3`. The pin **must not** also be used as a
  `Gpio` elsewhere in the program (no concurrent ownership).
- The async `SpiDevice::transaction` is **not** cancellation-safe:
  if the future is dropped after CS is asserted low but before it is
  deasserted, CS stays low. Match the behavior of
  `embedded-hal-bus::ExclusiveDevice`.
- `Operation::TransferInPlace(buf)` works, but the implementation
  allocates a `Vec` the same size as `buf` per occurrence in a
  transaction. For large in-place transfers prefer
  `Operation::Transfer(read, write)` with two buffers.
- Async usage: see §3 for the mandatory `current_thread` warning.

#### Config knobs

- `hal.spi_set_config(freq_hz, SpiPhase, SpiPolarity)` — clock
  frequency, phase, polarity. Defaults: 1 MHz,
  `SpiPhase::CaptureOnFirstTransition`, `SpiPolarity::IdleLow`
  (mode 0).
- `hal.spi_get_config() -> Result<SpiConfigurationInfo, _>` — read
  back current configuration.

### GPIO subsections (§§6.4–6.7) — read first

<!-- pin-state machine table filled in Task 9 -->

### 6.4 GPIO output

<!-- filled in Task 9 -->

### 6.5 GPIO input

<!-- filled in Task 9 -->

### 6.6 GPIO async wait

<!-- filled in Task 9 -->

### 6.7 GPIO subscribe (push events)

<!-- filled in Task 10 -->

### 6.8 PWM

<!-- filled in Task 11 -->

### 6.9 ADC

<!-- filled in Task 11 -->

### 6.10 1-Wire

<!-- filled in Task 11 -->

### 6.11 UART

<!-- filled in Task 11 -->

### 6.12 Delay

<!-- filled in Task 11 -->

## 7. Worked end-to-end example

<!-- filled in Task 12 -->

## 8. Completion checklist

<!-- filled in Task 13 -->

## 9. Drift-prevention note (for maintainers)

<!-- filled in Task 14 -->
