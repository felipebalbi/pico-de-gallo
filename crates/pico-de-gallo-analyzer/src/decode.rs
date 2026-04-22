use pico_de_gallo_lib::decode::{I2cDecoder, UartDecoder};

/// Which protocol decoder to use.
#[derive(Debug, Clone)]
pub enum DecoderKind {
    I2c { sda_channel: u8, scl_channel: u8 },
    Uart { rx_channel: u8, baud_rate: u32 },
}

/// Wrapper around the library decoders that produces human-readable strings.
pub struct Decoder {
    inner: DecoderInner,
}

enum DecoderInner {
    I2c(I2cDecoder),
    Uart(UartDecoder),
}

impl Decoder {
    /// Create a decoder from the configuration.
    pub fn new(kind: &DecoderKind, sample_rate_hz: u32) -> Self {
        let inner = match kind {
            DecoderKind::I2c {
                sda_channel,
                scl_channel,
            } => DecoderInner::I2c(I2cDecoder::new(*sda_channel, *scl_channel, sample_rate_hz)),
            DecoderKind::Uart {
                rx_channel,
                baud_rate,
            } => DecoderInner::Uart(UartDecoder::new(*rx_channel, sample_rate_hz, *baud_rate)),
        };
        Self { inner }
    }

    /// Feed raw samples and return decoded event strings.
    pub fn feed(&mut self, samples: &[u8]) -> Vec<String> {
        match &mut self.inner {
            DecoderInner::I2c(dec) => dec.feed(samples).map(|f| f.to_string()).collect(),
            DecoderInner::Uart(dec) => dec.feed(samples).map(|b| b.to_string()).collect(),
        }
    }
}
