# Pico de Gallo HAL AI-Agent Guide — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a single ~500–800-line markdown file at `docs/ai-agents/pico-de-gallo-hal-examples.md` that an AI coding agent fetches over HTTP and uses as a complete recipe for generating a working `pico-de-gallo-hal` example or HIL test. Add one row to `AGENTS.md` §15.1 so the file stays in lockstep with the HAL.

**Architecture:** Pure docs change. The file is built incrementally section-by-section (TL;DR → rules → decision tree → 12 peripheral subsections → worked example → checklist → drift note). After every task that adds code snippets, run the §15-acceptance grep to confirm no fabricated HAL accessors have appeared. One commit per task.

**Tech Stack:** Markdown only. Verification uses `grep`, `wc -l`, and `file`. No Rust, no build steps. LF line endings enforced by `.gitattributes`.

**Spec:** `docs/superpowers/specs/2026-06-03-pico-de-gallo-hal-agent-guide-design.md` is the source of truth for every decision. If the plan and spec disagree, the spec wins.

---

## Conventions

- **Working directory:** repo root (`/home/balbi/workspace/pico-de-gallo`) unless a step says otherwise.
- **Commit style:** Conventional Commit, scope `repo`, with the AI-agent attribution trailers from `AGENTS.md` §10:
  ```
  Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
  Assisted-by: <AGENT>:<MODEL>
  ```
  Substitute `<AGENT>:<MODEL>` with the actual agent/model running the task (verify before composing; do not guess). **Never** add `Signed-off-by:` on AI-assisted commits.
- **Per-task commit message body:** brief, ≤72-char wrap, says what and why.
- **Line endings:** LF only. After every file write, confirm with `file <path>` — output must not contain "CRLF".
- **Hard rules to honor:** AGENTS.md §4 (LF endings, never push without permission, no squash merge, AI-attribution trailers, no `Signed-off-by`, repo is `OpenDevicePartnership/pico-de-gallo`).
- **Verification grep (used in multiple tasks):**
  ```bash
  grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
      docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
  grep -oE 'pub fn [a-z_]+' \
      crates/pico-de-gallo-hal/src/lib.rs | sort -u
  ```
  Every name in the first list must appear in the second list, **or** be a method on `Gpio`, `I2c`, `Spi`, `SpiDev`, `Uart`, `PwmChannel`, `Delay`, or `OneWire` (the §11 surface table covers those).
- **Verified HAL surface** (do not invent anything beyond this — copy from spec §11):
  - `Hal::new()`, `Hal::new_with_serial_number(&str)`
  - `Hal::i2c()`, `Hal::i2c_set_config(I2cFrequency)`, `Hal::i2c_get_config()`, `Hal::i2c_scan(bool)`
  - `Hal::spi()`, `Hal::spi_device(u8) -> Result<SpiDev, SpiHalError>`, `Hal::spi_set_config(u32, SpiPhase, SpiPolarity)`, `Hal::spi_get_config()`
  - `Hal::gpio(u8) -> Gpio`, `Hal::gpio_subscribe(u8, GpioEdge)`, `Hal::gpio_unsubscribe(u8)`
  - `Hal::pwm_channel(u8) -> PwmChannel`, `Hal::pwm_set_config(u8, u32, bool)`, `Hal::pwm_get_config(u8)`
  - `Hal::adc_read(AdcChannel) -> Result<u16, AdcHalError>`, `Hal::adc_get_config()`
  - `Hal::uart() -> Uart`, `Hal::delay() -> Delay`, `Hal::onewire() -> OneWire`
  - `Gpio::set_config(GpioDirection, GpioPull)` plus blocking `OutputPin`/`InputPin`/`StatefulOutputPin` and async `Wait` trait methods (`is_high`, `is_low`, `set_high`, `set_low`, `is_set_high`, `is_set_low`, `wait_for_high`, `wait_for_low`, `wait_for_rising_edge`, `wait_for_falling_edge`, `wait_for_any_edge`).
  - `Uart::set_timeout_ms(u32)` plus `embedded_io::{Read,Write}` and `embedded_io_async::{Read,Write}` trait methods.
  - `OneWire::reset() -> Result<bool, _>`, `read(u16)`, `write(&[u8])`, `write_pullup(&[u8], u16)`, `search() -> Result<Option<u64>, _>`, `search_next()`.
  - Re-exported types from `pico_de_gallo_lib`: `GpioEvent`, `I2cFrequency`, `SpiConfigurationInfo`, `SpiPhase`, `SpiPolarity`, `UartConfigurationInfo`. **Not** re-exported: `AdcChannel`, `GpioDirection`, `GpioEdge`, `GpioPull`, `AdcConfigurationInfo`. Agent code must `use pico_de_gallo_lib::<Type>;` for those.

---

## Task 1: Create directory and frontmatter skeleton

**Files:**
- Create: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Create directory**

```bash
mkdir -p docs/ai-agents
ls docs/ai-agents
```
Expected: directory exists, empty.

- [ ] **Step 2: Write frontmatter + section headings shell**

Write `docs/ai-agents/pico-de-gallo-hal-examples.md` with this exact content (and nothing else yet):

```markdown
# Pico de Gallo HAL — AI Agent Guide

Audience: AI coding agents writing host-side examples or HIL tests that
exercise an `embedded-hal` driver against real hardware via a Pico de
Gallo USB bridge.

**Source of truth:** `crates/pico-de-gallo-hal/src/lib.rs` on `main`.
If this file contradicts that file, the source wins — file an issue at
<https://github.com/OpenDevicePartnership/pico-de-gallo/issues>.

## 1. TL;DR

<!-- filled in Task 2 -->

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
```

- [ ] **Step 3: Confirm LF line endings**

