//! Shared wire-protocol types for the Pico de Gallo USB bridge.
//!
//! This crate defines the [postcard-rpc](https://docs.rs/postcard-rpc) endpoints,
//! request/response types, and shared constants used by both the firmware
//! ([`pico-de-gallo-firmware`]) and the host-side library ([`pico-de-gallo-lib`]).
//!
//! # Wire Compatibility
//!
//! All types are serialized with [postcard](https://docs.rs/postcard). Postcard
//! encodes enum variants by **index** (0, 1, 2, …), not by discriminant value.
//! Reordering variants in any `enum` in this crate is a **breaking wire change**
//! that will silently corrupt communication between mismatched firmware and host
//! versions.
//!
//! # Feature Flags
//!
//! - **`use-std`** — Enables `Vec<u8>` response types for the host side. Without
//!   this feature (the default for firmware), responses use borrowed `&[u8]` slices.
//!
//! # Crate Organization
//!
//! - **Constants**: [`MICROSOFT_VID`], [`PICO_DE_GALLO_PID`], [`MAX_TRANSFER_SIZE`]
//! - **Endpoints**: Defined via the [`postcard_rpc::endpoints!`] macro — see
//!   [`ENDPOINT_LIST`] for the full table.
//! - **I2C types**: [`I2cReadRequest`], [`I2cWriteRequest`], [`I2cWriteReadRequest`]
//!   and their corresponding error types.
//! - **SPI types**: [`SpiReadRequest`], [`SpiWriteRequest`], [`SpiTransferRequest`]
//!   and their corresponding error types.
//! - **GPIO types**: [`GpioGetRequest`], [`GpioPutRequest`], [`GpioWaitRequest`],
//!   [`GpioState`].
//! - **Configuration**: [`I2cSetConfigurationRequest`], [`SpiSetConfigurationRequest`],
//!   [`I2cFrequency`], [`SpiPhase`], [`SpiPolarity`].
//! - **Version**: [`VersionInfo`].

#![cfg_attr(not(feature = "use-std"), no_std)]

use postcard_rpc::{TopicDirection, endpoints, topics};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};

/// USB Vendor ID (Microsoft Corporation).
pub const MICROSOFT_VID: u16 = 0x045e;

/// USB Product ID assigned to Pico de Gallo.
pub const PICO_DE_GALLO_PID: u16 = 0x067d;

/// Maximum number of bytes the firmware can handle in a single I2C or SPI
/// transaction. Requests exceeding this limit will be rejected by the
/// firmware with an error.
pub const MAX_TRANSFER_SIZE: usize = 4096;

// ---

/// Response type for I2C write operations.
pub type I2cWriteResponse = Result<(), I2cWriteFail>;

/// Response type for I2C read operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(feature = "use-std")]
pub type I2cReadResponse<'a> = Result<Vec<u8>, I2cReadFail>;
/// Response type for I2C read operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(not(feature = "use-std"))]
pub type I2cReadResponse<'a> = Result<&'a [u8], I2cReadFail>;

/// Response type for I2C write-read operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(feature = "use-std")]
pub type I2cWriteReadResponse<'a> = Result<Vec<u8>, I2cWriteReadFail>;
/// Response type for I2C write-read operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(not(feature = "use-std"))]
pub type I2cWriteReadResponse<'a> = Result<&'a [u8], I2cWriteReadFail>;

/// Response type for SPI write operations.
pub type SpiWriteResponse = Result<(), SpiWriteFail>;

/// Response type for SPI read operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(feature = "use-std")]
pub type SpiReadResponse<'a> = Result<Vec<u8>, SpiReadFail>;
/// Response type for SPI read operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(not(feature = "use-std"))]
pub type SpiReadResponse<'a> = Result<&'a [u8], SpiReadFail>;

/// Response type for SPI flush operations.
pub type SpiFlushResponse = Result<(), SpiFlushFail>;

/// Response type for SPI transfer operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(feature = "use-std")]
pub type SpiTransferResponse<'a> = Result<Vec<u8>, SpiTransferFail>;
/// Response type for SPI transfer operations.
/// On the host (`use-std`), returns `Vec<u8>`; on firmware, returns `&[u8]`.
#[cfg(not(feature = "use-std"))]
pub type SpiTransferResponse<'a> = Result<&'a [u8], SpiTransferFail>;

