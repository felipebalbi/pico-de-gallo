# Pico de Gallo Lib

[![crates.io](https://img.shields.io/crates/v/pico-de-gallo-lib.svg)](https://crates.io/crates/pico-de-gallo-lib)
[![docs.rs](https://docs.rs/pico-de-gallo-lib/badge.svg)](https://docs.rs/pico-de-gallo-lib)

Async host-side library for communicating with a [Pico de Gallo](https://github.com/OpenDevicePartnership/pico-de-gallo)
USB bridge device. Provides typed methods for I²C, SPI, GPIO, and
device configuration over USB.

Requires the [tokio](https://tokio.rs) async runtime.

## Quick Start

```rust
use pico_de_gallo_lib::PicoDeGallo;

#[tokio::main]
async fn main() {
    let gallo = PicoDeGallo::new();
    let version = gallo.version().await.unwrap();
    println!("Firmware v{}.{}.{}", version.major, version.minor, version.patch);
}
```

See the [examples](https://github.com/OpenDevicePartnership/pico-de-gallo/tree/main/crates/pico-de-gallo-lib/examples)
for more usage patterns.

# License

Licensed under the terms of the [MIT license](http://opensource.org/licenses/MIT).

# Contribution

Any contribution intentionally submitted for inclusion in the work by
you shall be licensed under the terms of the same MIT license, without
any additional terms or conditions.
