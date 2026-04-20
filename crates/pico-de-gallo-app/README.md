# Gallo

[![crates.io](https://img.shields.io/crates/v/gallo.svg)](https://crates.io/crates/gallo)
[![docs.rs](https://docs.rs/gallo/badge.svg)](https://docs.rs/gallo)

Command-line interface for the [Pico de Gallo](https://github.com/OpenDevicePartnership/pico-de-gallo)
USB bridge. Provides direct access to I²C, SPI, and GPIO peripherals
in batch mode.

# Usage

```console
$ gallo list                             # List connected devices
$ gallo version                          # Query firmware version
$ gallo i2c scan                         # Scan I2C bus
$ gallo i2c read -a 0x48 -c 2            # Read 2 bytes from address 0x48
$ gallo i2c write -a 0x50 -b 0xDE 0xAD   # Write bytes to address 0x50
$ gallo spi transfer -b 0x01 0x02        # Full-duplex SPI transfer
$ gallo spi read -c 10 -f ascii          # Read 10 bytes, ASCII output
```

## Output Formats

Read data can be displayed in three formats via `-f` / `--format`:

- `hex` (default): hexadecimal byte dump
- `binary`: raw bytes to stdout
- `ascii`: printable characters, non-printable shown as `.`

# License

Licensed under the terms of the [MIT license](http://opensource.org/licenses/MIT).

# Contribution

Any contribution intentionally submitted for inclusion in the work by
you shall be licensed under the terms of the same MIT license, without
any additional terms or conditions.