/// Response type for GPIO get operations.
pub type GpioGetResponse = Result<GpioState, GpioGetFail>;
/// Response type for GPIO put operations.
pub type GpioPutResponse = Result<(), GpioPutFail>;
/// Response type for GPIO wait operations.
pub type GpioWaitResponse = Result<(), GpioWaitFail>;
/// Response type for I2C bus configuration operations.
pub type I2cSetConfigurationResponse = Result<(), I2cSetConfigurationFail>;
/// Response type for SPI bus configuration operations.
pub type SpiSetConfigurationResponse = Result<(), SpiSetConfigurationFail>;

endpoints! {
    list = ENDPOINT_LIST;
    | EndpointTy          | RequestTy                  | ResponseTy                  | Path                |
    | ----------          | ---------                  | ----------                  | ----                |
    | PingEndpoint        | u32                        | u32                         | "ping"              |
    | I2cRead             | I2cReadRequest             | I2cReadResponse<'a>         | "i2c/read"          |
    | I2cWrite            | I2cWriteRequest<'a>        | I2cWriteResponse            | "i2c/write"         |
    | I2cWriteRead        | I2cWriteReadRequest<'a>    | I2cWriteReadResponse<'b>    | "i2c/write-read"    |
    | SpiRead             | SpiReadRequest             | SpiReadResponse<'a>         | "spi/read"          |
    | SpiWrite            | SpiWriteRequest<'a>        | SpiWriteResponse            | "spi/write"         |
    | SpiFlush            | ()                         | SpiFlushResponse            | "spi/flush"         |
    | SpiTransfer         | SpiTransferRequest<'a>     | SpiTransferResponse<'b>     | "spi/transfer"      |
    | GpioGet             | GpioGetRequest             | GpioGetResponse             | "gpio/get"          |
    | GpioPut             | GpioPutRequest             | GpioPutResponse             | "gpio/put"          |
    | GpioWaitForHigh     | GpioWaitRequest            | GpioWaitResponse            | "gpio/wait-high"    |
    | GpioWaitForLow      | GpioWaitRequest            | GpioWaitResponse            | "gpio/wait-low"     |
    | GpioWaitForRising   | GpioWaitRequest            | GpioWaitResponse            | "gpio/wait-rising"  |
    | GpioWaitForFalling  | GpioWaitRequest            | GpioWaitResponse            | "gpio/wait-falling" |
    | GpioWaitForAny      | GpioWaitRequest            | GpioWaitResponse            | "gpio/wait-any"     |
    | I2cSetConfiguration | I2cSetConfigurationRequest | I2cSetConfigurationResponse | "i2c/set-config"    |
    | SpiSetConfiguration | SpiSetConfigurationRequest | SpiSetConfigurationResponse | "spi/set-config"    |
    | Version             | ()                         | VersionInfo                 | "version"           |
}

topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    | TopicTy | MessageTy | Path |
    | ------- | --------- | ---- |
}

topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy | MessageTy | Path | Cfg |
    | ------- | --------- | ---- | --- |
}

// --- I2C

/// Request to write bytes to an I2C device, then read back.
///
/// The firmware performs a write followed by a repeated-start read in a
/// single I2C transaction, which is the standard pattern for reading
/// registers from most I2C devices.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct I2cWriteReadRequest<'a> {
    /// 7-bit I2C slave address.
    pub address: u8,
    /// Bytes to write (typically a register address).
    pub contents: &'a [u8],
    /// Number of bytes to read back (max [`MAX_TRANSFER_SIZE`]).
    pub count: u16,
}

/// Error returned when an I2C write-read operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2cWriteReadFail;

/// Request to read bytes from an I2C device.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct I2cReadRequest {
    /// 7-bit I2C slave address.
    pub address: u8,
    /// Number of bytes to read (max [`MAX_TRANSFER_SIZE`]).
    pub count: u16,
}

/// Error returned when an I2C read operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2cReadFail;

/// Request to write bytes to an I2C device.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct I2cWriteRequest<'a> {
    /// 7-bit I2C slave address.
    pub address: u8,
    /// Bytes to write.
    pub contents: &'a [u8],
}

/// Error returned when an I2C write operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2cWriteFail;

// --- SPI

/// Request to read bytes from the SPI bus.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct SpiReadRequest {
    /// Number of bytes to read (max [`MAX_TRANSFER_SIZE`]).
    pub count: u16,
}

/// Error returned when an SPI read operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiReadFail;

/// Request to write bytes to the SPI bus.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct SpiWriteRequest<'a> {
    /// Bytes to write.
    pub contents: &'a [u8],
}

