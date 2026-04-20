# Pico de Gallo FFI

[![crates.io](https://img.shields.io/crates/v/pico-de-gallo-ffi.svg)](https://crates.io/crates/pico-de-gallo-ffi)
[![docs.rs](https://docs.rs/pico-de-gallo-ffi/badge.svg)](https://docs.rs/pico-de-gallo-ffi)

C-compatible FFI bindings for the [Pico de Gallo](https://github.com/OpenDevicePartnership/pico-de-gallo)
USB bridge. Wraps the Rust host library in an opaque-pointer API with
integer status codes, suitable for use from C, C++, Python, and other
languages.

## Usage

```c
#include "pico_de_gallo.h"

const PicoDeGallo *gallo = gallo_init();
uint32_t id = 42;
Status status = gallo_ping(gallo, &id);
gallo_free(gallo);
```

A C header (`pico_de_gallo.h`) is generated automatically by
[cbindgen](https://docs.rs/cbindgen) at build time.

# License

Licensed under the terms of the [MIT license](http://opensource.org/licenses/MIT).

# Contribution

Any contribution intentionally submitted for inclusion in the work by
you shall be licensed under the terms of the same MIT license, without
any additional terms or conditions.