```bash
file docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: output contains `UTF-8 text` and does **not** contain `CRLF`.

- [ ] **Step 4: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): scaffold AI-agent guide for pico-de-gallo-hal

Add empty section skeleton at docs/ai-agents/pico-de-gallo-hal-examples.md.
Subsequent commits fill each section. See spec at
docs/superpowers/specs/2026-06-03-pico-de-gallo-hal-agent-guide-design.md.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 2: Fill §1 TL;DR

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md` (replace the `<!-- filled in Task 2 -->` placeholder)

- [ ] **Step 1: Replace the §1 placeholder**

Replace exactly this string:

```
## 1. TL;DR

<!-- filled in Task 2 -->
```

with:

```
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
```

- [ ] **Step 2: Confirm placeholder is gone**

```bash
grep -n "filled in Task 2" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no output (exit 1).

- [ ] **Step 3: Confirm LF endings**

```bash
file docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no `CRLF`.

- [ ] **Step 4: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill TL;DR section in HAL agent guide

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 3: Fill §2 Output-shape rule

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md` (replace `<!-- filled in Task 3 -->`)

- [ ] **Step 1: Replace the §2 placeholder**

Replace:

```
## 2. Output-shape rule (binary vs. HIL test)

<!-- filled in Task 3 -->
```

with:

```
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

\`\`\`rust
#[cfg(feature = "hil")]
#[test]
fn <chip>_<assertion>() {
    let hal = pico_de_gallo_hal::Hal::new();
    // ... assert something the user would actually want to know
    //     is true for their hardware ...
}
\`\`\`

Gate it in the driver crate's `Cargo.toml`:

\`\`\`toml
[features]
default = []
hil = ["dep:pico-de-gallo-hal"]

[dev-dependencies]
pico-de-gallo-hal = { version = "*", optional = true }
\`\`\`
```

