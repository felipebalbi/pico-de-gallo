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

<!-- filled in Task 5 -->

## 5. Peripheral decision tree

<!-- filled in Task 6 -->

## 6. Per-peripheral reference

<!-- filled in Tasks 7–11 -->

## 7. Worked end-to-end example

<!-- filled in Task 12 -->

## 8. Completion checklist

<!-- filled in Task 13 -->

## 9. Drift-prevention note (for maintainers)

<!-- filled in Task 14 -->
