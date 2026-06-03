# Spec: AI-Agent Guide for `pico-de-gallo-hal` Examples

- **Date:** 2026-06-03
- **Status:** Draft — pending user approval
- **Scope:** Add one new markdown file to the repository, plus a small
  amount of supporting plumbing, so that an AI coding agent can fetch
  the file over HTTP and use it as a complete recipe for generating a
  working host-side example or HIL test against `pico-de-gallo-hal`.

## 1. Goal

Produce a single markdown file in this repository,
`docs/ai-agents/pico-de-gallo-hal-examples.md`, that is the canonical
reference an AI coding agent consults whenever it is asked to write an
example program or hardware-in-the-loop test exercising an
`embedded-hal` driver against real hardware through a Pico de Gallo USB
bridge.

The file's primary consumer is a future opencode skill called
`generate-pico-de-gallo-example` (or similar). That skill:

- Independently knows Rust, `cargo`, and how to edit `Cargo.toml`.
- Fetches the markdown file over HTTP via `webfetch`.
- Uses the file as the only source of truth for **Pico-de-Gallo-specific
  idioms, API choices, gotchas, and conventions**.

The skill is out of scope for this spec. This spec specifies only the
markdown file (and the small amount of plumbing required to keep it
from drifting from the HAL source).

## 2. Non-goals

- This file is not a Rust tutorial.
- It is not a `cargo` tutorial.
- It does not duplicate `embedded-hal` trait documentation.
- It is not a marketing or onboarding document; the existing
  `book/src/driver/` chapter covers that audience.
- It is not the skill itself.

## 3. Audience

A coding agent (LLM-driven) that has been asked to produce one of:

1. A standalone host-side binary at `examples/<chip>.rs` that runs
   against a real device via Pico de Gallo and prints/asserts
   something interesting.
2. A `#[cfg(feature = "hil")] #[test]` block inside an
   `embedded-hal` driver crate, gated behind a `hil` Cargo feature,
   that exercises the driver against real hardware.

The agent is expected to be reasonably capable but **not** expected to
read `crates/pico-de-gallo-hal/src/lib.rs` itself before generating
output. Anything not covered in the markdown file should be considered
unavailable to the agent for the purposes of producing the example.

## 4. File location and fetch URL

- **Path on disk:** `docs/ai-agents/pico-de-gallo-hal-examples.md`
- **Raw fetch URL:** `https://raw.githubusercontent.com/OpenDevicePartnership/pico-de-gallo/main/docs/ai-agents/pico-de-gallo-hal-examples.md`
- The `docs/ai-agents/` directory is new. It establishes a namespace
  for any future agent-targeted documentation (e.g. a future file for
  `pyco-de-gallo` or for the `gallo` CLI).
- The file is **not** served by mdBook. It is a standalone file
  consumed via raw GitHub content URLs. This is a deliberate choice
  so the agent fetches plain markdown without HTML chrome.
- The file is **MIT-licensed** under the repository's existing top-level
  `LICENSE`. No per-file license header.

## 5. Length budget

- Target: 400–600 lines of markdown.
- Hard ceiling: 700 lines. Above that, the agent's working memory
  starts paying a real cost per fetch.

## 6. Tone and voice

- Technical, terse, present tense.
- Matches the voice of the repository's `AGENTS.md`.
- Imperative or declarative; no second-person "you" coaching.
- No emoji.
- No marketing language.
- LF line endings (per repo-wide convention in `.gitattributes`).

## 7. Structural decisions (locked)

These decisions were settled during brainstorming and are not open for
revision in the implementation phase:

| Decision | Choice |
|----------|--------|
| File purpose | Reference + decision tree (not tutorial, not pure copy-paste). |
| Coverage | Full HAL public surface as it actually exists in `crates/pico-de-gallo-hal/src/lib.rs` (verified against the source — see §11). |
| Sync vs. async | Both documented, with an explicit selection rule. |
| Output shape | Both documented (binary and HIL test), with an explicit selection rule that has a tiebreaker. |
| File location | `docs/ai-agents/pico-de-gallo-hal-examples.md`, raw-served. |
| Internal structure | Option A: TL;DR → rules → decision tree → uniform per-peripheral subsections → worked example → checklist → drift note. |
| Depth | I²C, SPI device (with CS), and GPIO get deep treatment. UART, PWM, ADC, 1-Wire, raw SPI bus, Delay get short stubs. |
| Default execution model | Binary, blocking, unless explicit signals shift the choice. |
| Drift prevention | Add one row to `AGENTS.md` §15.1 mapping table; recommend a one-line CI grep check; full automation is future work. |

## 8. File structure (top-level)

The file is organized into nine numbered sections in this order:

1. **TL;DR** (~10–15 lines)
2. **Output-shape rule** — binary vs. HIL test (~15–25 lines)
3. **Sync-vs-async rule** (~15–25 lines), including a mandatory warning
   about `current_thread` tokio runtimes (see §10.3 below).
4. **Cargo setup** (~10 lines) — pared down to a short paragraph naming
   the crates involved, with no version pins. The downstream skill
   handles version resolution.
5. **Peripheral decision tree** (~20–30 lines) — flat lookup table.
6. **Per-peripheral reference** (~250–350 lines) — twelve subsections,
   uniform template, depth-weighted as described in §9.
7. **Worked end-to-end example** (~40 lines) — one fully-runnable I²C
   binary with a fixed-format decision-log header comment (see §10.7).
8. **Completion checklist** (~15 lines).
9. **Drift-prevention note for maintainers** (~10 lines).

## 9. Depth weighting

The HAL exposes twelve relevant peripheral surfaces. They are not
equally important to a driver-validation workflow. The file treats them
in three tiers:

| Tier | Peripherals | Treatment |
|------|-------------|-----------|
| Deep | I²C; SPI device (with CS); GPIO output; GPIO input; GPIO async wait; GPIO subscribe | Full template: when-to-use, accessor, traits, binary snippet, HIL-test snippet, gotchas, config knobs. |
| Medium | SPI bus (no CS); ADC; 1-Wire | Full template **except** the HIL-test snippet may be omitted if no meaningful invariant exists (see §10.4). |
| Stub | PWM; UART; Delay | One-paragraph description + one snippet + gotchas. No HIL-test snippet. Link out to `docs.rs` for more. |

Total peripheral subsections: twelve. Total subsection content:
~250–350 lines.

## 10. Authoritative content rules

These rules govern what the file says about specific HAL behaviors. They
were verified against `crates/pico-de-gallo-hal/src/lib.rs` and
`crates/pico-de-gallo-firmware/src/` during design review. Any
implementation that violates one of these rules is wrong.

### 10.1 GPIO pin range

The firmware exposes exactly **four** GPIO pins, indexed `0..=3`. This
matches `NUM_GPIOS: usize = 4` in
`crates/pico-de-gallo-firmware/src/context.rs:33`.

The file must state pin range as `0..=3` for all GPIO operations
(get, put, set_config, subscribe, wait_for_*, CS pin for `spi_device`).

> Side finding (not in scope here, file as separate issue): the
> doc-comment on `pico-de-gallo-internal/src/lib.rs` says GPIO pin
> range is `0..=7`. That comment contradicts firmware reality.

### 10.2 No `Hal::system_reset_subscriptions` and no `Hal::validate`

The HAL crate **does not** expose `system_reset_subscriptions` or
`validate`. `Hal::new` constructs a `PicoDeGallo` and wraps it in an
`Arc<Mutex<>>`; it does not call either method
(`crates/pico-de-gallo-hal/src/lib.rs:79–120`).

The file must not promise these methods. The GPIO-subscribe gotcha
must say plainly:

> If a previous host process died while holding a subscription,
> subsequent GPIO operations on that pin fail with `GpioError::PinMonitored`.
> The HAL does not expose a recovery method. Recover by power-cycling
> the board, or by depending on `pico-de-gallo-lib` directly and
> calling `PicoDeGallo::system_reset_subscriptions`.