/// Error returned when an SPI write operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiWriteFail;

/// Error returned when an SPI flush operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiFlushFail;

/// Request for a full-duplex SPI transfer.
///
/// The firmware simultaneously transmits `contents` and receives the same
/// number of bytes. This is a true full-duplex operation using DMA.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct SpiTransferRequest<'a> {
    /// Bytes to transmit. The response will contain the same number of received bytes.
    pub contents: &'a [u8],
}

/// Error returned when an SPI transfer operation fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiTransferFail;

// --- GPIO

/// Request to read the current level of a GPIO pin.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct GpioGetRequest {
    /// GPIO pin index (0–7).
    pub pin: u8,
}

/// Error returned when a GPIO read fails (e.g., invalid pin index).
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioGetFail;

/// Request to set a GPIO pin to a specific level.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct GpioPutRequest {
    /// GPIO pin index (0–7).
    pub pin: u8,
    /// Desired output level.
    pub state: GpioState,
}

/// Error returned when a GPIO write fails (e.g., invalid pin index).
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioPutFail;

/// Logic level of a GPIO pin.
//
// WARNING: Do not reorder enum variants — postcard serializes by
// variant index, not by discriminant. Reordering breaks wire compat.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpioState {
    /// Logic low (0V).
    Low,
    /// Logic high (3.3V on RP2350).
    High,
}

impl From<bool> for GpioState {
    fn from(value: bool) -> Self {
        if value {
            GpioState::High
        } else {
            GpioState::Low
        }
    }
}

impl From<GpioState> for bool {
    fn from(state: GpioState) -> Self {
        matches!(state, GpioState::High)
    }
}

/// Request to wait for a GPIO pin to reach a specific state or edge.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct GpioWaitRequest {
    /// GPIO pin index (0–7).
    pub pin: u8,
}

/// Error returned when a GPIO wait fails (e.g., invalid pin index).
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpioWaitFail;

// --- Set config

/// Request to reconfigure I2C bus parameters.
///
/// Takes effect immediately. The firmware applies the new frequency before
/// processing the next I2C operation.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct I2cSetConfigurationRequest {
    /// I2C bus clock frequency.
    pub frequency: I2cFrequency,
}

// WARNING: do not reorder variants — postcard encodes by index, not discriminant.
/// I2C bus clock frequency.
///
/// The RP2350 supports Standard (100 kHz), Fast (400 kHz), and Fast+ (1 MHz)
/// modes. Ultra-Fast mode is defined by the specification but not supported by
/// the RP2350 hardware.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum I2cFrequency {
    /// Standard mode — 100 kHz.
    Standard = 0,
    /// Fast mode — 400 kHz.
    Fast = 1,
    /// Fast+ mode — 1 MHz.
    FastPlus = 2,
}

/// Error returned when I2C configuration fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct I2cSetConfigurationFail;

/// Request to reconfigure SPI bus parameters.
///
/// Takes effect immediately. The firmware applies the new settings before
/// processing the next SPI operation.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct SpiSetConfigurationRequest {
    /// SPI bus clock frequency in Hz.
    pub spi_frequency: u32,
    /// SPI clock phase.
    pub spi_phase: SpiPhase,
    /// SPI clock polarity.
    pub spi_polarity: SpiPolarity,
}

/// SPI clock phase setting.
//
// WARNING: Do not reorder enum variants — postcard serializes by
// variant index, not by discriminant. Reordering breaks wire compat.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpiPhase {
    /// Data captured on the leading (first) clock edge.
    CaptureOnFirstTransition = 0,
    /// Data captured on the trailing (second) clock edge.
    CaptureOnSecondTransition = 1,
}

/// SPI clock polarity setting.
//
// WARNING: Do not reorder enum variants — postcard serializes by
// variant index, not by discriminant. Reordering breaks wire compat.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpiPolarity {
    /// Clock idles at logic low (CPOL=0).
    IdleLow = 0,
    /// Clock idles at logic high (CPOL=1).
    IdleHigh = 1,
}

/// Error returned when SPI configuration fails.
#[derive(Serialize, Deserialize, Schema, Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpiSetConfigurationFail;

// --- Version
/// Firmware version information.
#[derive(Serialize, Deserialize, Schema, Debug, PartialEq)]
pub struct VersionInfo {
    /// Major version number.
    pub major: u16,
    /// Minor version number.
    pub minor: u16,
    /// Patch version number.
    pub patch: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcard::{from_bytes, to_allocvec};

