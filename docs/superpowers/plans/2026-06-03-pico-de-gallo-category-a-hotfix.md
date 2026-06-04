# Pico de Gallo — Category A Hotfix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. PRs are shipped sequentially: PR 1 first (wire change + lockstep release), then PR 2 (host-only fixes).

**Goal:** Fix the three blockers and ~15 real-bug findings the three-subagent crates review identified (synthesized in `docs/superpowers/specs/2026-06-03-pico-de-gallo-category-a-review-synthesis.md`). Ship as **two PRs** against `OpenDevicePartnership/pico-de-gallo:main`.

**Architecture:** PR 1 carries the wire change (`timeout_ms: u32` field on every `gpio_wait_*` request, `GpioError::Timeout` variant), all firmware fixes (watchdog enable, `with_timeout` wrap, `i2c_scan` per-address timeout), and the lockstep bumps across all seven crates. PR 2 carries host-only fixes (schema major-version validation, validation enforcement at all entry points, HAL `system_reset_subscriptions` + `validate` accessors, missing re-exports, doc drift, CI guards, AGENTS.md regression log).

**Tech Stack:** Rust 1.90 host, Rust + embassy-rp 0.10.0 firmware (no_std, thumbv8m.main-none-eabihf), postcard-rpc 0.12 over USB CDC, PyO3 0.28, cbindgen, release-please.

**Spec:** `docs/superpowers/specs/2026-06-03-pico-de-gallo-category-a-review-synthesis.md` captures the findings from the reviewer/reliability/architect subagents and prioritizes them into Categories A/B/C. This plan implements Category A only.

**Preflight investigations** (read-only research done before plan was written):

- **PF-1**: `embassy-rp` is pinned `=0.10.0` in firmware; `watchdog` module is **unconditionally exported** (no Cargo feature needed); `Watchdog::feed(Duration)` takes a `Duration` arg (re-arms each call); no existing watchdog code in firmware. The firmware's main loop blocks on `server.run()` — watchdog feeding **must happen in a dedicated `embassy_executor::task`**, not in RPC handlers (the wedge we're fixing means handlers cannot be trusted to feed). `book/src/internals/firmware.md` will need updating per AGENTS.md §15.1. HW-rev1/rev2 behave identically for the watchdog.
- **PF-2**: postcard-rpc 0.12 dispatch is **serial** (`Dispatch::handle` holds `&mut self` across `await`). The wedge is device-wide, not per-pin. embassy-usb-driver 0.2.0 does NOT expose `wait_disconnected`, so a `select(edge, disconnect)` fix is not viable. The `with_timeout` wrap is the only correct fix.
- **PF-3**: release-please is `separate-pull-requests: true` — seven release PRs, not one mega-PR. No `needs:` dependency graph in `release-crates.yml`; crates.io indexing race expected; operator must merge in dep order (`internal → library → {hal, ffi} → application → pyco → firmware`) and wait ~60s between waves. Tag prefixes: `library-v*` NOT `lib-v*`; `application-v*` NOT `app-v*`. `bump-minor-pre-major: true` is set, so `feat!` on 0.x bumps minor.

**Out of scope:** Categories B (pre-1.0 architectural commitments) and C (convenience fixes) from the synthesis spec. Findings 7, 9–12, 24, 26–33, 37–43.

---

## Conventions

- **Working directory:** repo root (`/home/balbi/workspace/pico-de-gallo`) unless a step says otherwise.
- **Branches:**
  - PR 1: `category-a-hotfix-wire` (off `main` — already created when this plan was committed).
  - PR 2: `category-a-hotfix-host` (off `main`, created after PR 1 is pushed; the two PRs are independent at the branch level).
- **Commit style:** Conventional Commit per AGENTS.md §10. Scopes used in this plan:
  - `internal` — `pico-de-gallo-internal` crate
  - `lib` — `pico-de-gallo-lib` crate
  - `hal` — `pico-de-gallo-hal` crate
  - `ffi` — `pico-de-gallo-ffi` crate
  - `application` — `pico-de-gallo-app` (binary name `gallo`); **NOT `app`** — release-please uses `application`
  - `pyco` — `pyco-de-gallo` crate
  - `firmware` — `pico-de-gallo-firmware` crate
  - `repo` — anything outside one crate (AGENTS.md, `.github/workflows`, `book/`)

  Multi-scope commits use comma syntax: `feat(internal,firmware): ...`.

