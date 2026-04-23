//! Handler module re-exports.
//!
//! Each peripheral has its own submodule. This `mod.rs` re-exports all
//! handler functions so `main.rs` can bring them into scope with a single
//! `use handlers::*;` for the `define_dispatch!` macro.

mod adc;
mod gpio;
mod i2c;
mod info;
mod onewire;
mod pwm;
mod spi;
mod uart;

pub(crate) use self::adc::*;
pub(crate) use self::gpio::*;
pub(crate) use self::i2c::*;
pub(crate) use self::info::*;
pub(crate) use self::onewire::*;
pub(crate) use self::pwm::*;
pub(crate) use self::spi::*;
pub(crate) use self::uart::*;