> Side finding (not in scope): adding `Hal::system_reset_subscriptions`
> and `Hal::validate` accessors would close this gap. File as a
> separate issue.

### 10.3 Async footgun: `current_thread` runtimes panic

`Hal::in_async_context()` only checks `Handle::try_current().is_ok()`
(`lib.rs:463`). It cannot distinguish a `current_thread` runtime from
a `multi_thread` one. Every blocking trait impl then unconditionally
calls `tokio::task::block_in_place(...)` — which is documented by tokio
to panic on `current_thread` runtimes.

The sync-vs-async section (§3 of the file) must include a prominent
**mandatory warning**:

> If you pick async, you must use the **default multi-thread** tokio
> runtime: `#[tokio::main]` is correct;
> `#[tokio::main(flavor = "current_thread")]` will panic the first
> time a driver issues a blocking I²C, SPI, GPIO, UART, PWM, ADC, or
> 1-Wire call.

This warning must also be repeated as a one-line gotcha in every
peripheral subsection whose blocking trait impls call `block_in_place`:
I²C, SPI bus, SPI device, GPIO output, GPIO input, PWM, ADC, 1-Wire,
UART. (GPIO async wait uses the native async path and is not
affected.)

### 10.4 No `Hal::uart_set_config`

There is **no** `pub fn uart_set_config` on `Hal`. The only UART-side
mutator the HAL exposes is `Uart::set_timeout_ms` (`lib.rs:1299`).

The file must not promise a `Hal::uart_set_config`. The UART
subsection's gotchas must state plainly:

> Baud rate is fixed at the firmware default. The HAL does not expose
> a baud-rate setter. To change baud, depend on `pico-de-gallo-lib`
> directly and call `PicoDeGallo::uart_set_config`.

> Side finding (not in scope): the doc comment at
> `lib.rs:1282` references a `Hal::uart_set_config` that doesn't
> exist. File as a separate issue.

### 10.5 `AdcChannel` is not re-exported from the HAL

`AdcChannel` is imported in `lib.rs:47` for use in HAL function
signatures, but is not in the `pub use pico_de_gallo_lib::{...}` block
at `lib.rs:56–58`.

The ADC subsection must show the import explicitly:

```rust
use pico_de_gallo_hal::Hal;
use pico_de_gallo_lib::AdcChannel;  // not re-exported by the HAL

let raw = hal.adc_read(AdcChannel::Adc0)?;
```

> Side finding (not in scope): add `AdcChannel` to the HAL's
> `pub use` block. File as a separate issue.

### 10.6 SPI device — `TransferInPlace` allocates

`SpiDev::transaction_inner` allocates a per-operation `Vec<u8>` to
back any `Operation::TransferInPlace` so the batch encoder has a
stable reference (`lib.rs:1116, 1129, 1199, 1212`).

The SPI-device subsection's gotchas must state:

> `Operation::TransferInPlace(buf)` is supported, but the implementation
> allocates a `Vec` of the same size as `buf` for each occurrence in a
> transaction. For large in-place transfers, prefer
> `Operation::Transfer(read, write)` with separate buffers.

### 10.7 Decision-log header — fixed format

The worked-example section (§7 of the file) and the completion
checklist (§8 of the file) must both reference a single fixed
four-line header format that the agent should prepend to every
generated example:

```
// pico-de-gallo decision log:
//   shape:        binary | hil-test
//   sync/async:   sync | async (reason: …)
//   peripherals:  i2c, gpio(3)
//   hal version:  <crate version observed at generation time>
```

The format is mandatory and literal. Free-form variants are not
permitted. (Rationale: a literal format is grep-checkable by the CI
hook described in §13.2 and is consistently reproducible by a
low-thinking model.)

## 11. Verified peripheral surface

The file documents exactly these accessors and types. Each is
verified against `crates/pico-de-gallo-hal/src/lib.rs`. The
implementation phase must not add, remove, or rename any of these.

