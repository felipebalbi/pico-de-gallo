# Spec: Category A Hotfix — Review Synthesis

- **Date:** 2026-06-03
- **Status:** Draft — basis for `2026-06-03-pico-de-gallo-category-a-hotfix.md` plan
- **Scope:** Captures findings from three parallel read-only subagent
  reviews (reviewer / reliability / architect) of the seven crates in
  `crates/` and partitions them into three categories. Specifies the
  Category A hotfix (this spec → that plan). Lists Category B and C as
  out-of-scope for the hotfix but tracked for future work.

## 1. Background

After PR #56 (the AI-agent HAL guide) shipped, we dispatched three
read-only subagents in parallel to audit the seven Rust crates under
`crates/`:

- **reviewer** (adversarial correctness) — looked for surface gaps,
  wire-protocol inconsistencies, cross-crate parity drift, test
  coverage holes, hygiene issues, book/code drift.
- **reliability** (failure modes) — looked for host crash recovery,
  USB transport failure, firmware-side panics / wedges, race
  conditions, timeouts, idempotency, observability.
- **architect** (design) — looked for module/crate boundary mistakes,
  API shape problems, error-type design, wire-protocol scalability,
  versioning model, long-term evolution.

This spec deduplicates their findings, prioritizes by severity, and
groups them by what work each finding implies.

## 2. Findings (43 items, prioritized)

### 2.1 Critical bugs — Category A (fix before next release)

| # | Finding | Source | Severity |
|---|---------|--------|----------|
| 1 | `validate()` ignores `schema_major` — only compares minor. Today the minor is the pre-1.0 breaking-change axis so the check works; the moment `internal` hits 1.0 or someone bumps major by mistake, validation silently accepts a mismatched device. | reviewer B1: `crates/pico-de-gallo-lib/src/lib.rs:667` | Blocker for 1.0 |
| 2 | GPIO `wait_for_*` wedges the entire firmware dispatcher. Host crash + `gpio_wait_for_high` on a pin that never transitions = device unusable until power-cycle. Worse than the original `system/reset-subscriptions` regression because it bricks the whole device. | reliability B1: `crates/pico-de-gallo-firmware/src/handlers/gpio.rs:86–143` | Blocker — field reproducible |
| 3 | No watchdog in firmware. Any handler hang (B2 above, I²C stuck bus, 1-Wire PIO stall) requires power-cycle to recover. | reliability R5: `crates/pico-de-gallo-firmware/src/main.rs:354–468` | Blocker — defense-in-depth |

### 2.2 Real bugs — Category A (fix before next release if practical)

