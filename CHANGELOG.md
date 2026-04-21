# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **lib**: Corrected `MAX_TRANSFER_SIZE` references in rustdoc for `i2c_read`,
  `i2c_write_read`, and `spi_read` (was 512, actual value is 4096)

## [0.7.0] — 2025-04-20

### Breaking Changes

- **internal 0.3.0**: Split `SetConfigurationRequest` into `I2cSetConfigurationRequest`
  and `SpiSetConfigurationRequest`
- **internal 0.3.0**: Replaced raw `u32` I2C frequency with `I2cFrequency` enum
  (`Standard`, `Fast`, `FastPlus`)
- **lib 0.3.0**: Split `set_config()` into `i2c_set_config()` and `spi_set_config()`
- **lib 0.3.0**: `PicoDeGalloError` is now generic over the endpoint error type
- **hal 0.3.0**: Split `set_config()` into `i2c_set_config()` and `spi_set_config()`
- **ffi 0.4.0**: Split `gallo_set_config()` into `gallo_i2c_set_config()` and
  `gallo_spi_set_config()`
- **app 0.4.0**: CLI `set-config` command replaced by `i2c set-config` and
  `spi set-config` subcommands
- **firmware 0.7.0**: Wire protocol updated — firmware and host must be upgraded together

### Added

- **spi**: Full-duplex transfer endpoint (`spi/transfer`) using DMA
- **lib**: `list_devices()` function for enumerating connected boards
- **app**: `list` command to show connected devices with serial numbers
- **lib**: `Display` and `std::error::Error` implementations for `PicoDeGalloError`
- **internal**: `From<bool>` / `Into<bool>` conversions for `GpioState`
- **internal**: `MAX_TRANSFER_SIZE` constant (4096 bytes) shared across crates
- **ffi**: Compile-time `Send + Sync` assertion for thread safety
- **hal**: Per-call async context detection (reuses existing tokio runtime if available)
- **docs**: Comprehensive rustdoc documentation across all crates
- **docs**: Repository-level Copilot instructions (`.github/copilot-instructions.md`)
- **ci**: Fixed Windows release asset naming (`.dll` extension)

### Changed

- **firmware**: Handler functions modernized with improved ergonomics
- **firmware**: Buffer increased to `MAX_TRANSFER_SIZE` (4096 bytes)
- **firmware**: `PacketBuffers` sized to `MAX_TRANSFER_SIZE + 1024` per direction
- **lib**: `client` field made private (was accidentally public)
- **app**: `I2cFrequency` exposed as `--frequency standard|fast|fast-plus` CLI arg

## [firmware-v0.6.0] — 2025-03-15

### Added

- Updated all Embassy and postcard-rpc dependencies
- Addressed critical safety issues and improved API ergonomics
- Added more tests and extracted `connect()` helper

## [application-v0.2.1] — 2025-03-15

### Fixed

- Bumped library dependency for latest fixes

## [ffi-v0.3.0] — 2025-03-15

### Changed

- Updated dependencies to match library changes

## [hal-v0.2.0] — 2025-03-15

### Changed

- Updated dependencies and API to match library

---

[Unreleased]: https://github.com/OpenDevicePartnership/pico-de-gallo/compare/firmware-v0.7.0...HEAD
[0.7.0]: https://github.com/OpenDevicePartnership/pico-de-gallo/compare/firmware-v0.6.0...firmware-v0.7.0
[firmware-v0.6.0]: https://github.com/OpenDevicePartnership/pico-de-gallo/releases/tag/firmware-v0.6.0
[application-v0.2.1]: https://github.com/OpenDevicePartnership/pico-de-gallo/releases/tag/application-v0.2.1
[ffi-v0.3.0]: https://github.com/OpenDevicePartnership/pico-de-gallo/releases/tag/ffi-v0.3.0
[hal-v0.2.0]: https://github.com/OpenDevicePartnership/pico-de-gallo/releases/tag/hal-v0.2.0