| Subsection | HAL accessor | Returned handle | Traits implemented | Tier |
|------------|--------------|-----------------|--------------------|------|
| 6.1 I²C | `hal.i2c()` | `I2c` | `embedded_hal::i2c::I2c`, `embedded_hal_async::i2c::I2c` | Deep |
| 6.2 SPI bus (no CS) | `hal.spi()` | `Spi` | `embedded_hal::spi::SpiBus`, `embedded_hal_async::spi::SpiBus` | Medium |
| 6.3 SPI device (with CS) | `hal.spi_device(cs_pin)` → `Result<SpiDev, SpiHalError>` | `SpiDev` | `embedded_hal::spi::SpiDevice`, `embedded_hal_async::spi::SpiDevice` | Deep |
| 6.4 GPIO output | `hal.gpio(pin)` | `Gpio` | `OutputPin`, `StatefulOutputPin` | Deep |
| 6.5 GPIO input | `hal.gpio(pin)` | `Gpio` | `InputPin` | Deep |
| 6.6 GPIO async wait | `hal.gpio(pin)` | `Gpio` | `embedded_hal_async::digital::Wait` | Deep |
| 6.7 GPIO subscribe | `hal.gpio_subscribe(pin, edge)` + `hal.gpio_unsubscribe(pin)` | (no handle; push-based) | n/a | Deep |
| 6.8 PWM | `hal.pwm_channel(channel)` | `PwmChannel` | `embedded_hal::pwm::SetDutyCycle` | Stub |
| 6.9 ADC | `hal.adc_read(channel)` → `Result<u16, AdcHalError>` | n/a | n/a (no embedded-hal 1.0 ADC trait) | Medium |
| 6.10 1-Wire | `hal.onewire()` | `OneWire` | n/a (no embedded-hal 1-Wire trait) | Medium |
| 6.11 UART | `hal.uart()` | `Uart` | `embedded_io::{Read,Write}`, `embedded_io_async::{Read,Write}` | Stub |
| 6.12 Delay | `hal.delay()` | `Delay` | `embedded_hal::delay::DelayNs`, `embedded_hal_async::delay::DelayNs` | Stub |

Configuration setters/getters the file must also document (in §3
or §6.x as appropriate):

- `Hal::i2c_set_config(I2cFrequency)`, `Hal::i2c_get_config()`,
  `Hal::i2c_scan(include_reserved: bool)`
- `Hal::spi_set_config(freq, phase, polarity)`, `Hal::spi_get_config()`
- `Hal::pwm_set_config(channel, freq_hz, phase_correct)`,
  `Hal::pwm_get_config(channel)`
- `Hal::adc_get_config()`
- `Uart::set_timeout_ms(u32)`
- `Hal::new_with_serial_number(&str)` for multi-board setups

## 12. Per-peripheral subsection template

Every Tier-Deep and Tier-Medium subsection in §6 must use this exact
structure:

```
### <Title>

When to use: <one sentence>.
HAL accessor: `hal.<method>(<args>)` → returns `<Type>`.
Traits implemented: <comma-separated list>, or "n/a" with reason.

#### Snippet — binary form

```rust
// examples/<chip>.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   <sync|async> (reason: <reason>)
//   peripherals:  <list>
//   hal version:  <current version>

use pico_de_gallo_hal::Hal;
// ... 5–10 lines of canonical idiomatic use, no error-handling shortcuts ...
```

#### Snippet — HIL-test form

(Omit only for Tier-Medium peripherals that have no meaningful
invariant in isolation, and for all Tier-Stub peripherals.)

```rust
#[cfg(feature = "hil")]
#[test]
fn <chip>_<assertion>() {
    let hal = pico_de_gallo_hal::Hal::new();
    // ... 5–10 lines that assert something the user would actually
    // want to know is true for their hardware ...
}
```

#### Gotchas

- <one bullet per real gotcha, 1–4 bullets per subsection>

#### Config knobs (only if non-trivial)

- `hal.<setter>(...)` — what it changes, when to call it, default value.
```

Tier-Stub subsections (PWM, UART, Delay) use a collapsed form:

