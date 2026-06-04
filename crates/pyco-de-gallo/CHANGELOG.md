# Changelog

All notable changes to `pyco-de-gallo` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added (2026-06-03 — Category A hotfix)

- `PycoDeGallo.gpio_wait_for_{high,low,rising_edge,falling_edge,any_edge}_with_timeout`
  Python methods accept a `timeout_ms: int`. 0 waits forever
  (matches the existing methods); non-zero bounds the wait and
  raises `RuntimeError` on `GpioError::Timeout`. Available on
  firmware schema 0.7+.

### Changed (2026-06-03 — Category A hotfix)

- Bumped `pico-de-gallo-lib` dependency to 0.7.0. Lockstep release
  with `pico-de-gallo-internal` 0.7.0 / `pico-de-gallo-firmware`
  0.11.0 per AGENTS.md §6.5.

### Added

- `PycoDeGallo.system_reset_subscriptions()` returns an `int`.

## [0.2.0] — 2026-05-04

### Added

- `pyco-de-gallo` is now part of the `check.yml` CI matrix (fmt,
  clippy, doc, hack, test, msrv) on equal footing with the other
  host crates.
