# Changelog

All notable changes to `gallo` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.1] — 2026-07-20

### Fixed

- `gallo` now opens a **single** USB connection per invocation and
  shares it (by reference) across schema validation and the command
  handler. Previously every subcommand except `list`/`version`
  opened one connection to run `validate()`, dropped it, then opened
  a second connection for the operation (`spi write-read` opened a
  third). On Windows, WinUSB grants exclusive access to one session
  per interface, and the first connection's background `nusb` worker
  had not released the handle before the second `claim_interface`,
  so the operation panicked with
  `Failed claiming interface: … Access is denied`. Commands such as
  `gallo i2c scan`, `i2c get-config`, and `adc info` failed
  deterministically on Windows, while `version`/`list` (single/zero
  connections) worked — making it look like a driver or permissions
  problem. Linux and macOS release the interface synchronously on
  drop, so CI never caught it. Regression from the 2026-06-04
  up-front `validate()` change (Category A finding #4). No CLI
  surface changed.

## [0.7.0] — 2026-06-22

### Fixed (2026-06-04 — Category A hotfix host-only PR)

- `gallo` now calls `validate()` at the top of every subcommand
  except `list` and `version`. Previously the CLI connected
  lazily and surfaced schema-version mismatches as confusing
  `CommsFailed` errors on the first RPC; now the mismatch is
  reported up-front with an actionable error message that
  points at `gallo version` for the device-reported schema and
  recommends either re-flashing the firmware or installing a
  matching `gallo` build. Closes Category A finding #4 (reviewer
  R4) at the CLI layer.

  `list` is exempt because it doesn't touch a connected device.
  `version` is exempt because it IS the diagnostic subcommand
  that reports schema skew (it already handles legacy firmware
  via `device_info()` with fallback).

### Changed (2026-06-04 — Category A hotfix host-only PR)

- Bumped `pico-de-gallo-lib` dependency to 0.6.0 (validate() now
  also checks `schema_major`, so any future major-version skew
  surfaces immediately rather than silently mis-decoding wire
  bytes).

### Changed (2026-06-03 — Category A hotfix wire PR)

- Bumped `pico-de-gallo-lib` dependency to 0.6.0. Required for
  lockstep release with the wire-protocol schema bump in
  `pico-de-gallo-internal` 0.6.0 / `pico-de-gallo-firmware` 0.10.0
  (`timeout_ms` field on `GpioWaitRequest`, `GpioError::Timeout`
  variant).
- Existing `gallo` CLI behavior is unchanged in this release: the
  pre-existing `gpio` subcommands (`get`, `put`, `set-config`,
  `monitor`) all keep working. The CLI does not currently expose
  `gpio wait-for-*` subcommands, so no new flags are added here.
  Bounded waits remain accessible to Rust / C / Python consumers
  via `pico-de-gallo-lib`, `pico-de-gallo-hal`, and `pico-de-gallo-ffi`.

## [0.6.0] — 2026-05-04

### Added

- `gallo version` now shows schema version, HW revision, and
  capabilities with graceful fallback for legacy firmware.

## [0.5.0] — 2026-04-22

### Added

- `gallo gpio monitor --pin N --edge rising|falling|any` command.
  Subscribes, prints edge events with timestamps, unsubscribes on
  Ctrl+C.
- `gallo i2c batch` and `gallo spi batch` CLI commands for
  executing batched operations (e.g.,
  `--op write:0x00,0x10 --op read:16`).
- `gallo pwm` subcommand group with `set-duty`, `get-duty`,
  `enable`, `disable`, `set-config`, and `get-config` commands.
- `gallo adc` subcommand group with `read` and `info` commands.
- `gallo onewire` subcommand group with `reset`, `read`, `write`,
  `write-pullup`, and `search` commands.
- `gallo uart` subcommand group with `read`, `write`, `flush`,
  `set-config`, and `get-config` commands.
- `gallo i2c scan` now uses the dedicated scan endpoint (single
  round-trip) instead of 112 individual reads.
- `gallo gpio set-config`, `gallo gpio get`, and `gallo gpio put`
  subcommands for direct GPIO access from the command line.
- `gallo i2c get-config` and `gallo spi get-config` subcommands.

## [0.4.0] — 2025-04-20

### Breaking Changes

- CLI `set-config` command replaced by `i2c set-config` and
  `spi set-config` subcommands.

### Added

- `list` command to show connected devices with serial numbers.

### Changed

- `I2cFrequency` exposed as `--frequency standard|fast|fast-plus`
  CLI arg.

## [0.2.1] — 2025-03-15

### Fixed

- Bumped library dependency for latest fixes.