```
### <Title>

When to use: <one sentence>.
HAL accessor: `hal.<method>(<args>)` → returns `<Type>`.

```rust
// minimal snippet
```

Gotchas: <inline bullets, no subheading>.

See [`pico-de-gallo-hal` docs.rs](https://docs.rs/pico-de-gallo-hal)
for the full surface.
```

## 13. Drift prevention

### 13.1 AGENTS.md §15.1 mapping row

Add one row to the per-area mapping table in `AGENTS.md` §15.1:

| Code area | Book chapter(s) |
|-----------|-----------------|
| `crates/pico-de-gallo-hal/src/...` — public API | `book/src/crates/hal.md`, `book/src/driver/*`, **`docs/ai-agents/pico-de-gallo-hal-examples.md`** |

This means any PR that changes the HAL public surface must update the
agent guide in the same PR, and reviewers (human + automated) treat
omissions as blockers under the existing parity rule.

### 13.2 Recommended CI check (described, not implemented in this spec)

A cheap regex-based CI check should grep the markdown for every
identifier of the form `hal\.\w+\(` and `Hal::\w+`, then sanity-check
each one against `grep -oE 'pub fn \w+' crates/pico-de-gallo-hal/src/lib.rs`.

Out of scope for this spec. Listed as a recommended follow-up inside
the generated file's own drift-prevention note (its §9) and as a
future issue (see §14 of this spec, item 5).

### 13.3 Header in the file itself

The file's frontmatter (top ~5 lines) names the source of truth and
states the rule:

> **Source of truth:** `crates/pico-de-gallo-hal/src/lib.rs` on `main`.
> If this file contradicts that file, the source wins — file an issue.

## 14. Out of scope

These items came up during design but are deliberately not part of
this spec. They will be filed as separate issues:

1. Add `Hal::system_reset_subscriptions` and `Hal::validate` accessors.
2. Add `AdcChannel` to the HAL's `pub use` re-export block.
3. Fix the stale `Hal::uart_set_config` reference in
   `crates/pico-de-gallo-hal/src/lib.rs:1282`.
4. Fix the `pico-de-gallo-internal` doc comment that incorrectly says
   GPIO pin range is `0..=7`.
5. Implement the §13.2 CI grep check.
6. Build the downstream `generate-pico-de-gallo-example` opencode
   skill that consumes this file.
7. Apply the same agent-guide pattern to `pyco-de-gallo` and the
   `gallo` CLI.

## 15. Acceptance criteria

The implementation phase is complete when all of the following hold:

1. `docs/ai-agents/pico-de-gallo-hal-examples.md` exists, follows the
   structure in §8, honors every authoritative content rule in §10,
   documents only the surface in §11, and uses the template in §12.
2. The file is between 400 and 700 lines, with LF line endings.
3. Every HAL accessor named in the file resolves to a real `pub fn`
   in `crates/pico-de-gallo-hal/src/lib.rs`. The implementer verifies
   this manually before opening the PR by running, from the repo root:
   ```bash
   grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
       docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
   grep -oE 'pub fn [a-z_]+' \
       crates/pico-de-gallo-hal/src/lib.rs | sort -u
   ```
   and confirming every name in the first list has a backing entry in
   the second (or is a method on `Gpio`, `I2c`, `Spi`, `SpiDev`,
   `Uart`, `PwmChannel`, `Delay`, or `OneWire`, which the §11 surface
   table covers).
4. Every snippet in the file is valid Rust syntax that would
   `cargo check` if dropped into a fresh crate with the correct
   dependencies. Snippets do not need to be standalone-compilable
   inside the markdown (no doctest harness); syntactic validity is
   sufficient.
5. `AGENTS.md` §15.1 includes the new mapping row from §13.1.
6. The file is committed in a single PR with a Conventional Commit of
   the form `docs(repo): add AI-agent guide for pico-de-gallo-hal`
   (scope is `repo` because the file lives outside any one crate).
7. CHANGELOG entries are not required; this is a docs-only change
   with no released-artifact impact.

## 16. Open questions

None. All decisions are locked.