| # | Finding | Source | Effort |
|---|---------|--------|--------|
| 4 | Schema validation opt-in everywhere except FFI `gallo_get_device_info`. `Hal::new`, `gallo_init`, `pyco::open`, `app::Cli::connect` all skip `validate()`. | reliability R4 | S |
| 5 | HAL doesn't expose `system_reset_subscriptions` or `validate`. (PR #56 side finding #1; reliability R2.) | known | S |
| 6 | `AdcChannel`, `GpioDirection`, `GpioPull`, `GpioEdge`, `AdcConfigurationInfo` not re-exported by `pico-de-gallo-hal`. (PR #56 side finding #2.) | known | XS |
| 7 | `gpio_subscribe_handler` capacity-1 channel can wedge dispatcher. Same shape as B1, requires multiple things to go wrong but it exists. | reliability S6 | S |
| 8 | I²C stuck-bus has no recovery. Slave holds SDA after host crash → every future I²C op fails until power-cycle. Need 9-SCL bit-bang recovery or new `i2c/recover` endpoint. | reliability B3 | M |
| 9 | FFI error mappers route every `*::Other` to the *read* failure code regardless of which operation was called. Dedicated `*GetConfigFailed`/`*WriteFailed` codes exist but are unreachable. | reviewer S1: `crates/pico-de-gallo-ffi/src/lib.rs:223, 231, 242, 255, 268, 276, 287` | S |
| 10 | `spi_batch_handler` mutates GPIO direction without updating `pin_modes` — silent hardware/state divergence. | reliability S2: `crates/pico-de-gallo-firmware/src/handlers/spi.rs:138` | S |
| 11 | `pico-de-gallo-ffi` has no GPIO event subscription read surface. Codes exist; subscribe/unsubscribe exist; but no `gallo_gpio_poll_event` or iter. | reviewer S2 | M |
| 12 | `pyco-de-gallo` ADC binding takes raw `u8` instead of `AdcChannel` enum. Bypasses Python type system. | reviewer S5: `crates/pyco-de-gallo/src/lib.rs:1287` | XS |
| 13 | Stale doc reference to `Hal::uart_set_config` at `crates/pico-de-gallo-hal/src/lib.rs:1282`. (PR #56 side finding #3.) | known | XS |
| 14 | `pico-de-gallo-internal` GPIO range docs inconsistent — some say `0..=7` (wrong), some say `0..=3` (right). | reviewer S3 + PR #56 side finding #4: `crates/pico-de-gallo-internal/src/lib.rs:470, 482, 510, 547, 586` | XS |
| 15 | Book endpoint catalog (`book/src/appendix/endpoints.md` lines 77–88) describes `gpio/event` topic wrong — implies one stream per pin, actually one multiplexed stream. | reviewer S6 | XS |

### 2.3 Test / hygiene additions — Category A

| # | Finding | Source | Effort |
|---|---------|--------|--------|
| 34 | FFI `Status` enum has no discriminant-uniqueness test. Copy-paste during enum extension could collide. | reviewer B2 | XS |
| 35 | `crates/pico-de-gallo-internal/build.rs` schema constants have no test asserting they match `[package].version`. Stale incremental cache could ship a mismatched build. | reviewer S8 | XS |
| 36 | CI grep guard for the PR #56 agent-guide markdown not yet implemented. (PR #56 side finding #5.) | known | XS |

### 2.4 Architectural commitments — Category B (decide before 1.0)

These are not part of the Category A hotfix. Listed for tracking and
for the post-hotfix architectural discussion.

| # | Decision | Source | Cost if deferred |
|---|----------|--------|-------------------|
| 16 | Freeze `internal` enum-ordering ABI with explicit discriminants on every wire enum + golden-file postcard snapshot tests. Today enforced only by comments. | architect 1.1 | Major break post-1.0 |
| 17 | `PicoDeGallo` is a flat god-object (50+ methods). Split into per-peripheral handles so HAL is mostly mechanical and there's a place for cached session state. | architect 1.2 | Major break post-1.0 |
| 18 | FFI `Status` enum is sequential-by-insertion-order, not grouped. Reserve ranges per peripheral now, while still 0.x. | architect 1.6 + reviewer S1 | Major C ABI break post-1.0 |
| 19 | Add `gpio_pin_count`, `pwm_channel_count`, `adc_channel_count`, `i2c_bus_count`, `spi_bus_count` to `DeviceInfo` now. Host currently assumes 4/4/4/1/1. | architect 1.8 | Major break post-1.0 |
| 20 | Add optional `bus: u8` (default 0) to every peripheral request type now, before 1.0. Adding a second I²C/SPI bus later means duplicating every endpoint. | architect 3.1 | Major wire break post-1.0 |
| 21 | `Comms(String)` in seven HAL error types loses error chains. Carry the typed `HostErr<WireError>` instead. | architect 1.5 | Locked into stringly-typed errors |
| 22 | HAL pin ownership is runtime-enforced (`GpioError::PinMonitored`). Compile-time pin ownership tokens + typed pin direction would close the foot-gun. | architect 1.3 + 1.4 | One more outage of the same class |
| 23 | `Hal::new()` `.unwrap()` on `Runtime::new()` + silent multi-runtime creation. Should return `Result`, detect `current_thread` flavor, accept an existing `Handle`. | architect 1.7 | API break, but worse if it ships into 1.0 |

### 2.5 Sharp edges — Category C (convenience fixes, file as issues)

| # | Finding | Source | Effort |
|---|---------|--------|--------|
| 24 | `SpiDev` cancellation-safety docstring is wrong. | reliability S1: `crates/pico-de-gallo-hal/src/lib.rs:1086–1091` | XS |
| 25 | No `system/reset-all` endpoint. Only `system/reset-subscriptions` exists; pin direction / PWM enable / configs survive across host crashes. | reliability R1 | M |
| 26 | `gallo_init` / `pyco::open` / `app::Cli::connect` are lazy — no enumeration check. | reliability R3 | S |
| 27 | `gpio_unsubscribe` race — event for unsubscribed pin can arrive after `Ok(())`. Doc only. | reliability S3 | XS |
| 28 | GPIO event publish failures silent. Add `warn!` on drop. | reliability S4 | XS |
| 29 | USB OTP-read failure → serial = `"0000000000000000"`. Collision risk. | reliability S5 | XS |
| 30 | `nusb` claims interface exclusively. Document "one host per device". | reliability S8 | XS doc-only |
| 31 | `onewire_search` silently restarts mid-enumeration if called instead of `search_next`. | reliability S9 | XS |
| 32 | PWM compare scaling truncates silently when reconfiguring frequency. 100%-on becomes 99.x%-on. | reliability S10: `crates/pico-de-gallo-firmware/src/handlers/pwm.rs:148–158` | S |
| 33 | `i2c_scan` blocks dispatcher for entire scan. **Becomes Category A** as a prereq for the watchdog (finding #3). | reliability B2 | S |
| 37 | `pico-de-gallo-driver-tests` crate that runs canonical drivers as HIL `#[test]`s gated by env var. | architect 2.5 | M (post-1.0) |
| 38 | `pyco-de-gallo` has no Rust-side tests. | known | M |
| 39 | `pico-de-gallo-app` magic `4` for event channel size. | reviewer S7: `crates/pico-de-gallo-app/src/lib.rs:1015` | XS |
| 40 | AGENTS.md is 800+ lines. Split into top-level + per-crate. | architect 3.6 | M (process work) |
| 41 | (Done in PR #56) — Hal::new async usage warning. | known | done |
| 42 | `Capabilities::contains(NONE)` always returns true — add doctest. | reviewer S4 | XS |
| 43 | (Will be done in Cat A) — Add gotcha rows to AGENTS.md §13 for B1, R4, B3. | reliability rec | XS |

## 3. Things verified clean (no work needed)

The reviewer and reliability subagents independently verified these
are **not** problems:

- Enum-variant ordering in `internal` (no reorderings vs. wire-index
  expectations).
- Endpoint catalog in `internal` matches AGENTS.md §6.3 and
  `book/src/appendix/endpoints.md` line-for-line.
- `Cargo.lock` present and committed for both workspaces.
- Pinned-deps rationale (`embassy-usb-driver = "=0.2.0"`) documented
  in AGENTS.md §7.2.
- All seven crates have `CHANGELOG.md`.
- 300+ wire-type round-trip tests in `internal`.
- FFI null-pointer tests for every batch / transfer / system endpoint.
- `Capabilities` round-trip + bitop tests present.
- CLI `version` correctly handles `LegacyFirmware` fallback.
- Cargo.toml metadata complete on all six published crates.
- `pyco` correctly `publish = false`.
- All seven crates pin `rust-version = "1.90"`.
- Batch handlers (`i2c_batch`, `spi_batch`) walk ops twice (validate-
  then-execute) — no partial state mutation on bad encoding.
- `map_validate_error` policy correct.
- `spi_set_config` and `uart_set_config` guard against zero
  frequency / baud.
- `pwm_channel_parts` bounds-checks — no `unwrap` reaches
  `pwm_slices`/`pwm_configs`.
- GPIO `set_config` correctly rejects monitored pins.
- No firmware NVM / persistent state — boot-clean.
- `compute_pwm_params` bounded loop, no panic paths.

## 4. Category A scope (this hotfix)

The Category A hotfix ships as **two PRs**:

### 4.1 PR 1 — wire-change lockstep release

Covers findings **#1** (validate major), **#2** (GPIO wait wedge),
**#3** (no watchdog), **#33** (i2c_scan blocks dispatcher — prereq
for the watchdog timing budget).

**Wire change:** append `timeout_ms: u32` field to all five
`gpio_wait_*` request types in `pico-de-gallo-internal`; append
`GpioError::Timeout` variant at end of enum. Bumps schema minor
0.6 → 0.7. Per AGENTS.md §6.5, requires lockstep release of
`internal`, `firmware`, and every host crate.

**Firmware changes:** wrap `gpio_wait_*` handlers in
`embassy_time::with_timeout` when `timeout_ms > 0`; enable
`embassy_rp::watchdog::Watchdog` at 2 s in a dedicated
`embassy_executor::task` that feeds periodically; fix `i2c_scan`
with per-address `with_timeout(50ms)` to bound worst-case below
the watchdog window.

**Host changes:** every host surface adds a `_with_timeout(Duration)`
overload (`lib`, `hal`, `ffi`, `app` CLI flag, `pyco`).

### 4.2 PR 2 — host-only fixes

Covers findings **#1** (validate major check), **#4** (schema
validation at all entry points), **#5** (HAL exposes
`system_reset_subscriptions` + `validate`), **#6** (HAL re-exports),
**#13** (stale doc), **#14** (GPIO range doc drift), **#15**
(book endpoints doc), **#34** (Status uniqueness test),
**#35** (schema version test), **#36** (agent-guide CI guard).

No wire change. Each crate releases independently via release-please.

### 4.3 Out of scope

- All Category B items (architectural commitments — separate plan).
- All Category C items (file as issues, batch later).
- Findings 7, 8, 10, 11, 12 (deferred to Category C unless trivial
  to fold into PR 2):
  - **#7** GPIO subscribe wedge — kept on Cat C; same fix shape as
    the watchdog covers the worst case.
  - **#8** I²C stuck bus — Cat C; requires bit-bang recovery design.
  - **#10** SPI batch pin_mode tracking — Cat C; doc-fix it in the
    same area in PR 2 if cheap.
  - **#11** FFI GPIO event read surface — Cat C; design needed.
  - **#12** pyco ADC raw u8 — Cat C; folded into a pyco hygiene PR.
  - **#9** FFI Other → ReadFailed mis-routing — Cat C; structural,
    needs the renumbering decision (#18) first.

## 5. Acceptance criteria

The Category A hotfix is complete when:

1. PR 1 is open as a draft against `OpenDevicePartnership/pico-de-gallo:main`
   from `felipebalbi:category-a-hotfix-wire`.
2. PR 2 is open as a draft against the same base from
   `felipebalbi:category-a-hotfix-host`.
3. Every finding in §2.1, §2.2, §2.3 has a closing commit in one of
   the two PRs.
4. Every commit honors AGENTS.md §4 hard rules (LF endings, both AI
   attribution trailers, no `Signed-off-by:`, lockfile pairing,
   `--locked` validation, no enum reordering).
5. CI on both PRs is green (`check`, `lockfile`, `deny`, `semver`,
   `actionlint`, `nostd`).
6. PR 1's description contains the release-operator runbook from
   the plan (six-step merge order, 60-second waits, re-run
   instructions, tag-prefix typo trap).
7. AGENTS.md §13 has two new rows logging the regressions closed
   (B1 dispatcher wedge, R4 silent schema mismatch).

## 6. Open questions

None. All decisions are locked. Discussion is in this conversation's
brainstorming and review history.
