# Pico de Gallo Internal

[![crates.io](https://img.shields.io/crates/v/pico-de-gallo-internal.svg)](https://crates.io/crates/pico-de-gallo-internal)
[![docs.rs](https://docs.rs/pico-de-gallo-internal/badge.svg)](https://docs.rs/pico-de-gallo-internal)

Shared wire-protocol types for the [Pico de Gallo](https://github.com/OpenDevicePartnership/pico-de-gallo)
USB bridge. This crate defines all [postcard-rpc](https://docs.rs/postcard-rpc)
endpoints, request/response types, and constants used by both the
firmware and host-side library.

> **Note:** This is an internal crate. Application code should use
> [`pico-de-gallo-lib`](https://crates.io/crates/pico-de-gallo-lib) or
> [`pico-de-gallo-hal`](https://crates.io/crates/pico-de-gallo-hal) instead.

## Features

- `use-std` — Enables `Vec<u8>` response types for the host side.
  Without this feature (the firmware default), responses use borrowed
  `&[u8]` slices.

# License

Licensed under the terms of the [MIT license](http://opensource.org/licenses/MIT).

# Contribution

Any contribution intentionally submitted for inclusion in the work by
you shall be licensed under the terms of the same MIT license, without
any additional terms or conditions.