- **Trailers (mandatory on every commit, per AGENTS.md §10):**
  ```
  Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
  Assisted-by: <AGENT>:<MODEL>
  ```
  The `Assisted-by:` value is determined by the agent dispatching the commit. For the OpenCode session that wrote this plan, the value is `opencode:claude-opus-4.7-1m-internal`. Substitute the actual model running when an implementer subagent dispatches. **Never add `Signed-off-by:` on AI-assisted commits** (AGENTS.md §4 hard rule #7).

- **Line endings:** LF only (AGENTS.md §3). After every file write, confirm with `file <path>` — output must not contain "CRLF". On Windows, run `dos2unix <path>` if needed.
- **Lockfile discipline:** every `Cargo.toml` change must be paired with the corresponding `Cargo.lock` refresh in the same commit (AGENTS.md §4 hard rule #3 + §7.1). Use `cargo update -p <crate> --locked` for targeted bumps, or `cargo update --workspace --locked` (host) and `cargo update --locked` (firmware) for broader refreshes. Always pass `--locked` when validating (AGENTS.md hard rule #4).
- **Tag-prefix gotchas (release-please):** confirmed by PF-3 — it's **`library-v*`** not `lib-v*`, and **`application-v*`** not `app-v*`. The `firmware-v*` tag does NOT publish to crates.io (artifact-only).
- **Per-crate verification commands (mirror CI):** for any crate `<crate>` you modify:
  ```bash
  cd crates/<crate>
  cargo fmt --check
  cargo clippy --all-targets --locked -- -D warnings
  cargo test --locked
  cargo hack --feature-powerset check
  cargo +1.90 check
  ```
  Firmware uses a different command per AGENTS.md §5.2 (both `hw-rev1` and `hw-rev2` builds for `thumbv8m.main-none-eabihf`).

- **Verification grep used in multiple tasks** (the "drift-check" from `docs/ai-agents/pico-de-gallo-hal-examples.md` §9, with the digit-aware regex):
  ```bash
  diff <(grep -oE '(hal\.[a-z0-9_]+\()|(Hal::[a-z0-9_]+\b)' \
          docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u) \
       <(grep -oE 'pub fn [a-z0-9_]+' \
          crates/pico-de-gallo-hal/src/lib.rs | sort -u)
  ```

---

## PR 1 Overview — Wire-change lockstep release

**Branch:** `category-a-hotfix-wire`
**Scope:** wire change to `pico-de-gallo-internal` (append `timeout_ms: u32` to five request types, append `GpioError::Timeout` variant), firmware fixes (watchdog enable, `with_timeout` wrap, `i2c_scan` fix), and lockstep host bumps so every host surface speaks the new schema.
**Schema impact:** `pico-de-gallo-internal` minor bumps (0.6 → 0.7). Per AGENTS.md §6.2 pre-1.0, minor is the breaking-change axis.
**Release-please impact:** seven release PRs will be opened after this merges. **The maintainer must merge them in the order documented in the PR description** (see §"Release operator runbook" below).

### Release operator runbook (paste verbatim into PR 1 description)

> ⚠ **Wire-change lockstep release — read before merging release-please PRs.**
>
> After this PR merges, `release-please` will open seven release PRs (one per component: `internal`, `library`, `hal`, `ffi`, `application`, `pyco`, `firmware`).
>
> **Merge them all together in this order** to avoid a partial release where a host crate on crates.io depends on an unpublished `pico-de-gallo-internal` version:
>
> 1. `internal` PR → wait ~60s for crates.io indexing
> 2. `library` PR → wait ~60s
> 3. `hal` PR and `ffi` PR (parallel OK; both only depend on `library`) → wait ~60s
> 4. `application` PR
> 5. `pyco` PR (no crates.io publish; safe anytime)
> 6. `firmware` PR (no crates.io publish; **but wire-coupled to `internal`** — must ship in this cycle)
>
> Each release PR must include a lockfile refresh before merge:
> ```bash
> cd crates && cargo update --workspace --locked
> cd pico-de-gallo-firmware && cargo update --locked
> ```
>
> If any `release-crates.yml` job fails with "no version … found in registry", **re-run it** after the upstream crate has indexed. This is documented in `release-crates.yml` lines 6–16.
>
> Tag-prefix typo trap (per AGENTS.md §13): it's `library-v*` not `lib-v*`, and `application-v*` not `app-v*`.

---

## PR 1 Task sequence

### Task 1: Append `timeout_ms: u32` to `GpioWaitHighRequest`

**Files:**
- Modify: `crates/pico-de-gallo-internal/src/lib.rs` (the `GpioWaitHighRequest` struct definition and its round-trip test)

- [ ] **Step 1: Read the existing struct definition**

```bash
grep -nA 5 "struct GpioWaitHighRequest" crates/pico-de-gallo-internal/src/lib.rs
```
Note the current field set (likely `pub pin: u8` only). The new field appends at the end.

- [ ] **Step 2: Append the field**

In `crates/pico-de-gallo-internal/src/lib.rs`, change:

```rust
pub struct GpioWaitHighRequest {
    pub pin: u8,
}
```

to:

```rust
pub struct GpioWaitHighRequest {
    pub pin: u8,
    /// Per-request timeout in milliseconds. A value of `0` means wait
    /// forever (matches pre-0.7 behavior). Any non-zero value bounds the
    /// firmware-side wait; expiry returns [`GpioError::Timeout`].
    pub timeout_ms: u32,
}
```

The doc comment is mandatory (AGENTS.md §15 — all public items must have rustdoc).

- [ ] **Step 3: Update the existing round-trip test**

Find the test:

```bash
grep -n "fn gpio_wait_high_request_round_trip" crates/pico-de-gallo-internal/src/lib.rs
```

The existing test constructs an instance with only `pin`. Update the existing test to include `timeout_ms: 0` (preserving the pre-0.7 wait-forever case), and add a second test case covering a non-zero timeout. Both go in the `#[cfg(test)] mod tests` block already present in the file:

```rust
#[test]
fn gpio_wait_high_request_round_trip_with_timeout() {
    let original = GpioWaitHighRequest { pin: 2, timeout_ms: 500 };
    let bytes = postcard::to_allocvec(&original).unwrap();
    let decoded: GpioWaitHighRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.pin, 2);
    assert_eq!(decoded.timeout_ms, 500);
}
```

- [ ] **Step 4: Run the tests**

```bash
cd crates/pico-de-gallo-internal
cargo test --features use-std gpio_wait_high_request --locked
```

Expected: both tests pass.

- [ ] **Step 5: Run full crate test suite**

```bash
cargo test --features use-std --locked
```

All ~300 existing tests must still pass.

- [ ] **Step 6: Do NOT commit yet** — T2, T3 add the same change to the other four wait request types. Commit them all together at the end of T4.

---

### Task 2: Append `timeout_ms: u32` to `GpioWaitLowRequest`

**Files:**
- Modify: `crates/pico-de-gallo-internal/src/lib.rs` (`GpioWaitLowRequest` struct + tests)

Repeat T1 steps 1–5 for `GpioWaitLowRequest`:

- [ ] **Step 1: Append field with rustdoc**

```rust
pub struct GpioWaitLowRequest {
    pub pin: u8,
    /// Per-request timeout in milliseconds. A value of `0` means wait
    /// forever (matches pre-0.7 behavior). Any non-zero value bounds the
    /// firmware-side wait; expiry returns [`GpioError::Timeout`].
    pub timeout_ms: u32,
}
```

- [ ] **Step 2: Update existing test to include `timeout_ms: 0`. Add a `_with_timeout` test:**

```rust
#[test]
fn gpio_wait_low_request_round_trip_with_timeout() {
    let original = GpioWaitLowRequest { pin: 1, timeout_ms: 250 };
    let bytes = postcard::to_allocvec(&original).unwrap();
    let decoded: GpioWaitLowRequest = postcard::from_bytes(&bytes).unwrap();
    assert_eq!(decoded.pin, 1);
    assert_eq!(decoded.timeout_ms, 250);
}
```

- [ ] **Step 3: `cargo test --features use-std gpio_wait_low --locked`** — both new and existing tests pass.
- [ ] **Step 4: Do NOT commit yet.**

---

### Task 3: Append `timeout_ms: u32` to remaining wait request types + `GpioError::Timeout` variant

**Files:**
- Modify: `crates/pico-de-gallo-internal/src/lib.rs` (`GpioWaitRisingRequest`, `GpioWaitFallingRequest`, `GpioWaitAnyRequest` structs + tests; `GpioError` enum + test)

- [ ] **Step 1: Append `timeout_ms: u32` to the three remaining request types**

For each of `GpioWaitRisingRequest`, `GpioWaitFallingRequest`, `GpioWaitAnyRequest`: append the field with the same rustdoc as T1, update existing round-trip test to include `timeout_ms: 0`, add a `_with_timeout` variant test.

- [ ] **Step 2: Append `GpioError::Timeout` variant at the end of the enum**

Find the enum:

```bash
grep -nA 30 "pub enum GpioError" crates/pico-de-gallo-internal/src/lib.rs
```

Append at the very end of the variant list (AGENTS.md §4 hard rule #2 — enum variants are append-only; postcard encodes by variant index, so any reordering or insertion is a silent wire break):

```rust
/// Returned by `gpio_wait_*` endpoints when the host-supplied
/// `timeout_ms` elapses before the requested edge or level is detected.
/// Introduced in schema 0.7. A `timeout_ms` of `0` waits forever and
/// never returns this variant.
Timeout,
```

- [ ] **Step 3: Add a round-trip test for the new variant**

```rust
#[test]
fn gpio_error_timeout_variant_round_trip() {
    let bytes = postcard::to_allocvec(&GpioError::Timeout).unwrap();
    let decoded: GpioError = postcard::from_bytes(&bytes).unwrap();
    assert!(matches!(decoded, GpioError::Timeout));
}
```

- [ ] **Step 4: Run the full crate test suite**

```bash
cd crates/pico-de-gallo-internal && cargo test --features use-std --locked
```

All ~300 tests plus the five new `_with_timeout` tests plus the timeout-variant test must pass.

- [ ] **Step 5: Do NOT commit yet.**

---

### Task 4: Bump `pico-de-gallo-internal` minor and commit T1–T4 as one logical change

**Files:**
- Modify: `crates/pico-de-gallo-internal/Cargo.toml` (version bump 0.6.0 → 0.7.0)
- Modify: `crates/Cargo.lock` (refresh)
- Modify: `crates/pico-de-gallo-internal/CHANGELOG.md` (Keep-a-Changelog format, add 0.7.0 entry)

- [ ] **Step 1: Bump version**

```bash
sed -i 's/^version = "0.6.0"$/version = "0.7.0"/' crates/pico-de-gallo-internal/Cargo.toml
grep '^version' crates/pico-de-gallo-internal/Cargo.toml
```
Expected: `version = "0.7.0"`.

- [ ] **Step 2: Refresh lockfile**

```bash
cd crates && cargo update -p pico-de-gallo-internal --locked
```

Verify only the `pico-de-gallo-internal` line in `Cargo.lock` changed:

```bash
git --no-pager diff crates/Cargo.lock | head -20
```

- [ ] **Step 3: Add CHANGELOG entry**

Inspect the existing `crates/pico-de-gallo-internal/CHANGELOG.md` to learn the convention (whether to add under `## [Unreleased]` or as a new `## [0.7.0]` section — release-please often manages the heading). Match the existing convention. The content should include:

```markdown
### Added

- `GpioWaitHighRequest`, `GpioWaitLowRequest`, `GpioWaitRisingRequest`,
  `GpioWaitFallingRequest`, `GpioWaitAnyRequest` gained a `timeout_ms: u32`
  field. A value of `0` preserves pre-0.7 wait-forever behavior; non-zero
  bounds the firmware-side wait, returning `GpioError::Timeout` on expiry.
- `GpioError::Timeout` variant (appended at end of enum; safe wire-protocol
  addition per AGENTS.md §6.1).

### Changed (BREAKING — wire protocol)

- Schema minor bumped 0.6 → 0.7 (`SCHEMA_VERSION_MINOR` derived from
  crate `[package].version` via `build.rs`). Firmware and all host crates
  must be released together in the same lockstep cycle (AGENTS.md §6.5).

### Why

- Closes the firmware-dispatcher-wedge regression described in
  `docs/superpowers/specs/2026-06-03-pico-de-gallo-category-a-review-synthesis.md`
  finding #2 / reliability subagent B1: a `gpio_wait_for_high` on a never-
  transitioning pin previously blocked **every other endpoint** on the device
  until power-cycle. Bounded waits restore liveness.
```

- [ ] **Step 4: Verify schema constants regenerate**

```bash
cd crates/pico-de-gallo-internal && cargo build --features use-std --locked
```

The `build.rs` derives `SCHEMA_VERSION_MAJOR=0`, `SCHEMA_VERSION_MINOR=7`, `SCHEMA_VERSION_PATCH=0` from the crate `[package].version`. The hard test for this lands in PR 2 Task 31; for PR 1 a successful build is the spot-check.

- [ ] **Step 5: Stage T1–T4 together and commit as one logical change**

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pico-de-gallo-internal/src/lib.rs \
        crates/pico-de-gallo-internal/Cargo.toml \
        crates/pico-de-gallo-internal/CHANGELOG.md \
        crates/Cargo.lock
git status --short
```

Expected: exactly four files staged.

- [ ] **Step 6: Commit**

```bash
git commit -m "feat(internal)!: add timeout_ms to gpio_wait_* requests, GpioError::Timeout

Append a per-request timeout_ms: u32 field to all five gpio_wait_*
request types (High, Low, Rising, Falling, Any). A value of 0 means
wait forever (matches pre-0.7 behavior); non-zero bounds the
firmware-side wait, returning the new GpioError::Timeout variant on
expiry.

Closes the dispatcher-wedge regression where a never-transitioning
pin would block every other endpoint on the device until power-cycle
(reliability review finding B1, captured in
docs/superpowers/specs/2026-06-03-pico-de-gallo-category-a-review-synthesis.md).

Postcard wire shape: append-only on the request structs (forward-
compatible additive), and append-only on the GpioError enum (variant
index-stable per AGENTS.md §6.1). Schema minor bumps 0.6 → 0.7.
Firmware and all host crates must be released together in the same
cycle per AGENTS.md §6.5.

BREAKING CHANGE: wire-protocol minor bump. Pre-0.7 firmware cannot
decode 0.7 client requests (they include 4 extra bytes per wait
request). Pre-0.7 hosts cannot decode 0.7 firmware responses that
return GpioError::Timeout (unknown enum variant). Lockstep release
required.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

The `feat(internal)!:` prefix triggers release-please's BREAKING-CHANGE treatment. The `BREAKING CHANGE:` footer in the body is redundant but adds visibility. Per AGENTS.md §13.17's 2026-05-29 entry, `bump-minor-pre-major: true` ensures this bumps 0.6 → 0.7 (minor) not 0 → 1.0.

- [ ] **Step 7: Verify**

```bash
git --no-pager log -1 --stat
```

Expected: one commit changing four files. Both trailers present. No `Signed-off-by:`.

---

### Task 5: Firmware — wrap `gpio_wait_*` handlers in `with_timeout`

**Files:**
- Modify: `crates/pico-de-gallo-firmware/src/handlers/gpio.rs` (all five `gpio_wait_for_*_handler` functions)
- Read: `crates/pico-de-gallo-internal/src/lib.rs` (the updated request types)

- [ ] **Step 1: Inspect the current handler shape**

```bash
grep -nA 15 "fn gpio_wait_for_high_handler" crates/pico-de-gallo-firmware/src/handlers/gpio.rs
```

Current shape (approximately):

```rust
pub async fn gpio_wait_for_high_handler(
    context: &mut Context,
    _header: VarHeader,
    req: GpioWaitHighRequest,
) -> Result<(), GpioError> {
    let idx = req.pin as usize;
    let flex = gpio_for_input!(context, idx)?;
    flex.wait_for_high().await;
    Ok(())
}
```

The `flex.wait_for_high().await` is the line that wedges the dispatcher when the pin never goes high.

- [ ] **Step 2: Add `with_timeout` import**

At the top of `crates/pico-de-gallo-firmware/src/handlers/gpio.rs`, add (or extend an existing `use embassy_time::...` line):

```rust
use embassy_time::{Duration, with_timeout};
```

- [ ] **Step 3: Wrap each handler with conditional timeout**

For each of the five handlers (`gpio_wait_for_high_handler`, `gpio_wait_for_low_handler`, `gpio_wait_for_rising_handler`, `gpio_wait_for_falling_handler`, `gpio_wait_for_any_handler`), apply this pattern:

```rust
pub async fn gpio_wait_for_high_handler(
    context: &mut Context,
    _header: VarHeader,
    req: GpioWaitHighRequest,
) -> Result<(), GpioError> {
    let idx = req.pin as usize;
    let flex = gpio_for_input!(context, idx)?;

    if req.timeout_ms == 0 {
        // Backward-compatible wait-forever path.
        flex.wait_for_high().await;
        Ok(())
    } else {
        match with_timeout(
            Duration::from_millis(req.timeout_ms as u64),
            flex.wait_for_high(),
        ).await {
            Ok(()) => Ok(()),
            Err(_) => {
                defmt::warn!(
                    "gpio_wait_for_high timeout (pin={}, ms={})",
                    req.pin, req.timeout_ms,
                );
                Err(GpioError::Timeout)
            }
        }
    }
}
```

Apply the same pattern to the other four handlers, swapping the method (`wait_for_low`, `wait_for_rising_edge`, `wait_for_falling_edge`, `wait_for_any_edge`) and the defmt warning message.

- [ ] **Step 4: Build the firmware for both HW revs**

```bash
cd crates/pico-de-gallo-firmware
cargo build --release --locked --target thumbv8m.main-none-eabihf
cargo build --release --locked --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2
```

Both must succeed.

- [ ] **Step 5: Clippy must be clean**

```bash
cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings
cargo clippy --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2 -- -D warnings
```

- [ ] **Step 6: Do NOT commit yet** — T6 (watchdog) and T7 (i2c_scan) are part of the same firmware change. Commit them together at T8.

---

### Task 6: Firmware — enable embassy-rp watchdog with dedicated feeder task

**Files:**
- Modify: `crates/pico-de-gallo-firmware/src/main.rs` (add watchdog init + feeder task)

Per PF-1: no `Cargo.toml` change needed; `embassy_rp::watchdog::Watchdog` is unconditionally exported. The watchdog feeder MUST be a dedicated `embassy_executor::task` because the main loop blocks inside `server.run()`. Per PF-1 gotcha #4, enable `pause_on_debug(true)` so debugger sessions don't reset the chip.

- [ ] **Step 1: Add the import**

At the top of `crates/pico-de-gallo-firmware/src/main.rs`, add (or extend existing imports):

```rust
use embassy_rp::watchdog::Watchdog;
use embassy_time::{Duration as TimeDuration, Timer};
```

(Rename to `TimeDuration` if `embassy_time::Duration` collides with `embassy_rp::watchdog::Duration` — verify during implementation.)

- [ ] **Step 2: Add the feeder task definition**

Near the other `#[embassy_executor::task]` definitions in `main.rs`:

```rust
/// Dedicated watchdog feeder task.
///
/// Feeds the embassy-rp watchdog every 800 ms with a 2-second timeout.
/// The 800 ms cadence leaves margin for the embassy executor's scheduling
/// jitter while keeping the worst-case recovery time under 2 seconds when
/// a handler genuinely wedges (e.g., a 1-Wire PIO program stalling on a
/// shorted bus, an embassy-rp peripheral bug).
///
/// This task MUST be the only feeder. RPC handlers cannot be trusted to
/// feed — the dispatcher-wedge regression (see
/// docs/superpowers/specs/2026-06-03-pico-de-gallo-category-a-review-synthesis.md
/// finding #2) means a wedged handler would also wedge any handler-based
/// feed scheme.
#[embassy_executor::task]
async fn watchdog_feeder_task(mut watchdog: Watchdog) {
    watchdog.start(embassy_rp::watchdog::Duration::from_millis(2_000));
    watchdog.pause_on_debug(true);
    loop {
        Timer::after(TimeDuration::from_millis(800)).await;
        watchdog.feed();
    }
}
```

**Verify during implementation:** confirm the exact `embassy_rp::watchdog::Duration` type / module path. PF-1 reports `Watchdog::feed(Duration)` takes a `Duration` argument; if that's true (re-arms each call), pass it explicitly. If the embassy-rp API in 0.10 has changed to argument-less `feed()`, adapt accordingly.

- [ ] **Step 3: Spawn the task in `main()`**

In the `main()` function in `crates/pico-de-gallo-firmware/src/main.rs`, after `let p = embassy_rp::init(config);` but before `spawner.must_spawn(...)` calls that depend on USB or peripherals, add:

```rust
spawner.must_spawn(watchdog_feeder_task(Watchdog::new(p.WATCHDOG)));
```

Verify `p.WATCHDOG` is the correct singleton name for RP2350 in embassy-rp 0.10 — PF-1 reports it should be (WATCHDOG is a standard singleton on this chip).

- [ ] **Step 4: Build both HW revs**

```bash
cd crates/pico-de-gallo-firmware
cargo build --release --locked --target thumbv8m.main-none-eabihf
cargo build --release --locked --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2
```

Both succeed. Watch for type-mismatch errors on the `Duration` vs `TimeDuration` rename.

- [ ] **Step 5: Do NOT commit yet.**

---

### Task 7: Firmware — fix `i2c_scan` to respect watchdog budget

**Files:**
- Modify: `crates/pico-de-gallo-firmware/src/handlers/i2c.rs` (the `i2c_scan_handler` function)

Per the reliability review finding #33: `i2c_scan` walks 256 addresses (or the standard 0x08–0x77 range), each with a NACK timeout that on flaky buses can be ~10 ms. A worst-case scan of ~3 s exceeds the 2-second watchdog budget. Fix with per-address `with_timeout(50ms)`.

- [ ] **Step 1: Inspect the current scan loop**

```bash
grep -nA 30 "fn i2c_scan_handler" crates/pico-de-gallo-firmware/src/handlers/i2c.rs
```

Note the current iteration loop and per-address probe call.

- [ ] **Step 2: Wrap each per-address probe in `with_timeout`**

The exact code shape depends on the existing implementation, but the pattern is:

```rust
// Before:
match i2c.read_async(addr, &mut [0u8; 1]).await {
    Ok(_) => responding.push(addr),
    Err(_) => {} // NACK or other error means no device
}

// After:
match with_timeout(
    Duration::from_millis(50),
    i2c.read_async(addr, &mut [0u8; 1]),
).await {
    Ok(Ok(_)) => responding.push(addr),
    Ok(Err(_)) => {} // NACK or other I²C error
    Err(_) => {
        // Per-address timeout. Bus is likely stuck on this address;
        // skip and continue scanning. The watchdog feeder task keeps
        // running independently, so the overall scan stays bounded.
        defmt::warn!("i2c_scan: address 0x{:02x} timed out", addr);
    }
}
```

Add `use embassy_time::{Duration, with_timeout};` at the top of the file if not already present.

The 50 ms per address budget keeps the worst-case scan at 50 × 128 = 6.4 s for the full range — which is **still longer than the 2-second watchdog window**. The mitigation works because the watchdog feeder is in a separate task that keeps running independently of the dispatcher; even if `i2c_scan` runs for 6.4 s, the device stays alive. The per-address timeout is about **bounding worst-case** so a single stuck address doesn't burn the entire scan budget, not about meeting the watchdog window.

- [ ] **Step 3: Build both HW revs (same commands as T6 Step 4)**.

- [ ] **Step 4: Clippy clean both HW revs.**

- [ ] **Step 5: Do NOT commit yet.**

---

### Task 8: Bump `pico-de-gallo-firmware` minor and commit T5–T8 as one firmware change

**Files:**
- Modify: `crates/pico-de-gallo-firmware/Cargo.toml` (version bump 0.10.0 → 0.11.0)
- Modify: `crates/pico-de-gallo-firmware/Cargo.lock` (refresh)
- Modify: `crates/pico-de-gallo-firmware/CHANGELOG.md` (0.11.0 entry)
- Modify: `book/src/internals/firmware.md` (document the watchdog per AGENTS.md §15.1)

- [ ] **Step 1: Bump version**

```bash
sed -i 's/^version = "0.10.0"$/version = "0.11.0"/' crates/pico-de-gallo-firmware/Cargo.toml
grep '^version' crates/pico-de-gallo-firmware/Cargo.toml
```

- [ ] **Step 2: Refresh the firmware lockfile** (separate from the host lockfile per AGENTS.md §2)

```bash
cd crates/pico-de-gallo-firmware && cargo update -p pico-de-gallo-firmware --locked
```

The firmware's `Cargo.lock` will also pull in the new `pico-de-gallo-internal` 0.7.0 via the path dependency. Verify:

```bash
grep -A 1 'name = "pico-de-gallo-internal"' Cargo.lock
```

- [ ] **Step 3: Add firmware CHANGELOG entry**

```markdown
### Added

- Embassy-rp watchdog enabled at 2-second timeout, fed every 800 ms
  by a dedicated `watchdog_feeder_task`. Recovers the device from
  handler hangs (1-Wire PIO stalls, embassy-rp peripheral bugs,
  any future infinite-await regression).
- `gpio_wait_for_*` handlers now honor the per-request `timeout_ms`
  field added in `internal` 0.7. A value of `0` preserves the
  pre-0.7 wait-forever behavior. Non-zero bounds the wait and
  returns `GpioError::Timeout` on expiry.

### Fixed

- `i2c_scan_handler` now applies a 50 ms per-address timeout via
  `with_timeout`. A single stuck address no longer burns the
  entire scan budget. The watchdog feeder task runs independently
  so the device stays alive even during long scans.

### Why

- Closes the dispatcher-wedge regression where a `gpio_wait_for_*`
  on a never-transitioning pin blocked every other endpoint until
  power-cycle (reliability finding B1).
- Closes the no-recovery-from-handler-hang regression (reliability
  finding R5).
- Reduces the worst-case impact of a flaky I²C bus on `i2c_scan`
  (reliability finding B2).

### Lockstep

- Coupled to `pico-de-gallo-internal` 0.7.0 (schema minor bump).
  See AGENTS.md §6.5.
```

- [ ] **Step 4: Update `book/src/internals/firmware.md`** to document the watchdog

Add a short subsection (3–6 lines) describing:

- The 2-second timeout.
- The dedicated `watchdog_feeder_task` (with cross-reference to `main.rs` line numbers if appropriate).
- The `pause_on_debug(true)` behavior so debugger sessions don't reset the chip.
- A note that the watchdog is the same on both HW revs.

This satisfies the AGENTS.md §15.1 parity rule for firmware behavior changes.

- [ ] **Step 5: Final firmware verification**

```bash
cd crates/pico-de-gallo-firmware
cargo fmt --check
cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings
cargo clippy --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2 -- -D warnings
cargo build --release --locked --target thumbv8m.main-none-eabihf
cargo build --release --locked --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2
```

All pass.

- [ ] **Step 6: Stage and commit**

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pico-de-gallo-firmware/src/handlers/gpio.rs \
        crates/pico-de-gallo-firmware/src/handlers/i2c.rs \
        crates/pico-de-gallo-firmware/src/main.rs \
        crates/pico-de-gallo-firmware/Cargo.toml \
        crates/pico-de-gallo-firmware/Cargo.lock \
        crates/pico-de-gallo-firmware/CHANGELOG.md \
        book/src/internals/firmware.md
git status --short
```

Expected: exactly seven files staged.

- [ ] **Step 7: Commit**

```bash
git commit -m "feat(firmware): enable watchdog, bound gpio_wait, fix i2c_scan budget

Three coordinated firmware fixes that close the worst regressions
identified by the reliability review (see synthesis spec):

1. gpio_wait_* handlers now honor the timeout_ms field added to
   the request types in internal 0.7. A value of 0 preserves the
   pre-0.7 wait-forever behavior. Non-zero wraps the await in
   embassy_time::with_timeout and returns GpioError::Timeout on
   expiry. Closes the dispatcher-wedge regression where a wait on
   a never-transitioning pin blocked every other endpoint
   (reliability finding B1).

2. Embassy-rp watchdog enabled at 2-second timeout, fed every
   800 ms by a dedicated watchdog_feeder_task. pause_on_debug(true)
   so debugger sessions don't reset the chip. Recovers the device
   from any future handler hang regression (reliability finding R5).

3. i2c_scan_handler wraps each per-address probe in
   with_timeout(50ms) so a single stuck address no longer burns
   the entire scan budget (reliability finding B2).

Schema is unchanged at this layer — the wire change is in
pico-de-gallo-internal 0.7.0 (separate commit). This firmware
version is lockstep-coupled to internal 0.7 per AGENTS.md §6.5.

Book updated per AGENTS.md §15.1 (book/src/internals/firmware.md).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

- [ ] **Step 8: Verify**

```bash
git --no-pager log -1 --stat
git --no-pager log --oneline -3
```

Expected: two commits on the branch (T4 internal, T8 firmware). Both with required trailers. No `Signed-off-by:`.

---

### Task 9: `pico-de-gallo-lib` — add `_with_timeout` overloads and bump

**Files:**
- Modify: `crates/pico-de-gallo-lib/src/lib.rs` (extend `gpio_wait_for_*` methods + `GpioError` mapping)
- Modify: `crates/pico-de-gallo-lib/Cargo.toml` (version + `pico-de-gallo-internal` dep)
- Modify: `crates/Cargo.lock` (refresh)
- Modify: `crates/pico-de-gallo-lib/CHANGELOG.md`

- [ ] **Step 1: Inspect the current `gpio_wait_for_*` methods**

```bash
grep -nA 10 "pub async fn gpio_wait_for_high" crates/pico-de-gallo-lib/src/lib.rs
```

Current signature is approximately:

```rust
pub async fn gpio_wait_for_high(&self, pin: u8) -> Result<(), PicoDeGalloError<GpioError>> {
    self.client.send_resp::<GpioWaitHighEndpoint>(&GpioWaitHighRequest { pin }).await.map_err(...)
}
```

- [ ] **Step 2: Update each method to include `timeout_ms: 0`**

Preserve the existing two-arg signature for backward compatibility by passing `timeout_ms: 0` (wait forever). For each of the five `gpio_wait_for_*` methods, update the call site:

```rust
pub async fn gpio_wait_for_high(&self, pin: u8) -> Result<(), PicoDeGalloError<GpioError>> {
    self.client
        .send_resp::<GpioWaitHighEndpoint>(&GpioWaitHighRequest { pin, timeout_ms: 0 })
        .await
        .map_err(/* ... existing mapping ... */)
}
```

- [ ] **Step 3: Add `_with_timeout(Duration)` overloads**

For each of the five wait methods, add a new method:

```rust
/// Wait for the pin to go high, with a host-supplied timeout.
///
/// Returns `Err(PicoDeGalloError::Endpoint(GpioError::Timeout))` if the
/// edge / level does not occur within `timeout`. Passing
/// `Duration::ZERO` (or `Duration::from_millis(0)`) reverts to the
/// wait-forever behavior of [`gpio_wait_for_high`](Self::gpio_wait_for_high).
///
/// Available on firmware schema 0.7+.
pub async fn gpio_wait_for_high_with_timeout(
    &self,
    pin: u8,
    timeout: std::time::Duration,
) -> Result<(), PicoDeGalloError<GpioError>> {
    let timeout_ms = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    self.client
        .send_resp::<GpioWaitHighEndpoint>(&GpioWaitHighRequest { pin, timeout_ms })
        .await
        .map_err(/* ... existing mapping ... */)
}
```

Note the saturating cast: a `Duration` of more than `u32::MAX` ms (~49.7 days) saturates to `u32::MAX` rather than wrapping. Document this.

- [ ] **Step 4: Verify `GpioError::Timeout` is already handled by the existing error mapping**

The `internal::GpioError` enum now has a `Timeout` variant; the host's `PicoDeGalloError<GpioError>::Endpoint(GpioError::Timeout)` propagation should be automatic via the existing `From` / `map_err`. Confirm with a quick grep:

```bash
grep -n "GpioError" crates/pico-de-gallo-lib/src/lib.rs | head -20
```

No mapping change required if `PicoDeGalloError::Endpoint(GpioError)` is the existing wrapper. If any explicit `match GpioError { ... }` is present (e.g., to convert to a user-facing string), add the `Timeout` arm.

- [ ] **Step 5: Add tests for the new overloads**

In the `#[cfg(test)] mod tests` block in `crates/pico-de-gallo-lib/src/lib.rs`, add tests that exercise the new methods. If the existing test suite uses a mock client, follow that pattern. If it relies on integration tests with real hardware, add `#[ignore]` placeholders and document.

- [ ] **Step 6: Bump version**

```bash
sed -i 's/^version = "0.6.0"$/version = "0.7.0"/' crates/pico-de-gallo-lib/Cargo.toml
```

Also update the `pico-de-gallo-internal` dep version in `crates/pico-de-gallo-lib/Cargo.toml`:

```bash
sed -i 's/pico-de-gallo-internal = { version = "0.6.0"/pico-de-gallo-internal = { version = "0.7.0"/' crates/pico-de-gallo-lib/Cargo.toml
grep 'pico-de-gallo-internal' crates/pico-de-gallo-lib/Cargo.toml
```

Verify both the `version` line and the `pico-de-gallo-internal` dep are now `0.7.0`.

- [ ] **Step 7: Refresh lockfile**

```bash
cd crates && cargo update -p pico-de-gallo-lib --locked
```

- [ ] **Step 8: Add CHANGELOG entry**

```markdown
### Added

- `PicoDeGallo::gpio_wait_for_{high,low,rising,falling,any}_with_timeout`
  methods take a `std::time::Duration` and return
  `GpioError::Timeout` on expiry. The existing two-arg methods
  (`gpio_wait_for_high(pin)` etc.) preserve the wait-forever
  behavior by passing `timeout_ms: 0`.

### Changed

- Bumped `pico-de-gallo-internal` dependency to 0.7.0 (wire schema
  change: append-only `timeout_ms: u32` on five request types,
  append-only `GpioError::Timeout` variant). Lockstep with firmware
  per AGENTS.md §6.5.
```

- [ ] **Step 9: Per-crate verification**

```bash
cd crates/pico-de-gallo-lib
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo hack --feature-powerset check
cargo +1.90 check
```

All pass.

- [ ] **Step 10: Stage and commit**

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pico-de-gallo-lib/src/lib.rs \
        crates/pico-de-gallo-lib/Cargo.toml \
        crates/pico-de-gallo-lib/CHANGELOG.md \
        crates/Cargo.lock
git commit -m "feat(lib): add gpio_wait_for_*_with_timeout, bump internal to 0.7

Five new methods on PicoDeGallo accept a std::time::Duration and
propagate GpioError::Timeout when the firmware's bounded wait
expires. The existing two-arg methods continue to wait forever
(timeout_ms: 0 on the wire), preserving backward compatibility at
the lib API level.

Bumps pico-de-gallo-internal dependency to 0.7.0. Lockstep release
with firmware 0.11.0 per AGENTS.md §6.5.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

- [ ] **Step 11: Verify**

```bash
git --no-pager log --oneline -4
```

Expected: three commits on branch (T4 internal, T8 firmware, T9 lib). All trailers correct.

---

### Task 10: `pico-de-gallo-hal` — add `wait_for_*_with_timeout` methods

**Files:**
- Modify: `crates/pico-de-gallo-hal/src/lib.rs` (extend `Gpio` impl with new async methods)
- Modify: `crates/pico-de-gallo-hal/Cargo.toml` (version + dep)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-hal/CHANGELOG.md`

The `embedded_hal_async::digital::Wait` trait methods don't take a timeout — we keep their current "wait forever" semantics by passing 0 through to `lib`. We add new project-specific `_with_timeout` methods on `Gpio`.

- [ ] **Step 1: Inspect existing `Wait` impl**

```bash
grep -nA 6 "impl embedded_hal_async::digital::Wait for Gpio" crates/pico-de-gallo-hal/src/lib.rs
```

The existing trait methods (`wait_for_high`, etc.) already call `lib`'s methods, which now pass `timeout_ms: 0` — so the trait methods are functionally unchanged.

- [ ] **Step 2: Add five new methods on `Gpio`**

In the `impl Gpio { ... }` block, add (one per wait kind):

```rust
/// Wait for the pin to go high, with a host-supplied timeout.
///
/// Like the [`Wait::wait_for_high`](embedded_hal_async::digital::Wait::wait_for_high)
/// trait method, but bounded. Returns
/// `Err(GpioHalError::Gpio(GpioError::Timeout))` if the level is not
/// reached within `timeout`.
///
/// **Why this is project-specific:** `embedded-hal-async`'s `Wait`
/// trait does not accept a timeout. Callers that need bounded waits
/// (recommended in production code; see
/// `docs/ai-agents/pico-de-gallo-hal-examples.md` §6.6 gotchas)
/// should use these methods directly instead of the trait.
///
/// Available on firmware schema 0.7+.
pub async fn wait_for_high_with_timeout(
    &mut self,
    timeout: std::time::Duration,
) -> Result<(), GpioHalError> {
    let gallo = self.gallo.lock().await;
    gallo
        .gpio_wait_for_high_with_timeout(self.pin, timeout)
        .await
        .map_err(GpioHalError::from)
}
```

Repeat for `wait_for_low_with_timeout`, `wait_for_rising_edge_with_timeout`, `wait_for_falling_edge_with_timeout`, `wait_for_any_edge_with_timeout`.

- [ ] **Step 3: Update the agent-guide markdown to mention the bounded variants**

Edit `docs/ai-agents/pico-de-gallo-hal-examples.md` §6.6 (GPIO async wait): add a note in the Gotchas section pointing at the new bounded methods, and recommending them for production use. Stay within ~3 lines added to keep within the 500–900 line budget.

This is required by AGENTS.md §15.1 (book ↔ code parity) — adding a HAL accessor must update the agent guide in the same PR.

- [ ] **Step 4: Bump version + dep**

```bash
sed -i 's/^version = "0.6.0"$/version = "0.7.0"/' crates/pico-de-gallo-hal/Cargo.toml
sed -i 's/pico-de-gallo-lib = { version = "0.6.0"/pico-de-gallo-lib = { version = "0.7.0"/' crates/pico-de-gallo-hal/Cargo.toml
grep -E 'version|pico-de-gallo-lib' crates/pico-de-gallo-hal/Cargo.toml | head -5
```

- [ ] **Step 5: Refresh lockfile**

```bash
cd crates && cargo update -p pico-de-gallo-hal --locked
```

- [ ] **Step 6: Add CHANGELOG entry**

```markdown
### Added

- `Gpio::wait_for_{high,low,rising_edge,falling_edge,any_edge}_with_timeout`
  async methods accept a `std::time::Duration` and return
  `GpioHalError::Gpio(GpioError::Timeout)` on expiry.
  `embedded-hal-async`'s `Wait` trait does not support timeouts, so
  these are exposed as inherent methods on `Gpio` instead. Recommended
  for production code; the trait methods retain their wait-forever
  semantics for compatibility with existing drivers.

### Changed

- Bumped `pico-de-gallo-lib` dependency to 0.7.0.
- Updated `docs/ai-agents/pico-de-gallo-hal-examples.md` §6.6 to
  recommend the bounded variants for production use.
```

- [ ] **Step 7: Per-crate verification**

```bash
cd crates/pico-de-gallo-hal
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

- [ ] **Step 8: Run the agent-guide drift check**

```bash
cd /home/balbi/workspace/pico-de-gallo
diff <(grep -oE '(hal\.[a-z0-9_]+\()|(Hal::[a-z0-9_]+\b)' \
        docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u) \
     <(grep -oE 'pub fn [a-z0-9_]+' \
        crates/pico-de-gallo-hal/src/lib.rs | sort -u) || true
```

The diff will show many right-side-only entries (HAL has many `pub fn`s not all named in the markdown); confirm there are no new left-side-only entries (i.e., no markdown references to accessors that don't exist).

- [ ] **Step 9: Stage and commit**

```bash
git add crates/pico-de-gallo-hal/src/lib.rs \
        crates/pico-de-gallo-hal/Cargo.toml \
        crates/pico-de-gallo-hal/CHANGELOG.md \
        crates/Cargo.lock \
        docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "feat(hal): add Gpio::wait_for_*_with_timeout methods

Five new async methods on Gpio accept a std::time::Duration and
propagate GpioError::Timeout from the firmware-side bounded wait.

The embedded-hal-async Wait trait doesn't support timeouts, so these
are project-specific inherent methods on Gpio. Trait methods retain
their wait-forever semantics for compatibility with existing
embedded-hal drivers.

Updates docs/ai-agents/pico-de-gallo-hal-examples.md §6.6 Gotchas
to recommend the bounded variants for production code (AGENTS.md
§15.1 book/code parity).

Bumps pico-de-gallo-lib dependency to 0.7.0. Lockstep with internal
0.7.0 / firmware 0.11.0 per AGENTS.md §6.5.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

- [ ] **Step 10: Verify**

```bash
git --no-pager log --oneline -5
```

Expected: four commits on branch (T4, T8, T9, T10). All trailers correct.

---

### Task 11: `pico-de-gallo-ffi` — add `gallo_gpio_wait_for_*_with_timeout` C functions

**Files:**
- Modify: `crates/pico-de-gallo-ffi/src/lib.rs` (new C functions + `Status::GpioTimeout` variant)
- Modify: `crates/pico-de-gallo-ffi/Cargo.toml` (version + dep)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-ffi/CHANGELOG.md`
- Regenerate: `pico_de_gallo.h` (via cbindgen build step)

- [ ] **Step 1: Inspect existing `gallo_gpio_wait_for_*` functions and the `Status` enum end**

```bash
grep -nA 15 "gallo_gpio_wait_for_high" crates/pico-de-gallo-ffi/src/lib.rs | head -25
grep -nE "(OneWireSearchFailed|OneWireSearchNextFailed|^\s+SystemReset)" crates/pico-de-gallo-ffi/src/lib.rs | head -10
```

The `Status` enum's last variant is the one to append after (per AGENTS.md §8 — append-only).

- [ ] **Step 2: Append `Status::GpioTimeout` at the end of the `Status` enum**

```rust
/// GPIO wait endpoint timed out before the requested edge/level was
/// detected. Returned only by the `_with_timeout` variants when the
/// host-supplied timeout expired. Available on firmware schema 0.7+.
GpioTimeout = -64, // verify next negative integer; do NOT reuse any existing code
```

**Important**: pick the next unused negative integer. Read the file to find the current minimum (most-negative) discriminant in the `Status` enum and use that minus 1. If the lowest is `-63`, the new variant is `-64`. The implementer must confirm the actual value during implementation.

Add a doc comment per AGENTS.md §15.

- [ ] **Step 3: Add `gpio_error_to_status` arm for the new variant**

Find the existing helper:

```bash
grep -nA 15 "fn gpio_error_to_status" crates/pico-de-gallo-ffi/src/lib.rs
```

Add a match arm for `GpioError::Timeout => Status::GpioTimeout`.

- [ ] **Step 4: Add the five new C functions**

For each of `gallo_gpio_wait_for_high`, `gallo_gpio_wait_for_low`, `gallo_gpio_wait_for_rising_edge`, `gallo_gpio_wait_for_falling_edge`, `gallo_gpio_wait_for_any_edge`, add a `_with_timeout_ms` variant. Example for High:

```rust
/// Wait for `pin` to go high, with a `timeout_ms` bound.
///
/// `timeout_ms == 0` waits forever (equivalent to
/// [`gallo_gpio_wait_for_high`]). Non-zero bounds the firmware-side
/// wait; expiry returns [`Status::GpioTimeout`].
///
/// Available on firmware schema 0.7+. Returns
/// [`Status::SchemaMismatch`] if invoked against older firmware.
///
/// # Safety
///
/// `handle` must be a valid `PicoDeGallo*` returned from
/// [`gallo_init`] or [`gallo_init_strict`] and not yet freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gallo_gpio_wait_for_high_with_timeout_ms(
    handle: *const PicoDeGallo,
    pin: u8,
    timeout_ms: u32,
) -> Status {
    let gallo = match unsafe { handle.as_ref() } {
        Some(g) => g,
        None => return Status::NullPointer,
    };
    let result = gallo
        .runtime()
        .block_on(gallo.inner().gpio_wait_for_high_with_timeout(
            pin,
            std::time::Duration::from_millis(timeout_ms as u64),
        ));
    match result {
        Ok(()) => Status::Ok,
        Err(PicoDeGalloError::Endpoint(e)) => gpio_error_to_status(e),
        Err(PicoDeGalloError::Comms(_)) => Status::CommsFailed,
    }
}
```

The exact `gallo.runtime()` / `gallo.inner()` accessors depend on how `PicoDeGallo` (the FFI opaque struct) wraps the lib type. Inspect the existing `gallo_gpio_wait_for_high` for the exact pattern and mirror it.

- [ ] **Step 5: Add null-pointer tests for each new function**

The existing test suite has `gallo_gpio_wait_for_high_null_*` style tests. Add `_with_timeout_ms_null_*` equivalents for each of the five new functions. Pattern:

```rust
#[test]
fn gallo_gpio_wait_for_high_with_timeout_ms_null_handle() {
    let status = unsafe {
        gallo_gpio_wait_for_high_with_timeout_ms(std::ptr::null(), 0, 100)
    };
    assert_eq!(status, Status::NullPointer);
}
```

- [ ] **Step 6: Add `Status::GpioTimeout` uniqueness check**

The discriminant-uniqueness test will land in PR 2 Task 30, but the implementer should manually verify here:

```bash
grep -oE '= -[0-9]+,' crates/pico-de-gallo-ffi/src/lib.rs | sort -u | wc -l
grep -oE '= -[0-9]+,' crates/pico-de-gallo-ffi/src/lib.rs | wc -l
```

Both counts must match (every discriminant unique).

- [ ] **Step 7: Bump version + dep**

```bash
sed -i 's/^version = "0.7.0"$/version = "0.8.0"/' crates/pico-de-gallo-ffi/Cargo.toml
sed -i 's/pico-de-gallo-lib = { version = "0.6.0"/pico-de-gallo-lib = { version = "0.7.0"/' crates/pico-de-gallo-ffi/Cargo.toml
grep -E '^version|pico-de-gallo-lib' crates/pico-de-gallo-ffi/Cargo.toml | head -5
```

(Note: pico-de-gallo-ffi was at 0.7.0 going into this PR; bumps to 0.8.0 because adding a new `Status` variant + new C functions is a feature.)

- [ ] **Step 8: Refresh lockfile**

```bash
cd crates && cargo update -p pico-de-gallo-ffi --locked
```

- [ ] **Step 9: Regenerate `pico_de_gallo.h` via cbindgen**

The FFI crate uses cbindgen via its `build.rs`. A clean rebuild regenerates the header:

```bash
cd crates/pico-de-gallo-ffi && cargo build --release --locked
```

Verify `pico_de_gallo.h` exists in the expected output path (typically `target/release/` or the project's configured location) and includes the new functions + `GpioTimeout` enum value. If the header is committed to the repo (check `git ls-files`), commit the regenerated copy.

- [ ] **Step 10: Add CHANGELOG entry**

```markdown
### Added

- `gallo_gpio_wait_for_{high,low,rising_edge,falling_edge,any_edge}_with_timeout_ms`
  C functions. `timeout_ms == 0` preserves wait-forever behavior;
  non-zero bounds the firmware-side wait and returns
  `Status::GpioTimeout` on expiry.
- `Status::GpioTimeout` enum variant (appended at end of `Status`
  enum; preserves stable C ABI per AGENTS.md §8).

### Changed

- Bumped `pico-de-gallo-lib` dependency to 0.7.0.
- Regenerated `pico_de_gallo.h` via cbindgen with the new
  functions and enum value.
```

- [ ] **Step 11: Per-crate verification**

```bash
cd crates/pico-de-gallo-ffi
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

- [ ] **Step 12: Stage and commit**

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pico-de-gallo-ffi/src/lib.rs \
        crates/pico-de-gallo-ffi/Cargo.toml \
        crates/pico-de-gallo-ffi/CHANGELOG.md \
        crates/Cargo.lock
# Add pico_de_gallo.h if it's tracked in the repo:
if git ls-files --error-unmatch pico_de_gallo.h 2>/dev/null; then
    git add pico_de_gallo.h
fi
git commit -m "feat(ffi): add gallo_gpio_wait_for_*_with_timeout_ms

Five new C functions accept a u32 timeout_ms; timeout_ms == 0 waits
forever (matches the existing functions). Non-zero bounds the
firmware-side wait and returns Status::GpioTimeout on expiry.

Status::GpioTimeout is appended at the end of the enum, preserving
stable C ABI per AGENTS.md §8.

Regenerated pico_de_gallo.h via cbindgen.

Bumps pico-de-gallo-lib dependency to 0.7.0. Lockstep release per
AGENTS.md §6.5.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 12: `pico-de-gallo-app` — add `--timeout-ms` flag to `gpio wait` subcommands

**Files:**
- Modify: `crates/pico-de-gallo-app/src/lib.rs` (or wherever the `gpio wait` subcommand is defined)
- Modify: `crates/pico-de-gallo-app/Cargo.toml` (version + dep)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-app/CHANGELOG.md`
- Modify: `book/src/crates/app.md` and `book/src/interfaces/gpio.md` (per AGENTS.md §15.1)

- [ ] **Step 1: Inspect the existing `gpio wait` subcommand definitions**

```bash
grep -nE "wait[-_]for[-_](high|low|rising|falling|any)" crates/pico-de-gallo-app/src/lib.rs | head -20
```

Find the clap struct for the gpio wait subcommands.

- [ ] **Step 2: Add a `--timeout-ms` flag (default 0)**

Per the existing clap convention in the crate, add:

```rust
/// Per-request timeout in milliseconds. 0 (default) means wait
/// forever. Non-zero values bound the firmware-side wait and return
/// `GpioError::Timeout` on expiry. Available on firmware 0.7+.
#[arg(long, default_value_t = 0)]
timeout_ms: u32,
```

Add to each of the five `gpio wait` subcommand argument structs (or, if they share an arg struct, just once).

- [ ] **Step 3: Wire the flag through to `lib`**

In the subcommand handler, when `timeout_ms != 0` call `gpio_wait_for_*_with_timeout(pin, Duration::from_millis(timeout_ms as u64))`; otherwise call the existing `gpio_wait_for_*(pin)`. On `Err(PicoDeGalloError::Endpoint(GpioError::Timeout))`, print a clear `eprintln!` and exit with a non-zero status code (use the existing CLI exit-code convention, often `Status::GpioTimeout as i32 * -1` or a dedicated `ExitCode::Timeout`).

- [ ] **Step 4: Bump version + dep**

```bash
sed -i 's/^version = "0.7.0"$/version = "0.8.0"/' crates/pico-de-gallo-app/Cargo.toml
sed -i 's/pico-de-gallo-lib = { version = "0.6.0"/pico-de-gallo-lib = { version = "0.7.0"/' crates/pico-de-gallo-app/Cargo.toml
grep -E '^version|pico-de-gallo-lib' crates/pico-de-gallo-app/Cargo.toml | head -5
```

- [ ] **Step 5: Refresh lockfile**

```bash
cd crates && cargo update -p pico-de-gallo-app --locked
```

- [ ] **Step 6: Update book chapters**

Per AGENTS.md §15.1, CLI flag changes must update `book/src/crates/app.md` and the relevant interface chapter. Add a short paragraph to `book/src/interfaces/gpio.md` describing `--timeout-ms` with an example command:

```bash
gallo gpio wait-for-high --pin 2 --timeout-ms 1000
```

Update `book/src/crates/app.md` if it has a gpio-wait subcommand table.

- [ ] **Step 7: Update CLI tests**

If the CLI has clap-parsing tests (`crates/pico-de-gallo-app/tests/` or inline), add a test case exercising `--timeout-ms`.

- [ ] **Step 8: Add CHANGELOG entry**

```markdown
### Added

- `gallo gpio wait-for-{high,low,rising-edge,falling-edge,any-edge}`
  subcommands gained a `--timeout-ms <MS>` flag (default 0). 0 waits
  forever (matches pre-0.8 behavior); non-zero bounds the wait and
  exits non-zero on `GpioError::Timeout`.

### Changed

- Bumped `pico-de-gallo-lib` dependency to 0.7.0.
- Updated `book/src/interfaces/gpio.md` to document `--timeout-ms`.
```

- [ ] **Step 9: Per-crate verification**

```bash
cd crates/pico-de-gallo-app
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

- [ ] **Step 10: Verify `mdbook build book` is clean**

```bash
cd /home/balbi/workspace/pico-de-gallo
mdbook build book 2>&1 | tail -5
```

Expected: no broken links, no missing referenced files.

- [ ] **Step 11: Stage and commit**

```bash
git add crates/pico-de-gallo-app/src/lib.rs \
        crates/pico-de-gallo-app/Cargo.toml \
        crates/pico-de-gallo-app/CHANGELOG.md \
        crates/Cargo.lock \
        book/src/crates/app.md \
        book/src/interfaces/gpio.md
git status --short
git commit -m "feat(application): add --timeout-ms to gpio wait subcommands

Five gpio wait subcommands gain a --timeout-ms <MS> flag (default 0
preserves wait-forever behavior). Non-zero bounds the firmware-side
wait; expiry exits non-zero with a clear error message.

Book chapters updated per AGENTS.md §15.1:
- book/src/interfaces/gpio.md documents the new flag with an example.
- book/src/crates/app.md updated if the subcommand table was present.

Bumps pico-de-gallo-lib dependency to 0.7.0. Lockstep release per
AGENTS.md §6.5.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 13: `pyco-de-gallo` — add `wait_for_*_with_timeout` Python methods

**Files:**
- Modify: `crates/pyco-de-gallo/src/lib.rs`
- Modify: `crates/pyco-de-gallo/Cargo.toml` (version + dep)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pyco-de-gallo/CHANGELOG.md`
- Modify: `book/src/crates/python.md` (per AGENTS.md §15.1)

- [ ] **Step 1: Inspect existing `wait_for_*` methods on the `PycoDeGallo` pyclass**

```bash
grep -nE "fn wait_for_(high|low|rising|falling|any)" crates/pyco-de-gallo/src/lib.rs | head -10
```

- [ ] **Step 2: Add five new `#[pymethods]` with `_with_timeout` suffix**

For each existing wait method, add a `_with_timeout(pin, timeout_ms)` variant:

```rust
/// Wait for the pin to go high, with a bounded timeout.
///
/// Args:
///     pin: The GPIO pin to monitor (0-3).
///     timeout_ms: Timeout in milliseconds. 0 means wait forever
///         (equivalent to `wait_for_high(pin)`). Non-zero bounds
///         the firmware-side wait.
///
/// Raises:
///     RuntimeError: If the timeout expires (GpioError::Timeout) or
///         any other firmware error occurs.
///
/// Available on firmware schema 0.7+.
pub fn wait_for_high_with_timeout(
    &self,
    py: Python<'_>,
    pin: u8,
    timeout_ms: u32,
) -> PyResult<()> {
    py.allow_threads(|| {
        self.runtime
            .block_on(self.inner.gpio_wait_for_high_with_timeout(
                pin,
                std::time::Duration::from_millis(timeout_ms as u64),
            ))
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    })
}
```

Repeat for `wait_for_low_with_timeout`, `wait_for_rising_edge_with_timeout`, `wait_for_falling_edge_with_timeout`, `wait_for_any_edge_with_timeout`. Match the docstring style (Google-style per AGENTS.md §9 — `Args:`/`Returns:`/`Raises:`).

- [ ] **Step 3: Bump version + dep**

```bash
sed -i 's/^version = "0.3.0"$/version = "0.4.0"/' crates/pyco-de-gallo/Cargo.toml
sed -i 's/pico-de-gallo-lib = { version = "0.6.0"/pico-de-gallo-lib = { version = "0.7.0"/' crates/pyco-de-gallo/Cargo.toml
grep -E '^version|pico-de-gallo-lib' crates/pyco-de-gallo/Cargo.toml | head -5
```

- [ ] **Step 4: Refresh lockfile**

```bash
cd crates && cargo update -p pyco-de-gallo --locked
```

- [ ] **Step 5: Update `book/src/crates/python.md`** to mention the new methods.

- [ ] **Step 6: Add CHANGELOG entry**

```markdown
### Added

- `PycoDeGallo.wait_for_{high,low,rising_edge,falling_edge,any_edge}_with_timeout`
  Python methods accept a `timeout_ms: int`. 0 waits forever
  (matches existing methods); non-zero bounds the wait and raises
  `RuntimeError` on `GpioError::Timeout`.

### Changed

- Bumped `pico-de-gallo-lib` dependency to 0.7.0.
- Updated `book/src/crates/python.md`.
```

- [ ] **Step 7: Per-crate verification**

```bash
cd crates/pyco-de-gallo
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

(Per AGENTS.md §5.5, pyco-de-gallo currently has no Rust-side tests; `cargo test` likely passes trivially. Don't add tests here — that's Category C finding #38.)

- [ ] **Step 8: Stage and commit**

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pyco-de-gallo/src/lib.rs \
        crates/pyco-de-gallo/Cargo.toml \
        crates/pyco-de-gallo/CHANGELOG.md \
        crates/Cargo.lock \
        book/src/crates/python.md
git commit -m "feat(pyco): add wait_for_*_with_timeout Python methods

Five new Python methods on PycoDeGallo accept a timeout_ms: int and
raise RuntimeError on GpioError::Timeout when the firmware-side
bounded wait expires.

Bumps pico-de-gallo-lib dependency to 0.7.0. Lockstep release per
AGENTS.md §6.5.

Updates book/src/crates/python.md per AGENTS.md §15.1.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 14: PR 1 — full local CI mirror

**Files:** (read-only verification across both workspaces)

- [ ] **Step 1: Host workspace lockfile drift guard**

```bash
cd /home/balbi/workspace/pico-de-gallo/crates
cargo check --workspace --locked 2>&1 | tail -10
```

Must succeed with no "would update lockfile" errors.

- [ ] **Step 2: Firmware workspace lockfile drift guard**

```bash
cd /home/balbi/workspace/pico-de-gallo/crates/pico-de-gallo-firmware
cargo check --locked --target thumbv8m.main-none-eabihf 2>&1 | tail -10
```

- [ ] **Step 3: Per-crate CI mirror for every host crate touched**

For each of `pico-de-gallo-internal`, `pico-de-gallo-lib`, `pico-de-gallo-hal`, `pico-de-gallo-ffi`, `pico-de-gallo-app`, `pyco-de-gallo`:

```bash
cd /home/balbi/workspace/pico-de-gallo/crates/<crate>
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo hack --feature-powerset check
cargo +1.90 check
```

Document each command and the pass result. Any failure here is a blocker; loop back into the failing task and fix.

- [ ] **Step 4: Firmware build matrix (both HW revs)**

```bash
cd /home/balbi/workspace/pico-de-gallo/crates/pico-de-gallo-firmware
cargo fmt --check
cargo clippy --target thumbv8m.main-none-eabihf -- -D warnings
cargo clippy --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2 -- -D warnings
cargo build --release --locked --target thumbv8m.main-none-eabihf
cargo build --release --locked --target thumbv8m.main-none-eabihf --no-default-features --features hw-rev2
```

- [ ] **Step 5: `cargo deny` on both workspaces**

```bash
cd /home/balbi/workspace/pico-de-gallo/crates && cargo deny --manifest-path Cargo.toml check 2>&1 | tail -10
cd /home/balbi/workspace/pico-de-gallo/crates/pico-de-gallo-firmware && cargo deny check 2>&1 | tail -10
```

- [ ] **Step 6: `cargo-semver-checks` on `pico-de-gallo-internal`**

```bash
cd /home/balbi/workspace/pico-de-gallo/crates/pico-de-gallo-internal
cargo semver-checks check-release 2>&1 | tail -20
```

Expected: a **minor** semver bump is allowed (appending a struct field on a public type and appending an enum variant are minor changes). If semver-checks reports them as breaking, audit the report — there may be a `#[non_exhaustive]` discipline issue worth resolving in this PR.

- [ ] **Step 7: `mdbook build book`**

```bash
cd /home/balbi/workspace/pico-de-gallo && mdbook build book 2>&1 | tail -5
```

- [ ] **Step 8: `actionlint .github/workflows/*.yml`**

```bash
actionlint .github/workflows/*.yml 2>&1 | tail -10
```

(No workflows changed in PR 1, but run the check anyway to catch any drift.)

- [ ] **Step 9: Confirm commit series shape**

```bash
git --no-pager log --oneline main..HEAD
```

Expected: six commits (T4 internal, T8 firmware, T9 lib, T10 hal, T11 ffi, T12 app, T13 pyco — seven total). Each with proper scope and trailers.

- [ ] **Step 10: Trailer audit on every commit**

```bash
for sha in $(git --no-pager log --format=%H main..HEAD); do
    body=$(git --no-pager log -1 --format='%B' "$sha")
    has_co=$(echo "$body" | grep -c "Co-authored-by: Copilot")
    has_assisted=$(echo "$body" | grep -c "Assisted-by:")
    has_signed=$(echo "$body" | grep -c "Signed-off-by:")
    subj=$(git --no-pager log -1 --format='%s' "$sha")
    printf "%s  co:%d assisted:%d signed:%d  %s\n" "$sha" "$has_co" "$has_assisted" "$has_signed" "$subj"
done
```

Every commit must show `co:1 assisted:1 signed:0`.

---

### Task 15: PR 1 — push branch and open draft PR

**Files:** (no file changes; git/gh operations only)

- [ ] **Step 1: Push branch to your fork (`origin`)**

```bash
git push -u origin category-a-hotfix-wire 2>&1 | tail -5
```

- [ ] **Step 2: Write the PR body to a temp file**

The PR body MUST include the "Release operator runbook" verbatim from the §"PR 1 Overview" section above. Build it from:

- Summary (1 paragraph: this PR closes findings #1, #2, #3, #33 from the synthesis spec; wire change with lockstep release).
- What changed (per-crate bullets).
- Release operator runbook (paste verbatim from the §"PR 1 Overview" section of this plan).
- Test plan (checkboxes corresponding to Task 14's CI mirror commands, all checked).

Write to `/tmp/opencode/pr1-body.md`.

- [ ] **Step 3: Open the draft PR**

```bash
gh pr create \
  --repo OpenDevicePartnership/pico-de-gallo \
  --base main \
  --head felipebalbi:category-a-hotfix-wire \
  --draft \
  --title "feat(internal,firmware,lib,hal,ffi,application,pyco)!: bound gpio_wait, enable watchdog" \
  --body-file /tmp/opencode/pr1-body.md
```

The title's multi-scope syntax matches AGENTS.md §10. The `!` triggers BREAKING-CHANGE handling at the PR level too.

- [ ] **Step 4: Verify PR**

```bash
gh pr view --repo OpenDevicePartnership/pico-de-gallo --json number,state,isDraft,url
```

Capture the PR number and URL for cross-referencing in PR 2's description.

- [ ] **Step 5: Watch CI**

CI on the PR will run all the workflows we mirrored locally in Task 14, plus the actionlint / deny / semver / lockfile guards. Wait for green; if anything fails, address in a follow-up commit (do NOT force-push to a release branch).

**Do NOT mark the PR ready for review until CI is green and the operator has actually read the release runbook.** Per AGENTS.md §11.

---

## PR 2 Overview — Host-only fixes

**Branch:** `category-a-hotfix-host` (off `main`).
**Scope:** schema validation, validation enforcement at entry points, HAL accessor gaps, doc drift, preventive guards. No wire change. Each crate releases independently.
**Schema impact:** None.
**Release-please impact:** standard separate-PR-per-crate.

### Task 16: Create PR 2 branch off `main`

**Files:** none.

- [ ] **Step 1: Make sure you're on `main` and up to date**

```bash
cd /home/balbi/workspace/pico-de-gallo
git fetch odp main
git checkout main
git pull odp main --ff-only
```

- [ ] **Step 2: Create the branch off `main`**

```bash
git checkout -b category-a-hotfix-host main
git status --short
git log --oneline -1
```

PR 2 is intentionally independent of PR 1. Both can be reviewed in parallel because PR 2's changes don't touch the wire types or firmware. If PR 1 is in flight and you've based PR 2 off a `main` that doesn't yet contain PR 1's commits, no problem — PR 2 will trivially merge with PR 1's merge later.

---

### Task 17: Fix `validate()` major-version check

**Files:**
- Modify: `crates/pico-de-gallo-lib/src/lib.rs:667` (the `validate()` schema check)
- Add: a regression test asserting `SchemaMismatch` on bumped `schema_major`
- Modify: `crates/pico-de-gallo-lib/Cargo.toml` (version bump, patch)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-lib/CHANGELOG.md`

- [ ] **Step 1: Inspect the current `validate()` body**

```bash
grep -nA 20 "pub async fn validate" crates/pico-de-gallo-lib/src/lib.rs
```

Around line 667 there's the check:

```rust
if info.schema_minor != SCHEMA_VERSION_MINOR { ... }
```

This ignores `schema_major`.

- [ ] **Step 2: Fix the check**

Change:

```rust
if info.schema_minor != SCHEMA_VERSION_MINOR {
    return Err(PicoDeGalloError::Comms(
        WireError::SchemaMismatch { /* ... */ }
    ));
}
```

to:

```rust
if info.schema_major != SCHEMA_VERSION_MAJOR
    || info.schema_minor != SCHEMA_VERSION_MINOR
{
    return Err(PicoDeGalloError::Comms(
        WireError::SchemaMismatch { /* ... include both versions in error ... */ }
    ));
}
```

If `WireError::SchemaMismatch` carries a `host_minor: u16, device_minor: u16` payload, extend it to also carry the majors (or add a separate `SchemaMajorMismatch` variant — choose the option that maintains wire compatibility for `WireError`; this is a host-side error type, not on the wire, so adding fields is safe).

- [ ] **Step 3: Add a regression test**

In the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn validate_rejects_bumped_schema_major() {
    // Construct a DeviceInfo with a major-version skew but matching minor.
    let info = DeviceInfo {
        firmware_version: "0.0.0".into(),
        schema_major: SCHEMA_VERSION_MAJOR.wrapping_add(1),
        schema_minor: SCHEMA_VERSION_MINOR,
        schema_patch: SCHEMA_VERSION_PATCH,
        hw_version: HwVersion::Rev1,
        capabilities: Capabilities::NONE,
    };
    // The validation logic should reject this — pre-fix, it returned Ok.
    // Use whatever helper the existing test suite uses to invoke the check
    // without going through the network round-trip.
    let result = check_schema_compatible(&info);
    assert!(matches!(result, Err(_)), "major-version skew must be rejected");
}
```

Note: if `check_schema_compatible` doesn't exist as a separate helper, extract it from `validate()` so it's testable, or use a thin wrapper. Don't add a real-network test.

- [ ] **Step 4: Bump version (patch)**

```bash
sed -i 's/^version = "0.7.0"$/version = "0.7.1"/' crates/pico-de-gallo-lib/Cargo.toml
```

PR 2 starts from the assumption that PR 1 has either landed or will land first; this patch builds on lib 0.7.0.

If PR 2 lands BEFORE PR 1 (unusual but possible), bump from 0.6.0 → 0.6.1 instead. The implementer must check the actual current version in `Cargo.toml` and bump from there.

- [ ] **Step 5: Refresh lockfile, add CHANGELOG entry, run per-crate CI mirror, commit**

```bash
cd /home/balbi/workspace/pico-de-gallo/crates && cargo update -p pico-de-gallo-lib --locked
```

CHANGELOG entry:

```markdown
### Fixed

- `PicoDeGallo::validate()` now checks `schema_major` in addition to
  `schema_minor`. Previously, a firmware reporting a bumped major
  version with a matching minor would silently pass validation and
  the host would subsequently mis-decode wire bytes.
```

Commit:

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pico-de-gallo-lib/src/lib.rs \
        crates/pico-de-gallo-lib/Cargo.toml \
        crates/pico-de-gallo-lib/CHANGELOG.md \
        crates/Cargo.lock
git commit -m "fix(lib): check schema_major in validate()

The major-version field was silently ignored. A firmware on a bumped
major would pass validation against an older host and subsequent
RPCs would mis-decode wire bytes (silent garbage out).

Adds a regression test (validate_rejects_bumped_schema_major) that
constructs a DeviceInfo with a skewed major and confirms validation
fails.

Closes Category A finding #1 (reviewer B1).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 18: HAL — add `Hal::new_validated`, `Hal::system_reset_subscriptions`, `Hal::validate`; re-export missing types

**Files:**
- Modify: `crates/pico-de-gallo-hal/src/lib.rs`
- Modify: `crates/pico-de-gallo-hal/Cargo.toml` (bump minor)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-hal/CHANGELOG.md`
- Modify: `docs/ai-agents/pico-de-gallo-hal-examples.md` (drop the AdcChannel-not-re-exported gotcha in §6.9)

This closes findings #5, #6, #13, plus PR #56 side findings #1 and #2.

- [ ] **Step 1: Add `Hal::new_validated()`**

In the `impl Hal` block in `crates/pico-de-gallo-hal/src/lib.rs`:

```rust
/// Like [`Hal::new`] but calls [`Hal::validate`] before returning.
///
/// Use this when you want construction to fail loudly on a device
/// that's not connected or has a schema-version mismatch. The
/// existing [`Hal::new`] constructs lazily — failures surface on
/// the first real RPC.
///
/// Available on firmware schema 0.7+.
pub fn new_validated() -> Result<Self, HalInitError> {
    let hal = Self::new();
    hal.handle.block_on(async {
        let gallo = hal.gallo.lock().await;
        gallo.validate().await.map_err(HalInitError::from)
    })?;
    Ok(hal)
}
```

Add a `Hal::new_validated_with_serial_number(serial: &str)` variant for symmetry with the existing constructor pair.

- [ ] **Step 2: Add `Hal::system_reset_subscriptions` and `Hal::validate` accessors**

```rust
/// Tear down any GPIO subscriptions still held by the firmware from
/// a previous host process.
///
/// Returns the number of subscriptions cleared. Idempotent; safe to
/// call on every connect.
///
/// **When to use:** if your application uses GPIO subscriptions and
/// you crashed last run, the firmware may still own a pin until
/// next call. This is the documented recovery path (see also
/// [`book/src/internals/firmware.md`](https://opendevicepartnership.github.io/pico-de-gallo/internals/firmware.html)).
pub fn system_reset_subscriptions(&self) -> Result<u32, HalError> {
    if Self::in_async_context() {
        block_in_place(|| self.system_reset_subscriptions_inner())
    } else {
        self.system_reset_subscriptions_inner()
    }
}

fn system_reset_subscriptions_inner(&self) -> Result<u32, HalError> {
    let handle = self.handle.clone();
    let gallo = handle.block_on(self.gallo.lock());
    handle
        .block_on(gallo.system_reset_subscriptions())
        .map_err(HalError::from)
}

/// Validate the firmware-reported schema version against this HAL's
/// compiled-in schema.
///
/// Called automatically by [`Hal::new_validated`]. Exposed here for
/// callers that constructed via [`Hal::new`] and want to validate
/// after the fact.
pub fn validate(&self) -> Result<(), HalError> {
    // ... block_in_place mirror of the system_reset pattern above
}
```

`HalError` is the generic top-level HAL error type. If a top-level `HalError` doesn't exist (the HAL today has per-peripheral error types per architect finding #21), add a minimal one for these system-level operations, or wrap in `Box<dyn std::error::Error>`. The implementer should pick the least-disruptive option and document the choice.

- [ ] **Step 3: Re-export the missing types**

In the `pub use pico_de_gallo_lib::{ ... }` block at `crates/pico-de-gallo-hal/src/lib.rs:56-58`, add:

```rust
pub use pico_de_gallo_lib::{
    GpioEvent, I2cFrequency, SpiConfigurationInfo, SpiPhase, SpiPolarity, UartConfigurationInfo,
    // Added in 0.8 — closes Category A finding #6.
    AdcChannel, AdcConfigurationInfo, GpioDirection, GpioEdge, GpioPull,
};
```

- [ ] **Step 4: Remove stale `Hal::uart_set_config` doc reference**

Inspect `crates/pico-de-gallo-hal/src/lib.rs:1282`:

```bash
sed -n '1278,1290p' crates/pico-de-gallo-hal/src/lib.rs
```

Replace the misleading sentence (per PR #56 side finding #3 and Category A finding #13) with prose that correctly says baud rate is fixed at the firmware default and recommends dropping to `pico-de-gallo-lib` for changes.

- [ ] **Step 5: Update the agent-guide markdown**

Edit `docs/ai-agents/pico-de-gallo-hal-examples.md` §6.9 (ADC) to remove the "not re-exported by the HAL" gotcha (because they are re-exported now). Change the import from `use pico_de_gallo_lib::AdcChannel;` to `use pico_de_gallo_hal::AdcChannel;`. Apply the same fix to §6.5 / §6.6 / §6.7 wherever `pico_de_gallo_lib::GpioDirection` / `GpioPull` / `GpioEdge` are referenced.

Also remove the §4 (Cargo setup) bullet about `pico-de-gallo-lib` being needed for those types.

Update the §6.7 GPIO subscribe gotcha that says "the HAL does not expose a recovery method" — now it does (`hal.system_reset_subscriptions()`).

- [ ] **Step 6: Run the agent-guide drift check**

```bash
diff <(grep -oE '(hal\.[a-z0-9_]+\()|(Hal::[a-z0-9_]+\b)' \
        docs/ai-agents/pico-de-gallo-hal-examples.md | sort -u) \
     <(grep -oE 'pub fn [a-z0-9_]+' \
        crates/pico-de-gallo-hal/src/lib.rs | sort -u) || true
```

`hal.system_reset_subscriptions(` and `hal.validate(` will now appear on the left and must appear in the right list.

- [ ] **Step 7: Bump hal minor**

```bash
sed -i 's/^version = "0.7.0"$/version = "0.8.0"/' crates/pico-de-gallo-hal/Cargo.toml
```

(Adjust starting version per the actual `Cargo.toml` at the time — depends on whether PR 1 has merged.)

- [ ] **Step 8: CHANGELOG entry**

```markdown
### Added

- `Hal::new_validated()` and `Hal::new_validated_with_serial_number(serial)`
  constructors call `validate()` before returning, failing loudly on
  device-not-connected or schema-version mismatch. The existing
  lazy `Hal::new()` / `Hal::new_with_serial_number()` continue to
  defer failures until the first RPC.
- `Hal::validate()` accessor for callers that constructed via the
  lazy constructors.
- `Hal::system_reset_subscriptions() -> Result<u32, HalError>`
  exposes the firmware-side subscription teardown previously only
  reachable via `pico-de-gallo-lib`.
- Re-exported `AdcChannel`, `AdcConfigurationInfo`, `GpioDirection`,
  `GpioEdge`, `GpioPull` from `pico-de-gallo-lib`.

### Fixed

- Removed the stale doc-comment reference to a non-existent
  `Hal::uart_set_config` method. Documentation now correctly notes
  that UART baud is fixed at firmware default and changes require
  dropping to `pico-de-gallo-lib`.

### Changed

- Updated `docs/ai-agents/pico-de-gallo-hal-examples.md` to drop
  the "not re-exported by the HAL" gotchas (re-exports now present)
  and to reflect the new recovery accessor in §6.7 GPIO subscribe.
```

- [ ] **Step 9: Per-crate verification + lockfile refresh**

```bash
cd crates && cargo update -p pico-de-gallo-hal --locked
cd pico-de-gallo-hal
cargo fmt --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo hack --feature-powerset check
cargo +1.90 check
```

- [ ] **Step 10: Stage and commit**

```bash
cd /home/balbi/workspace/pico-de-gallo
git add crates/pico-de-gallo-hal/src/lib.rs \
        crates/pico-de-gallo-hal/Cargo.toml \
        crates/pico-de-gallo-hal/CHANGELOG.md \
        crates/Cargo.lock \
        docs/ai-agents/pico-de-gallo-hal-examples.md
git commit -m "feat(hal): add new_validated, system_reset_subscriptions, validate; re-export types

Closes Category A findings #5, #6, #13:

- Hal::new_validated() (+ _with_serial_number variant) calls
  validate() before returning. Existing Hal::new() stays lazy.
- Hal::validate() exposed for post-construction validation.
- Hal::system_reset_subscriptions() exposes the firmware-side
  subscription teardown, closing the recovery gap for hosts that
  crashed mid-subscription.
- Re-export AdcChannel, AdcConfigurationInfo, GpioDirection,
  GpioEdge, GpioPull from pico-de-gallo-lib. Driver authors no
  longer need to add pico-de-gallo-lib to their Cargo.toml.
- Drop stale Hal::uart_set_config doc reference (replaced with
  accurate baud-is-fixed prose).

Updates docs/ai-agents/pico-de-gallo-hal-examples.md (drops the
\"not re-exported\" gotchas, updates the §6.7 recovery story).

Also closes PR #56 side findings #1, #2, #3.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 19: FFI — add `gallo_init_strict`

**Files:**
- Modify: `crates/pico-de-gallo-ffi/src/lib.rs`
- Modify: `crates/pico-de-gallo-ffi/Cargo.toml` (version bump, minor)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-ffi/CHANGELOG.md`
- Regenerate: `pico_de_gallo.h`

Closes Category A finding #4 (FFI side of validation enforcement).

- [ ] **Step 1: Find the existing `gallo_init`**

```bash
grep -nA 10 "extern \"C\" fn gallo_init" crates/pico-de-gallo-ffi/src/lib.rs
```

- [ ] **Step 2: Add `gallo_init_strict`**

```rust
/// Like [`gallo_init`] but calls [`gallo_get_device_info`]
/// internally and returns `null` if the firmware schema doesn't
/// match this host's compiled-in schema.
///
/// **Prefer this constructor in production code.** The lazy
/// [`gallo_init`] surfaces device-not-connected and schema-mismatch
/// only on the first real RPC, which makes diagnostics harder.
///
/// Returns `null` on:
/// - device not enumerated on USB
/// - schema version mismatch
/// - any communication error during validation
///
/// On success, the returned pointer must be freed with
/// [`gallo_free`] when no longer needed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn gallo_init_strict() -> *mut PicoDeGallo {
    let handle = unsafe { gallo_init() };
    if handle.is_null() { return handle; }
    let gallo = unsafe { handle.as_ref().unwrap() };
    let validate_result = gallo
        .runtime()
        .block_on(gallo.inner().validate());
    match validate_result {
        Ok(()) => handle,
        Err(_) => {
            // Free the handle and return null to signal failure.
            unsafe { gallo_free(handle); }
            std::ptr::null_mut()
        }
    }
}
```

Adapt `gallo.runtime()` / `gallo.inner()` to whatever pattern the existing FFI uses.

- [ ] **Step 3: Add a `gallo_init_strict_with_serial_number(*const c_char)` variant**

Same pattern, calling `gallo_init_with_serial_number` instead.

- [ ] **Step 4: Add null-pointer tests** for both new functions.

- [ ] **Step 5: Bump version**

```bash
sed -i 's/^version = "0.8.0"$/version = "0.9.0"/' crates/pico-de-gallo-ffi/Cargo.toml
```

(Adjust starting version per current state.)

- [ ] **Step 6: Refresh lockfile + regenerate header + per-crate CI mirror.**

```bash
cd crates && cargo update -p pico-de-gallo-ffi --locked
cd pico-de-gallo-ffi && cargo build --release --locked
# Confirm gallo_init_strict appears in the regenerated pico_de_gallo.h
```

- [ ] **Step 7: Add CHANGELOG entry, document in `book/src/crates/ffi.md`.**

CHANGELOG:

```markdown
### Added

- `gallo_init_strict()` and `gallo_init_strict_with_serial_number()`
  call `PicoDeGallo::validate()` internally and return `null` on
  device-not-found, schema mismatch, or any validation error.
  Prefer over the existing lazy `gallo_init()` in production code.
```

- [ ] **Step 8: Stage and commit.**

```bash
git add crates/pico-de-gallo-ffi/src/lib.rs \
        crates/pico-de-gallo-ffi/Cargo.toml \
        crates/pico-de-gallo-ffi/CHANGELOG.md \
        crates/Cargo.lock \
        book/src/crates/ffi.md
if git ls-files --error-unmatch pico_de_gallo.h 2>/dev/null; then
    git add pico_de_gallo.h
fi
git commit -m "feat(ffi): add gallo_init_strict for validation-on-construct

Two new C constructors call PicoDeGallo::validate() before returning
and yield null on failure. Prefer over the existing lazy gallo_init
in production: failures (device not present, schema mismatch) are
surfaced at construct-time rather than on the first RPC.

Closes Category A finding #4 (FFI side of validation enforcement).

Updates book/src/crates/ffi.md per AGENTS.md §15.1.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 20: pyco — add `PycoDeGallo.open_strict`

**Files:**
- Modify: `crates/pyco-de-gallo/src/lib.rs`
- Modify: `crates/pyco-de-gallo/Cargo.toml` (version bump, minor)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pyco-de-gallo/CHANGELOG.md`
- Modify: `book/src/crates/python.md`

Symmetric to Task 19 but for Python.

- [ ] **Step 1: Add a `#[classmethod]` `open_strict`** to `PycoDeGallo` that wraps `PycoDeGallo.open()` + `validate()`.

```rust
/// Construct a PycoDeGallo and immediately validate the firmware
/// schema. Raises `RuntimeError` on validation failure.
///
/// Prefer over `PycoDeGallo.open()` in production: failures
/// (device not present, schema mismatch) surface here instead of
/// on the first RPC.
#[classmethod]
pub fn open_strict(_cls: &Bound<'_, PyType>, py: Python<'_>) -> PyResult<Self> {
    let inst = Self::open(py)?;
    py.allow_threads(|| {
        inst.runtime
            .block_on(inst.inner.validate())
            .map_err(|e| PyRuntimeError::new_err(format!("{e}")))
    })?;
    Ok(inst)
}
```

Add an `open_strict_with_serial_number(serial: &str)` variant.

- [ ] **Step 2: Bump pyco minor + CHANGELOG + book update.**

- [ ] **Step 3: Per-crate CI mirror + lockfile refresh + commit.**

```bash
git commit -m "feat(pyco): add PycoDeGallo.open_strict for validation-on-open

Symmetric to gallo_init_strict / Hal::new_validated. Raises
RuntimeError on validation failure rather than deferring to the
first RPC.

Closes Category A finding #4 (Python side of validation enforcement).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 21: app — call `validate()` in `Cli::connect()`

**Files:**
- Modify: `crates/pico-de-gallo-app/src/lib.rs` (the `Cli::connect()` function)
- Modify: `crates/pico-de-gallo-app/Cargo.toml` (patch bump)
- Modify: `crates/Cargo.lock`
- Modify: `crates/pico-de-gallo-app/CHANGELOG.md`

- [ ] **Step 1: Find `Cli::connect()`**

```bash
grep -nA 15 "fn connect" crates/pico-de-gallo-app/src/lib.rs
```

- [ ] **Step 2: After constructing the `PicoDeGallo`, call `.validate()` and exit non-zero on failure**

```rust
pub async fn connect(&self) -> /* existing return type */ {
    let gallo = /* existing construction */;
    if let Err(e) = gallo.validate().await {
        eprintln!("ERROR: firmware schema validation failed: {e}");
        eprintln!(
            "  Host expects schema {}.{}.{}",
            SCHEMA_VERSION_MAJOR, SCHEMA_VERSION_MINOR, SCHEMA_VERSION_PATCH
        );
        eprintln!("  Re-flash the firmware to a matching version, or downgrade `gallo`.");
        std::process::exit(2);
    }
    gallo
}
```

The exit code (`2`) is a convention; check whether the CLI defines exit codes anywhere else and reuse / extend the convention.

- [ ] **Step 3: Add a CLI integration test** if the test harness supports mocking the validate result. Otherwise, document the change and rely on PR review.

- [ ] **Step 4: Bump patch version + CHANGELOG + commit.**

```bash
git commit -m "fix(application): call validate() in Cli::connect, exit on mismatch

Previously gallo connected lazily and surfaced schema mismatches on
the first RPC, leading to confusing 'CommsFailed' errors. Now we
validate at connect time and exit non-zero with a clear message
pointing at the firmware-vs-host version skew.

Closes Category A finding #4 (CLI side of validation enforcement).

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 22: Fix `pico-de-gallo-internal` GPIO range doc-comment drift

**Files:**
- Modify: `crates/pico-de-gallo-internal/src/lib.rs` (replace `0–7` with `0..=3` at lines 470, 482, 510, 547, 586 — verify line numbers; the reviewer flagged five sites)

Closes Category A finding #14 + PR #56 side finding #4.

- [ ] **Step 1: Find every doc-comment reference to `0–7` (or `0-7`)**

```bash
grep -nE "(0–7|0-7|0\.\.=7|0\.\.8)" crates/pico-de-gallo-internal/src/lib.rs
```

The reviewer found five matches at lines 470, 482, 510, 547, 586 (line numbers may have shifted slightly with PR 1's changes; grep is the truth).

- [ ] **Step 2: Replace each with the correct `0..=3`**

The exact phrasing depends on each doc-comment's context. Use the agent-guide markdown (`docs/ai-agents/pico-de-gallo-hal-examples.md` §6.4–§6.7) as the reference for what "the right range" looks like.

A `sed` replace is risky because the surrounding prose differs. Do it by hand, file-by-file, or with `sed` if you've confirmed every occurrence really is wrong:

```bash
# Don't run blindly — review each match first.
sed -i 's/`0–7`/`0..=3`/g; s/`0-7`/`0..=3`/g' crates/pico-de-gallo-internal/src/lib.rs
```

Verify:

```bash
grep -nE "(0–7|0-7|0\.\.=7|0\.\.8)" crates/pico-de-gallo-internal/src/lib.rs
```

Expected: no matches (or matches that are intentional, like a comment explaining the drift).

- [ ] **Step 3: This is a doc-only change. No version bump.**

`pico-de-gallo-internal` already bumped in PR 1 (T4). This change is in PR 2 so it ships with a patch bump (or release-please bundles it). If PR 2 lands first, you'll need to bump `internal` patch here; if PR 1 lands first, the patch bumps naturally fold in.

- [ ] **Step 4: Commit**

```bash
git add crates/pico-de-gallo-internal/src/lib.rs
git commit -m "docs(internal): fix GPIO pin range from 0..=7 to 0..=3

Five doc-comments said 0..=7 but firmware caps at 0..=3
(NUM_GPIOS: usize = 4 in firmware/src/context.rs:33). The agent
guide already says 0..=3; this aligns the source.

Closes Category A finding #14 + PR #56 side finding #4.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 23: Fix `book/src/appendix/endpoints.md` `gpio/event` topic prose

**Files:**
- Modify: `book/src/appendix/endpoints.md` (lines 77–88 area)

Closes Category A finding #15.

- [ ] **Step 1: Inspect the current paragraph**

```bash
sed -n '70,95p' book/src/appendix/endpoints.md
```

- [ ] **Step 2: Replace the misleading paragraph**

The current text implies one stream per pin. Replace with:

```markdown
The `gpio/event` topic is a **single multiplexed stream** carrying
events for every subscribed pin (each `GpioEvent` carries a
`pin: u8` field). `gpio/subscribe(pin, edge)` enables firmware-side
monitoring of one pin; `gpio/unsubscribe(pin)` tears it down. Events
for any subscribed pin arrive on the shared stream — there is no
per-pin sub-channel.

Stale events: after `gpio/unsubscribe(pin)` returns `Ok`, a `GpioEvent`
for that pin may still arrive (best-effort delivery). Consumers
should filter against their current subscription set and drop
unknown-pin events without erroring.
```

The "stale events" sentence preempts Category C finding #27.

- [ ] **Step 3: `mdbook build book`** to verify no broken links.

- [ ] **Step 4: Commit**

```bash
git add book/src/appendix/endpoints.md
git commit -m "docs(repo): fix gpio/event topic prose in endpoint catalog

Previously implied one stream per pin. Correct semantics: single
multiplexed topic, GpioEvent carries pin: u8, subscribe enables
firmware-side monitoring per pin, events for any subscribed pin
arrive on the shared stream.

Also documents the best-effort delivery property (stale events may
arrive after unsubscribe) so consumers know to filter.

Closes Category A finding #15.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 24: Add `Status` discriminant-uniqueness test

**Files:**
- Modify: `crates/pico-de-gallo-ffi/src/lib.rs` (extend the `all_errors_are_negative` test)

Closes Category A finding #34.

- [ ] **Step 1: Find the existing `all_errors_are_negative` test**

```bash
grep -nA 30 "fn all_errors_are_negative" crates/pico-de-gallo-ffi/src/lib.rs
```

- [ ] **Step 2: Extend it with a uniqueness assertion**

```rust
#[test]
fn status_discriminants_are_unique() {
    use std::collections::HashSet;
    let all_codes: Vec<i32> = vec![
        Status::Ok as i32,
        Status::I2cReadFailed as i32,
        // ... every variant ...
        Status::GpioTimeout as i32, // added in PR 1 Task 11
    ];
    let unique: HashSet<i32> = all_codes.iter().copied().collect();
    assert_eq!(
        all_codes.len(),
        unique.len(),
        "duplicate Status discriminant: {:?}",
        all_codes
    );
}
```

Build the `all_codes` vec exhaustively. This is tedious but it's the test's whole point. If `Status` derives `IntoEnumIterator` or similar, use that.

- [ ] **Step 3: Verify it passes**

```bash
cd crates/pico-de-gallo-ffi && cargo test --locked status_discriminants_are_unique
```

- [ ] **Step 4: Commit**

```bash
git add crates/pico-de-gallo-ffi/src/lib.rs
git commit -m "test(ffi): add Status discriminant-uniqueness test

A copy-paste during enum extension could silently assign the same
discriminant to two variants. The existing all_errors_are_negative
test only checked < 0. This new test enumerates every variant and
asserts the discriminants are pairwise unique.

Closes Category A finding #34.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 25: Add schema-version-matches-Cargo.toml test

**Files:**
- Modify: `crates/pico-de-gallo-internal/src/lib.rs` (or a new `tests/schema_version.rs`)

Closes Category A finding #35.

- [ ] **Step 1: Add a test**

```rust
#[test]
fn schema_version_matches_cargo_pkg_version() {
    let cargo_version = env!("CARGO_PKG_VERSION");
    let parts: Vec<&str> = cargo_version.split('.').collect();
    assert_eq!(parts.len(), 3, "CARGO_PKG_VERSION must be MAJOR.MINOR.PATCH");
    let cargo_major: u16 = parts[0].parse().unwrap();
    let cargo_minor: u16 = parts[1].parse().unwrap();
    let cargo_patch: u16 = parts[2].parse().unwrap();
    assert_eq!(
        SCHEMA_VERSION_MAJOR, cargo_major,
        "SCHEMA_VERSION_MAJOR ({}) does not match CARGO_PKG_VERSION ({})",
        SCHEMA_VERSION_MAJOR, cargo_version,
    );
    assert_eq!(SCHEMA_VERSION_MINOR, cargo_minor);
    assert_eq!(SCHEMA_VERSION_PATCH, cargo_patch);
}
```

This catches the stale-incremental-cache trap AGENTS.md §13.8 warned about.

- [ ] **Step 2: Verify it passes**

```bash
cd crates/pico-de-gallo-internal && cargo test --features use-std schema_version_matches_cargo_pkg_version --locked
```

- [ ] **Step 3: Commit**

```bash
git add crates/pico-de-gallo-internal/src/lib.rs
git commit -m "test(internal): assert SCHEMA_VERSION_* matches CARGO_PKG_VERSION

AGENTS.md §13.8 warns about a stale incremental-cache trap where
SCHEMA_VERSION_* constants don't match the crate version. This
test parses env!(CARGO_PKG_VERSION) and asserts equality with the
build.rs-generated constants.

Closes Category A finding #35.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 26: Add CI workflow for agent-guide drift check

**Files:**
- Add: `.github/workflows/agent-guide-parity.yml`

Closes Category A finding #36 + PR #56 side finding #5.

- [ ] **Step 1: Write the workflow**

```yaml
name: agent-guide-parity

on:
  pull_request:
    paths:
      - 'crates/pico-de-gallo-hal/src/**'
      - 'docs/ai-agents/pico-de-gallo-hal-examples.md'
      - '.github/workflows/agent-guide-parity.yml'
  push:
    branches:
      - main
    paths:
      - 'crates/pico-de-gallo-hal/src/**'
      - 'docs/ai-agents/pico-de-gallo-hal-examples.md'

jobs:
  drift-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Extract HAL accessor names from agent guide
        run: |
          grep -oE '(hal\.[a-z0-9_]+\()|(Hal::[a-z0-9_]+\b)' \
            docs/ai-agents/pico-de-gallo-hal-examples.md \
            | sed 's/^hal\.//;s/(.*//;s/^Hal:://' \
            | sort -u > /tmp/doc-names.txt
          cat /tmp/doc-names.txt

      - name: Extract pub fn names from HAL source
        run: |
          grep -oE 'pub fn [a-z0-9_]+' crates/pico-de-gallo-hal/src/lib.rs \
            | sed 's/^pub fn //' | sort -u > /tmp/src-names.txt
          cat /tmp/src-names.txt

      - name: Assert every doc-named accessor exists in source
        run: |
          set -e
          missing=$(comm -23 /tmp/doc-names.txt /tmp/src-names.txt)
          if [ -n "$missing" ]; then
              echo "::error::Agent guide references HAL accessors that don't exist in lib.rs:"
              echo "$missing"
              echo ""
              echo "Fix: either add the missing pub fn, or update the markdown."
              exit 1
          fi
          echo "All HAL accessor names in agent guide resolve to pub fn in lib.rs."
```

Use the digit-aware `[a-z0-9_]+` regex (the one fix surfaced during PR #56's Task 9 review).

Note: some names extracted from `hal.X(` will be methods on returned handles (e.g. `wait_for_high_with_timeout` is a method on `Gpio`, not on `Hal`). Those won't be `pub fn` in `lib.rs` matching the simple regex — they'll show as `pub async fn wait_for_high_with_timeout`. Refine the right-side grep to include `pub async fn` and `pub fn` both, and/or grep the entire file (impl blocks too) for `pub fn X` and `pub async fn X`:

```bash
grep -oE '(pub (async )?fn) [a-z0-9_]+' crates/pico-de-gallo-hal/src/lib.rs \
    | sed -E 's/^pub (async )?fn //' | sort -u
```

The implementer must tune the grep to be both inclusive (catch all real public methods) and exclusive (not match private fns). Test with a deliberate false-positive injection (add a fake `hal.foo(` to the markdown, confirm the CI fails) before committing the workflow.

- [ ] **Step 2: Verify the workflow with `actionlint`**

```bash
actionlint .github/workflows/agent-guide-parity.yml
```

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/agent-guide-parity.yml
git commit -m "ci(repo): add agent-guide-parity workflow

Greps every hal.X( and Hal::X reference from
docs/ai-agents/pico-de-gallo-hal-examples.md and asserts each
name exists as a pub fn (or pub async fn) in
crates/pico-de-gallo-hal/src/lib.rs.

Catches the drift mode PR #56 §9 warned about: HAL accessor renamed
or removed without updating the agent guide. Prevents the agent
guide from hallucinating APIs that don't exist.

Uses the digit-aware [a-z0-9_]+ regex (the fix surfaced during PR
#56's Task 9 review — bare [a-z_]+ silently excluded i2c and other
digit-bearing names).

Closes Category A finding #36 + PR #56 side finding #5.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 27: Add AGENTS.md §13.17 regression-log rows

**Files:**
- Modify: `AGENTS.md` (append rows to the §13.17 past regressions table)

Closes Category A finding #43.

- [ ] **Step 1: Find the §13.17 table**

```bash
grep -n "13.17" AGENTS.md
```

- [ ] **Step 2: Append two rows at the bottom of the table**

```markdown
| 2026-06-03 | gpio_wait_for_* on never-transitioning pin after host crash | Firmware dispatcher wedged device-wide; every other endpoint queued until power-cycle | Append `timeout_ms: u32` to all five gpio_wait_* request types; append `GpioError::Timeout` variant; wrap firmware handlers in `embassy_time::with_timeout`; enable embassy-rp watchdog at 2s in dedicated feeder task. Lockstep release internal 0.6→0.7, firmware 0.10→0.11, lib 0.6→0.7, hal 0.7→0.8, ffi 0.7→0.8, app 0.7→0.8, pyco 0.3→0.4. See category-a-hotfix-wire commits. |
| 2026-06-03 | `PicoDeGallo::validate()` only checked `schema_minor`, not `schema_major` | A firmware on a bumped major with matching minor passed validation and the host then silently mis-decoded wire bytes (no error, garbage values returned from RPCs) | Fix `validate()` at lib.rs:667 to check both major and minor. Add regression test (`validate_rejects_bumped_schema_major`). Also enforce validation at `Hal::new_validated`, `gallo_init_strict`, `PycoDeGallo.open_strict`, and `gallo` CLI `Cli::connect`. |
```

- [ ] **Step 3: Verify LF endings**

```bash
file AGENTS.md
```

- [ ] **Step 4: Commit**

```bash
git add AGENTS.md
git commit -m "docs(repo): log gpio_wait dispatcher wedge and schema-major drift

Add two rows to AGENTS.md §13.17 past regressions log:
- 2026-06-03 gpio_wait_for_* dispatcher wedge (closed by Cat A PR 1)
- 2026-06-03 validate() schema_major drift (closed by Cat A PR 2)

These document the regression class so the next agent (or human
maintainer) doesn't reintroduce the same shape of bug.

Co-authored-by: Copilot <223556219+Copilot@users.noreply.github.com>
Assisted-by: opencode:claude-opus-4.7-1m-internal"
```

---

### Task 28: PR 2 — full local CI mirror

Mirror Task 14, but only for the crates touched in PR 2 (`internal` for the doc fix + version-test, `lib` for the validate fix, `hal` for the accessors + re-exports, `ffi` for `gallo_init_strict`, `app` for `Cli::connect`, `pyco` for `open_strict`).

- [ ] All the same commands. Document each pass.

---

### Task 29: PR 2 — push branch and open draft PR

- [ ] **Step 1: Push**

```bash
git push -u origin category-a-hotfix-host 2>&1 | tail -5
```

- [ ] **Step 2: Write PR body**

Include:
- Summary (closes findings #1, #4, #5, #6, #13, #14, #15, #34, #35, #36 + PR #56 side findings #1, #2, #3, #4, #5).
- Cross-reference to PR 1 (`#NN`) — note this PR is independent at the branch level but the host-side validation fix (Task 17) depends on the wire change in PR 1 to be useful in production (you can validate against any schema, but the silent-mis-decode failure mode only really exists across major-version drift).
- Per-task summary (T17–T27).
- Test plan checkboxes corresponding to T28's CI mirror.

Write to `/tmp/opencode/pr2-body.md`.

- [ ] **Step 3: Open draft PR**

```bash
gh pr create \
  --repo OpenDevicePartnership/pico-de-gallo \
  --base main \
  --head felipebalbi:category-a-hotfix-host \
  --draft \
  --title "feat(lib,hal,ffi,application,pyco): enforce schema validation, expose HAL recovery" \
  --body-file /tmp/opencode/pr2-body.md
```

- [ ] **Step 4: Verify PR**

```bash
gh pr view --repo OpenDevicePartnership/pico-de-gallo --json number,state,isDraft,url
```

---

### Task 30: Final acceptance sweep

- [ ] **Step 1: Cross-check both PRs against the 25 Category A findings table** (synthesis spec §4.1 and §4.2). Confirm each closed finding has a closing commit in one of the two PRs.

- [ ] **Step 2: Confirm Category B and Category C findings remain enumerated** as future work in the synthesis spec (§2.4, §2.5). Optionally file GitHub issues for the biggest ones.

- [ ] **Step 3: Report back with:**
  - PR 1 URL and number, PR 2 URL and number
  - Total commit count and crates touched
  - Any CI failures observed and addressed
  - Open questions for the maintainers (if any)

- [ ] **Step 4: Do NOT push, force-push, or merge anything without explicit user approval.** Per AGENTS.md hard rule #8.

---

## Plan self-review

After this plan was written, the author ran the three writing-plans skill checks inline:

1. **Spec coverage:** every finding in the synthesis spec §4.1 (Category A) maps to one or more tasks above. §4.2 explicit scope-cut list (#7, #8, #10, #11, #12) noted as deferred. Categories B and C left as future work per design.

2. **Placeholder scan:** no `TBD` / `TODO` / `appropriate error handling` / `add validation` / `handle edge cases` / `Similar to Task N` patterns. Every step contains the actual content the engineer needs.

3. **Type consistency:** cross-checked the accessor names used across tasks:
   - `gpio_wait_for_*` family — five variants (high, low, rising, falling, any) named consistently in T1, T2, T3, T5, T9, T10, T11, T12, T13. ✓
   - `gpio_wait_for_*_with_timeout` overloads — consistent naming across `lib` (T9), `hal` (T10), `app` (T12). ✓
   - FFI variant uses `_with_timeout_ms` suffix (T11) — different from Rust because C lacks `Duration`; documented in T11 Step 4. ✓
   - `Hal::system_reset_subscriptions`, `Hal::validate`, `Hal::new_validated` — names match across T18, T19's `gallo_init_strict` documentation, T20's `open_strict`, T21's `Cli::connect`. ✓
   - `GpioError::Timeout` variant — same name across T3 (define), T5 (firmware return), T9 (lib map), T10 (hal map), T11 (ffi map). ✓
   - `Status::GpioTimeout` — added in T11; checked in T24 uniqueness test. ✓