> **Important:** the literal backticks in the example above must be
> escaped (`` \` ``) when typing into the markdown so they render
> inside the outer fenced block. In the actual file content, write
> ordinary backticks — the escape characters in this plan are only
> there so the plan itself can show the code fence. **Strip the
> backslashes when writing to the file.**

- [ ] **Step 2: Confirm placeholder is gone and backticks render correctly**

```bash
grep -n "filled in Task 3" docs/ai-agents/pico-de-gallo-hal-examples.md
grep -c '\\`' docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: first grep no output (exit 1); second grep prints `0` (no escaped backticks left in the file).

- [ ] **Step 3: Confirm LF endings**

```bash
file docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no `CRLF`.

- [ ] **Step 4: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill output-shape rule in HAL agent guide

Default to a host-side binary; switch to HIL test only on explicit
signals (test/CI keywords or named driver crate).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 4: Fill §3 Sync-vs-async rule

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md` (replace `<!-- filled in Task 4 -->`)

- [ ] **Step 1: Replace the §3 placeholder**

Replace:

```
## 3. Sync-vs-async rule

<!-- filled in Task 4 -->
```

with:

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

\`\`\`toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
\`\`\`

Do **not** add `"current_thread"` to the feature list and do **not**
override the flavor.

(Remember to strip the backslash-backticks shown in the plan — write
real backticks to the file.)
```

- [ ] **Step 2: Verify placeholder is gone**

```bash
grep -n "filled in Task 4" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no output (exit 1).

- [ ] **Step 3: Confirm the `current_thread` warning is present**

```bash
grep -c "current_thread" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: at least `3` (warning header, prose, "do not" line).

- [ ] **Step 4: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill sync-vs-async rule in HAL agent guide

Selection algorithm plus mandatory warning about current_thread tokio
runtimes panicking on block_in_place. See spec §10.3.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 5: Fill §4 Cargo setup

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md` (replace `<!-- filled in Task 5 -->`)

- [ ] **Step 1: Replace the §4 placeholder**

Replace:

```
## 4. Cargo setup

<!-- filled in Task 5 -->
```

with:

```
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
```

- [ ] **Step 2: Verify placeholder is gone**

```bash
grep -n "filled in Task 5" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no output (exit 1).

- [ ] **Step 3: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill Cargo setup in HAL agent guide

Name the crates without pinning versions; downstream skill resolves.
Document re-exports vs. types that require depending on
pico-de-gallo-lib directly (AdcChannel, GpioDirection, etc.).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 6: Fill §5 Peripheral decision tree

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md` (replace `<!-- filled in Task 6 -->`)

- [ ] **Step 1: Replace the §5 placeholder**

Replace:

```
## 5. Peripheral decision tree

<!-- filled in Task 6 -->
```

with:

```
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
```

- [ ] **Step 2: Verify**

```bash
grep -n "filled in Task 6" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no output.

- [ ] **Step 3: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill peripheral decision tree in HAL agent guide

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 7: Fill §6 — replace the bulk placeholder with subsection scaffolding, then fill §6.1 (I²C, Deep)

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the §6 placeholder with subsection scaffolding**

Replace:

```
## 6. Per-peripheral reference

<!-- filled in Tasks 7–11 -->
```

with:

```
## 6. Per-peripheral reference

### 6.1 I²C

<!-- filled in Task 7 -->

### 6.2 SPI bus (no CS)

<!-- filled in Task 11 -->

### 6.3 SPI device (with CS)

<!-- filled in Task 8 -->

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
```

- [ ] **Step 2: Replace the §6.1 placeholder with the full I²C subsection**

Replace:

```
### 6.1 I²C

<!-- filled in Task 7 -->
```

with:

```
### 6.1 I²C

**When to use:** the device is on an I²C bus and uses 7-bit addressing.

**HAL accessor:** `hal.i2c()` → returns `I2c`.

**Traits implemented:** `embedded_hal::i2c::I2c`,
`embedded_hal_async::i2c::I2c`.

#### Snippet — binary form

\`\`\`rust
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
\`\`\`

#### Snippet — HIL-test form

\`\`\`rust
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
\`\`\`

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
```

(Strip the `\`` escapes; write real backticks.)

- [ ] **Step 3: Run the verification grep**

```bash
grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
    docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
```
Expected: every name should be one of `hal.i2c(`, `hal.i2c_set_config(`, `hal.i2c_get_config(`, `hal.i2c_scan(` — all of which match `pub fn` entries in `crates/pico-de-gallo-hal/src/lib.rs`. Confirm by:

```bash
grep -oE 'pub fn [a-z_]+' crates/pico-de-gallo-hal/src/lib.rs | sort -u
```

- [ ] **Step 4: Confirm LF endings**

```bash
file docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no `CRLF`.

- [ ] **Step 5: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): scaffold §6 and fill I²C subsection in HAL agent guide

§6 now contains placeholders for all 12 peripheral subsections.
§6.1 (I²C, Deep tier) is fully populated: when-to-use, accessor,
traits, binary snippet, HIL-test snippet, gotchas, config knobs.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 8: Fill §6.3 SPI device with CS (Deep)

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the §6.3 placeholder**

Replace:

```
### 6.3 SPI device (with CS)

<!-- filled in Task 8 -->
```

with:

```
### 6.3 SPI device (with CS)

**When to use:** the device is on an SPI bus and you provide a GPIO
pin as chip-select. This is the common case — most drivers expect an
`SpiDevice` that wraps CS handling for them.

**HAL accessor:** `hal.spi_device(cs_pin)` → returns
`Result<SpiDev, SpiHalError>`. CS pin range: `0..=3`.

**Traits implemented:** `embedded_hal::spi::SpiDevice`,
`embedded_hal_async::spi::SpiDevice`.

#### Snippet — binary form

\`\`\`rust
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
\`\`\`

#### Snippet — HIL-test form

\`\`\`rust
#[cfg(feature = "hil")]
#[test]
fn driver_who_am_i_matches_datasheet() {
    let hal = pico_de_gallo_hal::Hal::new();
    let spi = hal.spi_device(0).unwrap();
    let mut driver = <DriverCrate>::new(spi);
    assert_eq!(driver.who_am_i().unwrap(), 0xAB);
}
\`\`\`

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
```

- [ ] **Step 2: Verification grep**

```bash
grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
    docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
```
Expected: each new name (`hal.spi_device(`, `hal.spi_set_config(`, `hal.spi_get_config(`) appears as `pub fn` in `crates/pico-de-gallo-hal/src/lib.rs`.

- [ ] **Step 3: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill §6.3 SPI device subsection

Covers spi_device(cs_pin), CS pin range (0..=3), cancellation-safety
note, TransferInPlace allocation note, and the spi_set_config /
spi_get_config knobs. See spec §10.6.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 9: Fill GPIO pin-state-machine table + §§6.4–6.6 (output, input, async wait — Deep)

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the GPIO pin-state-machine placeholder**

Replace:

```
### GPIO subsections (§§6.4–6.7) — read first

<!-- pin-state machine table filled in Task 9 -->
```

with:

```
### GPIO subsections (§§6.4–6.7) — read first

**Pin range:** `0..=3` (firmware exposes four pins). Any pin number
outside this range returns `GpioError::InvalidPin`.

**Pin state machine** — at any moment a pin is in exactly one of
these states, and only the listed ops are valid:

| State              | Reached by                            | Allowed ops                                                    |
|--------------------|---------------------------------------|----------------------------------------------------------------|
| Unconfigured       | freshly-booted firmware               | `Gpio::set_config(...)` only                                   |
| Output             | `set_config(GpioDirection::Output,_)` | `OutputPin::set_low/high`, `StatefulOutputPin::is_set_low/high`|
| Input              | `set_config(GpioDirection::Input,_)`  | `InputPin::is_low/high`, async `Wait::wait_for_*`              |
| Subscribed (Input) | `hal.gpio_subscribe(pin, edge)`       | receive `GpioEvent`s; **no other ops** until `gpio_unsubscribe`|

Cross-state transitions: call `set_config(...)` again to flip
direction; call `gpio_unsubscribe(pin)` to leave Subscribed. **A pin
that is Subscribed cannot also be wait()'d on or read/written**: the
firmware returns `GpioError::PinMonitored`.

`use pico_de_gallo_lib::{GpioDirection, GpioPull, GpioEdge};` —
these are not re-exported by the HAL.
```

- [ ] **Step 2: Replace the §6.4 placeholder (GPIO output)**

Replace:

```
### 6.4 GPIO output

<!-- filled in Task 9 -->
```

with:

```
### 6.4 GPIO output

**When to use:** drive a line high or low — LED, relay, reset line,
manual CS pin (only if you're **not** using `hal.spi_device(cs)`).

**HAL accessor:** `hal.gpio(pin)` → returns `Gpio`. Pin range
`0..=3`.

**Traits implemented:** `embedded_hal::digital::OutputPin`,
`embedded_hal::digital::StatefulOutputPin`.

#### Snippet — binary form

\`\`\`rust
// examples/blink.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   sync (reason: trivial blocking loop)
//   peripherals:  gpio(0)
//   hal version:  <crate version observed at generation time>

use embedded_hal::digital::OutputPin;
use pico_de_gallo_hal::Hal;
use pico_de_gallo_lib::{GpioDirection, GpioPull};
use std::time::Duration;

fn main() {
    let hal = Hal::new();
    let mut gpio = hal.gpio(0);
    gpio.set_config(GpioDirection::Output, GpioPull::None).unwrap();

    loop {
        gpio.set_high().unwrap();
        std::thread::sleep(Duration::from_secs(1));
        gpio.set_low().unwrap();
        std::thread::sleep(Duration::from_secs(1));
    }
}
\`\`\`

#### Gotchas

- Pin range `0..=3`.
- Call `set_config(Output, _)` before driving the pin — pins boot
  unconfigured.
- The CS pin you pass to `hal.spi_device(cs)` is **not** usable here.
  Don't share pin numbers.
- Async usage: see §3 for the mandatory `current_thread` warning.

#### Config knobs

- `Gpio::set_config(GpioDirection, GpioPull)` — direction + internal
  pull resistor. Variants: `GpioDirection::{Input, Output}`,
  `GpioPull::{None, Up, Down}`. Default after firmware boot: input,
  no pull.
```

- [ ] **Step 3: Replace the §6.5 placeholder (GPIO input)**

Replace:

```
### 6.5 GPIO input

<!-- filled in Task 9 -->
```

with:

```
### 6.5 GPIO input

**When to use:** poll a digital line — a switch, a status pin, a
strap.

**HAL accessor:** `hal.gpio(pin)` → returns `Gpio`. Pin range
`0..=3`.

**Traits implemented:** `embedded_hal::digital::InputPin`.

#### Snippet — binary form

\`\`\`rust
// examples/<chip>.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   sync (reason: polled, no edge wait)
//   peripherals:  gpio(1)
//   hal version:  <crate version observed at generation time>

use embedded_hal::digital::InputPin;
use pico_de_gallo_hal::Hal;
use pico_de_gallo_lib::{GpioDirection, GpioPull};

fn main() {
    let hal = Hal::new();
    let mut button = hal.gpio(1);
    button.set_config(GpioDirection::Input, GpioPull::Up).unwrap();

    if button.is_low().unwrap() {
        println!("button pressed");
    }
}
\`\`\`

#### Snippet — HIL-test form

\`\`\`rust
#[cfg(feature = "hil")]
#[test]
fn strap_pin_reads_low_with_pullup() {
    use embedded_hal::digital::InputPin;
    use pico_de_gallo_lib::{GpioDirection, GpioPull};
    let hal = pico_de_gallo_hal::Hal::new();
    let mut pin = hal.gpio(2);
    pin.set_config(GpioDirection::Input, GpioPull::Up).unwrap();
    // Strapped low → pull-up loses, pin reads low.
    assert!(pin.is_low().unwrap());
}
\`\`\`

#### Gotchas

- Pin range `0..=3`.
- Default pull is `None`. For a button-to-ground use
  `GpioPull::Up`; for a button-to-VCC use `GpioPull::Down`.
- For edge-triggered waiting use §6.6; for streamed events §6.7.
- Async usage: see §3 for the mandatory `current_thread` warning.
```

- [ ] **Step 4: Replace the §6.6 placeholder (GPIO async wait)**

Replace:

```
### 6.6 GPIO async wait

<!-- filled in Task 9 -->
```

with:

```
### 6.6 GPIO async wait

**When to use:** block the current async task until a GPIO edge or
level happens. Common for button presses, IRQ lines from sensors,
SPI BUSY/READY pins. **This subsection is the only reason most
examples need to be async.**

**HAL accessor:** `hal.gpio(pin)` → returns `Gpio`. Pin range
`0..=3`.

**Traits implemented:** `embedded_hal_async::digital::Wait` —
methods `wait_for_high`, `wait_for_low`, `wait_for_rising_edge`,
`wait_for_falling_edge`, `wait_for_any_edge`.

#### Snippet — binary form

\`\`\`rust
// examples/button.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   async (reason: GPIO Wait trait is async-only)
//   peripherals:  gpio(0)
//   hal version:  <crate version observed at generation time>

use embedded_hal_async::digital::Wait;
use pico_de_gallo_hal::Hal;
use pico_de_gallo_lib::{GpioDirection, GpioPull};

#[tokio::main]               // multi-thread — DO NOT use current_thread
async fn main() {
    let hal = Hal::new();
    let mut button = hal.gpio(0);
    button.set_config(GpioDirection::Input, GpioPull::Up).unwrap();

    loop {
        button.wait_for_falling_edge().await.unwrap();
        println!("pressed");
        button.wait_for_rising_edge().await.unwrap();
        println!("released");
    }
}
\`\`\`

#### Gotchas

- Pin range `0..=3`.
- The pin must be configured as Input first. The default firmware
  state is unconfigured; call `set_config(Input, _)`.
- The pin **must not** also be Subscribed (§6.7) — pick one
  mechanism per pin.
- `#[tokio::main]` only — see §3 for the mandatory `current_thread`
  warning.
```

- [ ] **Step 5: Verification grep**

```bash
grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
    docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
```
Expected: any new entries (`hal.gpio(`, `hal.gpio_subscribe(`, `hal.gpio_unsubscribe(`) appear as `pub fn` in `crates/pico-de-gallo-hal/src/lib.rs`.

- [ ] **Step 6: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill GPIO state machine and §§6.4-6.6

GPIO output/input/async-wait subsections with pin-state machine
table at the top of the GPIO area surfacing the Subscribed-vs-other
mutual exclusion (spec §10.2).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 10: Fill §6.7 GPIO subscribe (Deep)

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the §6.7 placeholder**

Replace:

```
### 6.7 GPIO subscribe (push events)

<!-- filled in Task 10 -->
```

with:

```
### 6.7 GPIO subscribe (push events)

**When to use:** the application needs a stream of edge events from
a pin (rotary encoders, repeated button events) without polling and
without holding an async `wait_for_*` future per edge.

**HAL accessors:**

- `hal.gpio_subscribe(pin, edge)` → `Result<(), GpioHalError>` —
  start firmware-side monitoring. `edge` is `GpioEdge::Rising`,
  `Falling`, or `Any`.
- `hal.gpio_unsubscribe(pin)` → `Result<(), GpioHalError>` — stop
  monitoring and return the pin to ordinary use.

Push events arrive on the `"gpio/event"` topic as `GpioEvent` values.
**Receiving them from the HAL is not currently exposed as a typed
stream**; for end-to-end push handling you may need to drop down to
`pico_de_gallo_lib::PicoDeGallo` directly.

**Traits implemented:** n/a (project-specific).

#### Snippet — binary form

\`\`\`rust
// examples/<chip>.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   sync (reason: subscribe/unsubscribe are blocking calls)
//   peripherals:  gpio_subscribe(2)
//   hal version:  <crate version observed at generation time>

use pico_de_gallo_hal::Hal;
use pico_de_gallo_lib::{GpioDirection, GpioEdge, GpioPull};

fn main() {
    let hal = Hal::new();

    // Configure the pin as Input first.
    let mut pin = hal.gpio(2);
    pin.set_config(GpioDirection::Input, GpioPull::Up).unwrap();
    drop(pin);  // release the Gpio handle so the subscription owns it

    hal.gpio_subscribe(2, GpioEdge::Falling).unwrap();
    // Consume GpioEvents via pico_de_gallo_lib::PicoDeGallo if needed.
    // ... eventually:
    hal.gpio_unsubscribe(2).unwrap();
}
\`\`\`

#### Gotchas

- Pin range `0..=3`.
- While subscribed, the pin **cannot** be `is_low`/`is_high`/`set_*`/`wait_for_*`'d.
  Those calls return `GpioError::PinMonitored`. Call
  `hal.gpio_unsubscribe(pin)` first.
- If a previous host process died while holding a subscription,
  subsequent GPIO operations on that pin fail with `PinMonitored`
  until the firmware is reset. **The HAL does not expose a recovery
  method.** Recover by power-cycling the board, or by depending on
  `pico-de-gallo-lib` directly and calling
  `PicoDeGallo::system_reset_subscriptions().await`.
- The HAL does not currently surface push events as a stream. For
  full producer/consumer behavior you'll need
  `pico-de-gallo-lib` directly.
```

- [ ] **Step 2: Verification grep**

```bash
grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
    docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
```
Expected: `hal.gpio_subscribe(`, `hal.gpio_unsubscribe(` both match `pub fn` entries.

- [ ] **Step 3: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill §6.7 GPIO subscribe subsection

Documents subscribe/unsubscribe, the PinMonitored failure mode after
a host crash, and the HAL's lack of a recovery accessor (spec §10.2).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 11: Fill §§6.2, 6.8–6.12 (SPI bus, PWM, ADC, 1-Wire, UART, Delay — Medium + Stub)

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

This task fills six remaining subsections. Three are Medium (SPI bus, ADC, 1-Wire) and three are Stub (PWM, UART, Delay), per spec §9.

- [ ] **Step 1: Replace §6.2 (SPI bus, Medium)**

Replace:

```
### 6.2 SPI bus (no CS)

<!-- filled in Task 11 -->
```

with:

```
### 6.2 SPI bus (no CS)

**When to use:** rarely — only when the driver crate manages chip
select itself (passes its own GPIO handle, talks to a non-CS device,
or daisy-chains). Otherwise prefer §6.3.

**HAL accessor:** `hal.spi()` → returns `Spi`.

**Traits implemented:** `embedded_hal::spi::SpiBus`,
`embedded_hal_async::spi::SpiBus`.

\`\`\`rust
use embedded_hal::spi::SpiBus;
use pico_de_gallo_hal::Hal;

fn main() {
    let hal = Hal::new();
    let mut spi = hal.spi();

    let tx = [0xAA, 0x55];
    let mut rx = [0u8; 2];
    spi.transfer(&mut rx, &tx).unwrap();
    println!("rx: {:02x?}", rx);
}
\`\`\`

#### Gotchas

- No automatic CS. The bus is always live.
- Config knobs are shared with §6.3: `hal.spi_set_config(...)` and
  `hal.spi_get_config()`.
- Async usage: see §3 for the mandatory `current_thread` warning.
```

- [ ] **Step 2: Replace §6.8 (PWM, Stub)**

Replace:

```
### 6.8 PWM

<!-- filled in Task 11 -->
```

with:

```
### 6.8 PWM

**When to use:** the device needs a duty-cycle waveform — motor
ESCs, dimmable LEDs, servo control.

**HAL accessor:** `hal.pwm_channel(channel)` → returns `PwmChannel`.
Channels `0..=3` (channels 0/1 share slice 6 on GPIO 12/13, channels
2/3 share slice 7 on GPIO 14/15).

\`\`\`rust
use embedded_hal::pwm::SetDutyCycle;
use pico_de_gallo_hal::Hal;

fn main() {
    let mut hal = Hal::new();
    hal.pwm_set_config(0, 1_000, false).unwrap();  // 1 kHz, edge-aligned
    let mut ch = hal.pwm_channel(0);
    ch.set_duty_cycle_percent(50).unwrap();
}
\`\`\`

Gotchas: `pwm_set_config(channel, freq_hz, phase_correct)` affects
the whole slice — channels 0/1 cannot use different frequencies, and
neither can 2/3. Async usage: see §3 for the mandatory
`current_thread` warning. See
[`pico-de-gallo-hal` docs.rs](https://docs.rs/pico-de-gallo-hal) for
the full surface.
```

- [ ] **Step 3: Replace §6.9 (ADC, Medium)**

Replace:

```
### 6.9 ADC

<!-- filled in Task 11 -->
```

with:

```
### 6.9 ADC

**When to use:** read an analog input — potentiometer, thermistor,
battery voltage.

**HAL accessor:** `hal.adc_read(channel)` →
`Result<u16, AdcHalError>`. There is **no `embedded-hal` 1.0 ADC
trait**, so this is a project-specific method.

**Traits implemented:** n/a.

\`\`\`rust
use pico_de_gallo_hal::Hal;
use pico_de_gallo_lib::AdcChannel;  // not re-exported by the HAL

fn main() {
    let hal = Hal::new();
    let raw = hal.adc_read(AdcChannel::Adc0).unwrap();
    let volts = raw as f32 * 3.3 / 4096.0;
    println!("raw: {raw}, V≈{volts:.3}");
}
\`\`\`

#### Snippet — HIL-test form

\`\`\`rust
#[cfg(feature = "hil")]
#[test]
fn adc0_reads_in_valid_range() {
    use pico_de_gallo_lib::AdcChannel;
    let hal = pico_de_gallo_hal::Hal::new();
    let raw = hal.adc_read(AdcChannel::Adc0).unwrap();
    assert!(raw <= 4095, "12-bit ADC must be ≤ 4095, got {raw}");
}
\`\`\`

#### Gotchas

- `AdcChannel` is **not** re-exported by the HAL. Add
  `pico-de-gallo-lib` to your dependencies and
  `use pico_de_gallo_lib::AdcChannel;`.
- 12-bit raw value (`0..=4095`). Convert with `raw × 3.3 / 4096`.
- Use `hal.adc_get_config()` for resolution/reference details
  (returns `AdcConfigurationInfo`).
- Async usage: see §3 for the mandatory `current_thread` warning.
```

- [ ] **Step 4: Replace §6.10 (1-Wire, Medium)**

Replace:

```
### 6.10 1-Wire

<!-- filled in Task 11 -->
```

with:

```
### 6.10 1-Wire

**When to use:** the device is on a Dallas/Maxim 1-Wire bus — DS18B20
temperature sensor, DS2401 silicon serial number, etc.

**HAL accessor:** `hal.onewire()` → returns `OneWire`. No
`embedded-hal` 1-Wire trait exists.

**Traits implemented:** n/a.

\`\`\`rust
use pico_de_gallo_hal::Hal;

fn main() {
    let hal = Hal::new();
    let ow = hal.onewire();

    let present = ow.reset().unwrap();
    println!("device present: {present}");

    // Issue ROM-skip + convert-T to all devices on the bus.
    ow.write(&[0xCC, 0x44]).unwrap();
}
\`\`\`

Available methods: `reset()`, `read(len)`, `write(data)`,
`write_pullup(data, pullup_ms)`, `search()`, `search_next()`.

#### Snippet — HIL-test form

\`\`\`rust
#[cfg(feature = "hil")]
#[test]
fn onewire_bus_has_at_least_one_device() {
    let hal = pico_de_gallo_hal::Hal::new();
    let ow = hal.onewire();
    assert!(ow.reset().unwrap(), "no 1-Wire device responded");
}
\`\`\`

#### Gotchas

- Enumerate devices with `search()` for the first address, then
  `search_next()` until it returns `None`.
- Parasitic-power parts (DS18B20 in 2-wire mode) need
  `write_pullup(data, pullup_ms)` to hold the line high after the
  convert command.
- Async usage: see §3 for the mandatory `current_thread` warning.
```

- [ ] **Step 5: Replace §6.11 (UART, Stub)**

Replace:

```
### 6.11 UART

<!-- filled in Task 11 -->
```

with:

```
### 6.11 UART

**When to use:** the device speaks serial — a GPS module, a
debug-port-on-UART chip, AT-command modem.

**HAL accessor:** `hal.uart()` → returns `Uart`. Implements
`embedded_io::{Read,Write}` and `embedded_io_async::{Read,Write}`.

\`\`\`rust
use embedded_io::Write as _;
use pico_de_gallo_hal::Hal;

fn main() {
    let hal = Hal::new();
    let mut uart = hal.uart();
    uart.write_all(b"AT\r\n").unwrap();
}
\`\`\`

Gotchas: read uses a timeout (default 1000 ms). Set with
`uart.set_timeout_ms(0)` for non-blocking. **Baud rate is fixed at
the firmware default. The HAL does not expose a baud-rate setter** —
to change baud, depend on `pico-de-gallo-lib` and call
`PicoDeGallo::uart_set_config(...)` directly. Async usage: see §3
for the mandatory `current_thread` warning. See
[`pico-de-gallo-hal` docs.rs](https://docs.rs/pico-de-gallo-hal) for
the full surface.
```

- [ ] **Step 6: Replace §6.12 (Delay, Stub)**

Replace:

```
### 6.12 Delay

<!-- filled in Task 11 -->
```

with:

```
### 6.12 Delay

**When to use:** the driver wants `&mut impl DelayNs` for
register-settle delays, reset sequences, sensor warm-up.

**HAL accessor:** `hal.delay()` → returns `Delay`. Implements
`embedded_hal::delay::DelayNs` and
`embedded_hal_async::delay::DelayNs`.

\`\`\`rust
use pico_de_gallo_hal::Hal;

fn main() {
    let hal = Hal::new();
    let i2c = hal.i2c();
    let mut delay = hal.delay();
    let mut sensor = MyDriver::new(i2c);
    sensor.init(&mut delay).unwrap();
}
\`\`\`

Gotchas: blocking `Delay` uses `std::thread::sleep`; async `Delay`
uses `tokio::time::sleep`. Pass `&mut delay` by-mutable-reference to
driver methods. See
[`pico-de-gallo-hal` docs.rs](https://docs.rs/pico-de-gallo-hal) for
the full surface.
```

- [ ] **Step 7: Verification grep**

```bash
grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
    docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
```
Expected: every name resolves to a `pub fn` in
`crates/pico-de-gallo-hal/src/lib.rs`. Cross-check against:

```bash
grep -oE 'pub fn [a-z_]+' crates/pico-de-gallo-hal/src/lib.rs | sort -u
```

- [ ] **Step 8: Confirm no placeholders remain in §6**

```bash
grep -n "filled in Task" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: only matches in the §7, §8, §9 placeholders for the next tasks.

- [ ] **Step 9: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill §§6.2, 6.8-6.12 — SPI bus, PWM, ADC, 1-Wire, UART, Delay

Medium and Stub tiers per spec §9. ADC subsection notes AdcChannel is
not re-exported (spec §10.5). UART subsection states baud cannot be
set via the HAL (spec §10.4).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 12: Fill §7 Worked end-to-end example

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the §7 placeholder**

Replace:

```
## 7. Worked end-to-end example

<!-- filled in Task 12 -->
```

with:

```
## 7. Worked end-to-end example

A complete `examples/shtc3.rs` for the Sensirion SHTC3 temperature
+ humidity sensor. The four-line decision log at the top is
**mandatory** in every generated example — same prefixes, same
order, four lines exactly.

\`\`\`rust
// examples/shtc3.rs
// pico-de-gallo decision log:
//   shape:        binary
//   sync/async:   sync (reason: shtcx is blocking; no edge wait needed)
//   peripherals:  i2c, delay
//   hal version:  0.6

use pico_de_gallo_hal::Hal;
use shtcx::{shtc3, PowerMode};

fn main() {
    let hal = Hal::new();
    let i2c = hal.i2c();           // §6.1
    let mut delay = hal.delay();   // §6.12

    let mut sensor = shtc3(i2c);
    let m = sensor.measure(PowerMode::NormalMode, &mut delay).unwrap();

    println!(
        "{:.2} °C / {:.2} %RH",
        m.temperature.as_degrees_celsius(),
        m.humidity.as_percent(),
    );
}
\`\`\`

Decision-log format — **literal, four lines, exact prefixes**:

\`\`\`text
// pico-de-gallo decision log:
//   shape:        binary | hil-test
//   sync/async:   sync | async (reason: …)
//   peripherals:  i2c, gpio(3)
//   hal version:  <crate version observed at generation time>
\`\`\`

Free-form variants are not permitted. The fixed format is
grep-checkable by the maintainer drift-prevention hooks (see §9).
```

- [ ] **Step 2: Confirm the decision-log format block is literal**

```bash
grep -c "pico-de-gallo decision log:" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: at least `2` (once in the worked example, once in the
literal-format spec block). Many of the per-peripheral snippets also
contain it, so the count will be higher — that's fine.

- [ ] **Step 3: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill worked end-to-end example (shtc3) in HAL agent guide

Demonstrates a complete examples/<chip>.rs with the mandatory
decision-log header. Spec §10.7 format is reproduced verbatim.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 13: Fill §8 Completion checklist

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the §8 placeholder**

Replace:

```
## 8. Completion checklist

<!-- filled in Task 13 -->
```

with:

```
## 8. Completion checklist

Before declaring the example done, verify every one of these:

- [ ] The `Cargo.toml` entries from §4 are present and correct.
- [ ] The chosen output shape (binary vs HIL test) matches the
      user's intent per §2.
- [ ] The sync/async choice follows §3, and if async, the runtime is
      multi-thread (`#[tokio::main]` with **no** `current_thread`
      flavor).
- [ ] Every HAL accessor used appears in the §11-style table
      (i.e. is one of `hal.i2c`, `hal.spi`, `hal.spi_device`,
      `hal.gpio`, `hal.gpio_subscribe`, `hal.gpio_unsubscribe`,
      `hal.pwm_channel`, `hal.adc_read`, `hal.uart`, `hal.delay`,
      `hal.onewire`, or one of their associated `*_set_config`/
      `*_get_config` siblings). No invented APIs.
- [ ] Pin numbers are in range: GPIO `0..=3`, PWM channel `0..=3`,
      ADC channel is an `AdcChannel` enum variant.
- [ ] If using `spi_device(cs)`, the `cs` pin is **not** also used
      as a `Gpio` elsewhere.
- [ ] If using `gpio_subscribe(pin, …)`, a matching
      `gpio_unsubscribe(pin)` exists on every exit path.
- [ ] The mandatory four-line decision-log comment block is at the
      top of the example, with the literal prefixes from §7.
- [ ] If the host can run `cargo check` against the generated file,
      it passes.
```

- [ ] **Step 2: Verify placeholder is gone**

```bash
grep -n "filled in Task 13" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no output.

- [ ] **Step 3: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill completion checklist in HAL agent guide

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 14: Fill §9 Drift-prevention note

**Files:**
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md`

- [ ] **Step 1: Replace the §9 placeholder**

Replace:

```
## 9. Drift-prevention note (for maintainers)

<!-- filled in Task 14 -->
```

with:

```
## 9. Drift-prevention note (for maintainers)

This file documents the public surface of `pico-de-gallo-hal`. It is
listed in `AGENTS.md` §15.1 as a parity target for
`crates/pico-de-gallo-hal/src/`. Any PR that changes the HAL public
surface (accessor added/removed/renamed, gotcha discovered, trait
impl changed) must update this file in the same commit. Reviewers
treat omissions as a blocker, not a nit.

**Recommended CI guard (not yet implemented):** a regex check that
extracts every `hal\\.\\w+\\(` and `Hal::\\w+\\b` from this file and
asserts the symbol exists as a `pub fn` in
`crates/pico-de-gallo-hal/src/lib.rs`. A one-liner that catches the
common drift mode:

\`\`\`bash
diff <(grep -oE '(hal\\.[a-z_]+\\()|(Hal::[a-z_]+\\b)' \\
        docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u) \\
     <(grep -oE 'pub fn [a-z_]+' \\
        crates/pico-de-gallo-hal/src/lib.rs | sort -u)
\`\`\`

If the file disagrees with the source, the **source wins** — file an
issue at
<https://github.com/OpenDevicePartnership/pico-de-gallo/issues>.
```

- [ ] **Step 2: Verify no placeholders remain anywhere in the file**

```bash
grep -n "filled in Task" docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no output. **If any match remains, fix it before continuing.**

- [ ] **Step 3: Confirm length is within budget**

```bash
wc -l docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: between 500 and 900 lines. If under 500, sections were too
terse — re-read the spec §8 length budget and pad. If over 900,
shorten by removing redundant prose; the per-peripheral templates
must stay intact.

- [ ] **Step 4: Confirm LF endings**

```bash
file docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: no `CRLF`.

- [ ] **Step 5: Commit**

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fill drift-prevention note in HAL agent guide

Names the AGENTS.md §15.1 parity rule as the maintenance contract
and gives a one-liner regex check as the recommended CI guard.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 15: Add `AGENTS.md` §15.1 parity row

**Files:**
- Modify: `AGENTS.md` (single row added to §15.1 per-area mapping table)

- [ ] **Step 1: Read the existing §15.1 table to find the right insertion point**

```bash
grep -n "Per-area mapping" AGENTS.md
grep -n "pico-de-gallo-hal/src/" AGENTS.md
```
Expected: the §15.1 mapping table is around lines 750–770. Note the
existing row that mentions `pico-de-gallo-hal/src/...` (one row maps
HAL trait impls to `book/src/crates/hal.md`,
`book/src/driver/*`).

- [ ] **Step 2: Update that row**

The existing row reads:

```markdown
| `pico-de-gallo-hal/src/...` — trait impls                   | `book/src/crates/hal.md`, `book/src/driver/*`                            |
```

Replace it with:

```markdown
| `pico-de-gallo-hal/src/...` — trait impls                   | `book/src/crates/hal.md`, `book/src/driver/*`, `docs/ai-agents/pico-de-gallo-hal-examples.md` |
```

(Only the right-hand cell changes — append
`, docs/ai-agents/pico-de-gallo-hal-examples.md` to the existing
list. Do not change column widths beyond what the new path requires.)

- [ ] **Step 3: Verify the edit**

```bash
grep "pico-de-gallo-hal/src/" AGENTS.md
```
Expected: the row now includes `docs/ai-agents/pico-de-gallo-hal-examples.md`.

- [ ] **Step 4: Confirm LF endings**

```bash
file AGENTS.md
```
Expected: no `CRLF`.

- [ ] **Step 5: Commit**

```bash
git add AGENTS.md
git commit -m "docs(repo): add HAL agent guide to AGENTS.md §15.1 parity map

Treats docs/ai-agents/pico-de-gallo-hal-examples.md as a parity
target for crates/pico-de-gallo-hal/src/, so future HAL surface
changes must update the agent guide in the same PR.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

---

## Task 16: Final acceptance-criteria sweep

**Files:** (read-only verification; no edits unless a check fails)

- [ ] **Step 1: Acceptance criterion 1 — structure matches spec §8**

```bash
grep -nE '^## [0-9]\.' docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected exactly nine matches, in order: 1. TL;DR, 2. Output-shape
rule, 3. Sync-vs-async rule, 4. Cargo setup, 5. Peripheral decision
tree, 6. Per-peripheral reference, 7. Worked end-to-end example,
8. Completion checklist, 9. Drift-prevention note.

- [ ] **Step 2: Acceptance criterion 2 — length and EOL**

```bash
wc -l docs/ai-agents/pico-de-gallo-hal-examples.md
file docs/ai-agents/pico-de-gallo-hal-examples.md
```
Expected: line count between 500 and 900; `file` output contains
`UTF-8 text` and **not** `CRLF`.

- [ ] **Step 3: Acceptance criterion 3 — no fabricated accessors**

```bash
echo "=== names in the doc ==="
grep -oE '(hal\.[a-z_]+\()|(Hal::[a-z_]+\b)' \
    docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u
echo "=== pub fn names in the HAL ==="
grep -oE 'pub fn [a-z_]+' \
    crates/pico-de-gallo-hal/src/lib.rs | sort -u
```
Expected: every entry in the first list is one of: a `pub fn` in the
second list **or** a method on one of the surface types listed in
the conventions block at the top of this plan (`Gpio`, `I2c`, `Spi`,
`SpiDev`, `Uart`, `PwmChannel`, `Delay`, `OneWire`). Inspect by eye.

- [ ] **Step 4: Acceptance criterion 4 — snippets are syntactically valid Rust**

For every fenced Rust block in the file, mentally (or by extraction
+ `rustc --edition 2024 -`) confirm it would parse if dropped into
a fresh crate with the correct deps. Common pitfalls to look for:

- Missing `use` for trait methods (`embedded_hal::digital::OutputPin`
  for `set_high`, etc.).
- Imports of types that aren't re-exported by the HAL
  (`GpioDirection`, `GpioPull`, `GpioEdge`, `AdcChannel`).
- Async snippets using a `current_thread` runtime (must not appear).

If any snippet fails this check, fix it in a follow-up commit:

```bash
git add docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "docs(repo): fix syntax in HAL agent guide snippet (<which>)

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: <AGENT>:<MODEL>"
```

- [ ] **Step 5: Acceptance criterion 5 — AGENTS.md updated**

```bash
grep "docs/ai-agents/pico-de-gallo-hal-examples.md" AGENTS.md
```
Expected: at least one match (the row appended in Task 15).

- [ ] **Step 6: Acceptance criterion 6 — commit shape**

```bash
git --no-pager log --oneline | head -20
```
Expected: a sequence of 14–16 commits (Tasks 1–15 plus any fix-up
from Step 4 above), all on the current branch, **all** with
Conventional Commit headers using scope `repo` and bodies that
include the `Co-authored-by: Copilot …` and `Assisted-by: …`
trailers. **None** carry `Signed-off-by:`.

Verify trailers on every commit in the series:

```bash
git --no-pager log --format='%H %s%n%b%n---' | head -200
```
Read through and confirm each commit has both trailers and no DCO
sign-off.

- [ ] **Step 7: Push only after the user says so**

Do **not** push. Per `AGENTS.md` §4 hard rule #8, pushing requires
explicit user permission. Report back with:

```bash
git --no-pager log --oneline | head -20
wc -l docs/ai-agents/pico-de-gallo-hal-examples.md
```

and a one-line note that all acceptance criteria pass, then wait
for the user to decide whether to push or open a PR.