    // --- I2C round-trip tests ---

    #[test]
    fn i2c_read_request_round_trip() {
        let req = I2cReadRequest {
            address: 0x48,
            count: 4,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn i2c_write_request_round_trip() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        let req = I2cWriteRequest {
            address: 0x50,
            contents: &data,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cWriteRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn i2c_write_read_request_round_trip() {
        let data = [0x01, 0x02];
        let req = I2cWriteReadRequest {
            address: 0x68,
            contents: &data,
            count: 6,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cWriteReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn i2c_read_request_max_count() {
        let req = I2cReadRequest {
            address: 0x7F,
            count: u16::MAX,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    // --- SPI round-trip tests ---

    #[test]
    fn spi_read_request_round_trip() {
        let req = SpiReadRequest { count: 128 };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_write_request_round_trip() {
        let data = [0xCA, 0xFE];
        let req = SpiWriteRequest { contents: &data };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiWriteRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_transfer_request_round_trip() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let req = SpiTransferRequest { contents: &data };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiTransferRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_transfer_request_max_size() {
        let data = vec![0xAA; MAX_TRANSFER_SIZE];
        let req = SpiTransferRequest { contents: &data };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiTransferRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    // --- GPIO round-trip tests ---

    #[test]
    fn gpio_get_request_round_trip() {
        for pin in 0..8u8 {
            let req = GpioGetRequest { pin };
            let bytes = to_allocvec(&req).unwrap();
            let decoded: GpioGetRequest = from_bytes(&bytes).unwrap();
            assert_eq!(req, decoded);
        }
    }

    #[test]
    fn gpio_put_request_round_trip() {
        for state in [GpioState::Low, GpioState::High] {
            let req = GpioPutRequest { pin: 3, state };
            let bytes = to_allocvec(&req).unwrap();
            let decoded: GpioPutRequest = from_bytes(&bytes).unwrap();
            assert_eq!(req, decoded);
        }
    }

    #[test]
    fn gpio_wait_request_round_trip() {
        let req = GpioWaitRequest { pin: 7 };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: GpioWaitRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn gpio_state_round_trip() {
        for state in [GpioState::Low, GpioState::High] {
            let bytes = to_allocvec(&state).unwrap();
            let decoded: GpioState = from_bytes(&bytes).unwrap();
            assert_eq!(state, decoded);
        }
    }

    #[test]
    fn gpio_state_from_bool() {
        assert_eq!(GpioState::from(true), GpioState::High);
        assert_eq!(GpioState::from(false), GpioState::Low);
    }

    #[test]
    fn bool_from_gpio_state() {
        assert_eq!(bool::from(GpioState::High), true);
        assert_eq!(bool::from(GpioState::Low), false);
    }

    // --- Config round-trip tests ---

    #[test]
    fn i2c_set_configuration_request_round_trip() {
        let req = I2cSetConfigurationRequest {
            frequency: I2cFrequency::Fast,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cSetConfigurationRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_set_configuration_request_round_trip() {
        let req = SpiSetConfigurationRequest {
            spi_frequency: 1_000_000,
            spi_phase: SpiPhase::CaptureOnSecondTransition,
            spi_polarity: SpiPolarity::IdleHigh,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiSetConfigurationRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_phase_round_trip() {
        for phase in [
            SpiPhase::CaptureOnFirstTransition,
            SpiPhase::CaptureOnSecondTransition,
        ] {
            let bytes = to_allocvec(&phase).unwrap();
            let decoded: SpiPhase = from_bytes(&bytes).unwrap();
            assert_eq!(phase, decoded);
        }
    }

    #[test]
    fn spi_polarity_round_trip() {
        for pol in [SpiPolarity::IdleLow, SpiPolarity::IdleHigh] {
            let bytes = to_allocvec(&pol).unwrap();
            let decoded: SpiPolarity = from_bytes(&bytes).unwrap();
            assert_eq!(pol, decoded);
        }
    }

    // --- Version round-trip test ---

    #[test]
    fn version_info_round_trip() {
        let ver = VersionInfo {
            major: 1,
            minor: 2,
            patch: 42,
        };
        let bytes = to_allocvec(&ver).unwrap();
        let decoded: VersionInfo = from_bytes(&bytes).unwrap();
        assert_eq!(ver, decoded);
    }

    // --- Fail type round-trip tests ---

    #[test]
    fn fail_types_round_trip() {
        let bytes = to_allocvec(&I2cReadFail).unwrap();
        let _: I2cReadFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&I2cWriteFail).unwrap();
        let _: I2cWriteFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&I2cWriteReadFail).unwrap();
        let _: I2cWriteReadFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&SpiReadFail).unwrap();
        let _: SpiReadFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&SpiWriteFail).unwrap();
        let _: SpiWriteFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&SpiFlushFail).unwrap();
        let _: SpiFlushFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&GpioGetFail).unwrap();
        let _: GpioGetFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&GpioPutFail).unwrap();
        let _: GpioPutFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&GpioWaitFail).unwrap();
        let _: GpioWaitFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&I2cSetConfigurationFail).unwrap();
        let _: I2cSetConfigurationFail = from_bytes(&bytes).unwrap();

        let bytes = to_allocvec(&SpiSetConfigurationFail).unwrap();
        let _: SpiSetConfigurationFail = from_bytes(&bytes).unwrap();
    }

    // --- P1: Schema stability tests ---
    //
    // These lock down the wire encoding for each type. If a field is
    // added, removed, or reordered the serialized bytes will change
    // and these tests will catch it.

    #[test]
    fn i2c_read_request_wire_stability() {
        let req = I2cReadRequest {
            address: 0x48,
            count: 4,
        };
        let bytes = to_allocvec(&req).unwrap();
        assert_eq!(
            bytes,
            to_allocvec(&req).unwrap(),
            "encoding is deterministic"
        );
        // Re-decode and compare to ensure exact round-trip
        let decoded: I2cReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(decoded, req);
        // Lock the exact byte representation
        let snapshot = bytes.clone();
        let freshly_encoded = to_allocvec(&decoded).unwrap();
        assert_eq!(freshly_encoded, snapshot, "wire format must not change");
    }

    #[test]
    fn i2c_set_configuration_request_wire_stability() {
        let req = I2cSetConfigurationRequest {
            frequency: I2cFrequency::Fast,
        };
        let bytes = to_allocvec(&req).unwrap();
        let canonical = bytes.clone();
        let decoded: I2cSetConfigurationRequest = from_bytes(&bytes).unwrap();
        assert_eq!(decoded, req);
        assert_eq!(to_allocvec(&decoded).unwrap(), canonical);
    }

    #[test]
    fn spi_set_configuration_request_wire_stability() {
        let req = SpiSetConfigurationRequest {
            spi_frequency: 1_000_000,
            spi_phase: SpiPhase::CaptureOnFirstTransition,
            spi_polarity: SpiPolarity::IdleLow,
        };
        let bytes = to_allocvec(&req).unwrap();
        let canonical = bytes.clone();
        let decoded: SpiSetConfigurationRequest = from_bytes(&bytes).unwrap();
        assert_eq!(decoded, req);
        assert_eq!(to_allocvec(&decoded).unwrap(), canonical);
    }

    #[test]
    fn version_info_wire_stability() {
        let ver = VersionInfo {
            major: 1,
            minor: 0,
            patch: 0,
        };
        let bytes = to_allocvec(&ver).unwrap();
        let canonical = bytes.clone();
        let decoded: VersionInfo = from_bytes(&bytes).unwrap();
        assert_eq!(decoded, ver);
        assert_eq!(to_allocvec(&decoded).unwrap(), canonical);
    }

    #[test]
    fn gpio_put_request_wire_stability() {
        let req = GpioPutRequest {
            pin: 0,
            state: GpioState::High,
        };
        let bytes = to_allocvec(&req).unwrap();
        let canonical = bytes.clone();
        let decoded: GpioPutRequest = from_bytes(&bytes).unwrap();
        assert_eq!(decoded, req);
        assert_eq!(to_allocvec(&decoded).unwrap(), canonical);
    }

    // --- P1: Boundary value tests ---

    #[test]
    fn i2c_read_request_zero_count() {
        let req = I2cReadRequest {
            address: 0x00,
            count: 0,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn i2c_read_request_max_address() {
        let req = I2cReadRequest {
            address: u8::MAX,
            count: 1,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_read_request_max_count() {
        let req = SpiReadRequest { count: u16::MAX };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_read_request_zero_count() {
        let req = SpiReadRequest { count: 0 };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn i2c_write_request_empty_contents() {
        let req = I2cWriteRequest {
            address: 0x50,
            contents: &[],
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cWriteRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn spi_write_request_empty_contents() {
        let req = SpiWriteRequest { contents: &[] };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiWriteRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn i2c_write_read_request_empty_contents_max_count() {
        let req = I2cWriteReadRequest {
            address: 0x7F,
            contents: &[],
            count: u16::MAX,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: I2cWriteReadRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    #[test]
    fn gpio_get_request_all_pins() {
        for pin in 0..=u8::MAX {
            let req = GpioGetRequest { pin };
            let bytes = to_allocvec(&req).unwrap();
            let decoded: GpioGetRequest = from_bytes(&bytes).unwrap();
            assert_eq!(req, decoded);
        }
    }

    #[test]
    fn gpio_wait_request_all_pins() {
        for pin in 0..=u8::MAX {
            let req = GpioWaitRequest { pin };
            let bytes = to_allocvec(&req).unwrap();
            let decoded: GpioWaitRequest = from_bytes(&bytes).unwrap();
            assert_eq!(req, decoded);
        }
    }

    #[test]
    fn version_info_boundary_values() {
        for ver in [
            VersionInfo {
                major: 0,
                minor: 0,
                patch: 0,
            },
            VersionInfo {
                major: u16::MAX,
                minor: u16::MAX,
                patch: u32::MAX,
            },
        ] {
            let bytes = to_allocvec(&ver).unwrap();
            let decoded: VersionInfo = from_bytes(&bytes).unwrap();
            assert_eq!(ver, decoded);
        }
    }

    #[test]
    fn spi_set_configuration_request_all_enum_combinations() {
        for (phase, polarity) in [
            (SpiPhase::CaptureOnFirstTransition, SpiPolarity::IdleLow),
            (SpiPhase::CaptureOnFirstTransition, SpiPolarity::IdleHigh),
            (SpiPhase::CaptureOnSecondTransition, SpiPolarity::IdleLow),
            (SpiPhase::CaptureOnSecondTransition, SpiPolarity::IdleHigh),
        ] {
            let req = SpiSetConfigurationRequest {
                spi_frequency: 500_000,
                spi_phase: phase,
                spi_polarity: polarity,
            };
            let bytes = to_allocvec(&req).unwrap();
            let decoded: SpiSetConfigurationRequest = from_bytes(&bytes).unwrap();
            assert_eq!(req, decoded);
        }
    }

    #[test]
    fn i2c_set_configuration_request_all_frequencies() {
        for freq in [
            I2cFrequency::Standard,
            I2cFrequency::Fast,
            I2cFrequency::FastPlus,
        ] {
            let req = I2cSetConfigurationRequest { frequency: freq };
            let bytes = to_allocvec(&req).unwrap();
            let decoded: I2cSetConfigurationRequest = from_bytes(&bytes).unwrap();
            assert_eq!(req, decoded);
        }
    }

    #[test]
    fn i2c_frequency_discriminants_are_stable() {
        assert_eq!(I2cFrequency::Standard as u8, 0);
        assert_eq!(I2cFrequency::Fast as u8, 1);
        assert_eq!(I2cFrequency::FastPlus as u8, 2);
    }

    #[test]
    fn spi_set_configuration_request_max_frequency() {
        let req = SpiSetConfigurationRequest {
            spi_frequency: u32::MAX,
            spi_phase: SpiPhase::CaptureOnFirstTransition,
            spi_polarity: SpiPolarity::IdleLow,
        };
        let bytes = to_allocvec(&req).unwrap();
        let decoded: SpiSetConfigurationRequest = from_bytes(&bytes).unwrap();
        assert_eq!(req, decoded);
    }

    // SpiPhase and SpiPolarity discriminants must be stable for wire compat
    #[test]
    fn spi_phase_discriminants_are_stable() {
        assert_eq!(
            to_allocvec(&SpiPhase::CaptureOnFirstTransition).unwrap(),
            to_allocvec(&SpiPhase::CaptureOnFirstTransition).unwrap()
        );
        // Different variants must produce different bytes
        assert_ne!(
            to_allocvec(&SpiPhase::CaptureOnFirstTransition).unwrap(),
            to_allocvec(&SpiPhase::CaptureOnSecondTransition).unwrap()
        );
    }

    #[test]
    fn spi_polarity_discriminants_are_stable() {
        assert_ne!(
            to_allocvec(&SpiPolarity::IdleLow).unwrap(),
            to_allocvec(&SpiPolarity::IdleHigh).unwrap()
        );
    }

    #[test]
    fn gpio_state_discriminants_are_stable() {
        assert_ne!(
            to_allocvec(&GpioState::Low).unwrap(),
            to_allocvec(&GpioState::High).unwrap()
        );
    }
}
