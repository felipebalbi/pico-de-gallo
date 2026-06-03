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

<!-- filled in Task 3 -->

## 3. Sync-vs-async rule

<!-- filled in Task 4 -->

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
