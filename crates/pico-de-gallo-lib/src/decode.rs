//! Host-side protocol decoders for raw capture data.
//!
//! These decoders operate on the raw sample stream produced by the logic capture
//! subsystem. Each sample is one byte where bit N represents the state of capture
//! channel N.
//!
//! Both decoders are **stateful** and **iterator-based**: call [`I2cDecoder::feed`]
//! or [`UartDecoder::feed`] with each chunk of samples. The returned iterator
//! yields decoded frames/bytes. State is preserved across calls so a protocol
//! transaction can span multiple chunks.
//!
//! # Example
//!
//! ```rust,no_run
//! use pico_de_gallo_lib::decode::{I2cDecoder, I2cFrame};
//!
//! let mut decoder = I2cDecoder::new(0, 1, 500_000);
//! // ... receive samples from capture subscription ...
//! # let samples: &[u8] = &[];
//! for frame in decoder.feed(samples) {
//!     println!("{frame:?}");
//! }
//! ```

use std::fmt;

// ---------------------------------------------------------------------------
// I2C Decoder
// ---------------------------------------------------------------------------

/// A decoded I2C bus event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I2cFrame {
    /// START condition (SDA falls while SCL is high).
    Start {
        /// Sample index within the capture stream.
        sample_index: u64,
    },
    /// Repeated START condition.
    RepeatedStart {
        /// Sample index within the capture stream.
        sample_index: u64,
    },
    /// STOP condition (SDA rises while SCL is high).
    Stop {
        /// Sample index within the capture stream.
        sample_index: u64,
    },
    /// A complete byte (8 data bits + ACK/NACK).
    Data {
        /// The decoded byte value.
        value: u8,
        /// `true` if the receiver acknowledged (SDA low on 9th clock).
        ack: bool,
        /// Sample index of the first bit.
        sample_index: u64,
    },
}

impl fmt::Display for I2cFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start { sample_index } => write!(f, "[S @{sample_index}]"),
            Self::RepeatedStart { sample_index } => write!(f, "[Sr @{sample_index}]"),
            Self::Stop { sample_index } => write!(f, "[P @{sample_index}]"),
            Self::Data {
                value,
                ack,
                sample_index,
            } => {
                let ack_str = if *ack { "A" } else { "N" };
                write!(f, "[0x{value:02X} {ack_str} @{sample_index}]")
            }
        }
    }
}

/// Internal state machine for the I2C decoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum I2cState {
    /// Waiting for a START condition.
    Idle,
    /// Collecting data bits (0–7).
    CollectBits { byte_val: u8, bit_count: u8 },
    /// Waiting for the ACK/NACK bit (9th clock).
    WaitAck { byte_val: u8 },
}

/// Stateful I2C bus decoder.
///
/// Detects START, STOP, data bytes, and ACK/NACK by monitoring SDA transitions
/// relative to SCL state, and sampling SDA on SCL rising edges.
pub struct I2cDecoder {
    sda_bit: u8,
    scl_bit: u8,
    state: I2cState,
    prev_sda: bool,
    prev_scl: bool,
    /// Running sample counter (persists across `feed` calls).
    sample_offset: u64,
    /// Sample index where the current byte started.
    byte_start: u64,
    /// Sample rate in Hz (for documentation/future use).
    #[allow(dead_code)]
    sample_rate_hz: u32,
}

impl I2cDecoder {
    /// Create a new decoder.
    ///
    /// - `sda_channel`: which bit in each sample byte represents SDA
    /// - `scl_channel`: which bit in each sample byte represents SCL
    /// - `sample_rate_hz`: the capture sample rate (used for timing calculations)
    pub fn new(sda_channel: u8, scl_channel: u8, sample_rate_hz: u32) -> Self {
        Self {
            sda_bit: sda_channel,
            scl_bit: scl_channel,
            state: I2cState::Idle,
            prev_sda: true,
            prev_scl: true,
            sample_offset: 0,
            byte_start: 0,
            sample_rate_hz,
        }
    }

    /// Reset decoder state (e.g. after a bus error).
    pub fn reset(&mut self) {
        self.state = I2cState::Idle;
        self.prev_sda = true;
        self.prev_scl = true;
    }

    /// Feed a chunk of raw samples and return decoded frames.
    ///
    /// State is preserved across calls — a byte or condition can span chunks.
    pub fn feed<'a>(&'a mut self, samples: &'a [u8]) -> I2cIter<'a> {
        I2cIter {
            decoder: self,
            samples,
            pos: 0,
        }
    }

    /// Process a single sample and optionally return a decoded frame.
    fn process_sample(&mut self, sample: u8, index: u64) -> Option<I2cFrame> {
        let sda = (sample >> self.sda_bit) & 1 == 1;
        let scl = (sample >> self.scl_bit) & 1 == 1;

        let sda_fell = self.prev_sda && !sda;
        let sda_rose = !self.prev_sda && sda;
        let scl_rose = !self.prev_scl && scl;

        self.prev_sda = sda;
        self.prev_scl = scl;

        // START: SDA falls while SCL is high.
        if sda_fell && scl {
            let frame = match self.state {
                I2cState::Idle => I2cFrame::Start { sample_index: index },
                _ => I2cFrame::RepeatedStart { sample_index: index },
            };
            self.state = I2cState::CollectBits {
                byte_val: 0,
                bit_count: 0,
            };
            self.byte_start = index;
            return Some(frame);
        }

        // STOP: SDA rises while SCL is high.
        if sda_rose && scl {
            self.state = I2cState::Idle;
            return Some(I2cFrame::Stop { sample_index: index });
        }

        // Data sampling happens on SCL rising edge.
        if scl_rose {
            match self.state {
                I2cState::CollectBits { byte_val, bit_count } => {
                    // MSB first.
                    let new_val = (byte_val << 1) | (sda as u8);
                    let new_count = bit_count + 1;
                    if new_count == 8 {
                        self.state = I2cState::WaitAck { byte_val: new_val };
                    } else {
                        self.state = I2cState::CollectBits {
                            byte_val: new_val,
                            bit_count: new_count,
                        };
                    }
                }
                I2cState::WaitAck { byte_val } => {
                    let ack = !sda; // ACK = SDA low
                    self.state = I2cState::CollectBits {
                        byte_val: 0,
                        bit_count: 0,
                    };
                    let frame = I2cFrame::Data {
                        value: byte_val,
                        ack,
                        sample_index: self.byte_start,
                    };
                    self.byte_start = index + 1;
                    return Some(frame);
                }
                I2cState::Idle => {}
            }
        }

        None
    }
}

/// Iterator returned by [`I2cDecoder::feed`].
pub struct I2cIter<'a> {
    decoder: &'a mut I2cDecoder,
    samples: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for I2cIter<'a> {
    type Item = I2cFrame;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.samples.len() {
            let idx = self.decoder.sample_offset + self.pos as u64;
            let sample = self.samples[self.pos];
            self.pos += 1;
            if let Some(frame) = self.decoder.process_sample(sample, idx) {
                return Some(frame);
            }
        }
        // Update offset for next feed call.
        self.decoder.sample_offset += self.samples.len() as u64;
        None
    }
}

// ---------------------------------------------------------------------------
// UART Decoder
// ---------------------------------------------------------------------------

/// A decoded UART byte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UartByte {
    /// The decoded byte value.
    pub value: u8,
    /// Sample index of the start bit.
    pub sample_index: u64,
    /// `true` if the stop bit was not high (framing error).
    pub framing_error: bool,
}

impl fmt::Display for UartByte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.framing_error {
            write!(f, "[0x{:02X} FE @{}]", self.value, self.sample_index)
        } else {
            write!(f, "[0x{:02X} @{}]", self.value, self.sample_index)
        }
    }
}

/// Internal state of the UART decoder.
#[derive(Debug, Clone, Copy)]
enum UartState {
    /// Waiting for start bit (high → low transition).
    Idle,
    /// Inside a byte: counting samples to the midpoint of each bit.
    Receiving {
        /// Accumulated byte value (LSB first).
        byte_val: u8,
        /// Number of data bits collected so far.
        bit_count: u8,
        /// Samples remaining until we sample the next bit.
        samples_to_next: u32,
        /// Sample index of the start bit.
        start_index: u64,
    },
    /// Waiting to sample the stop bit.
    WaitStop {
        byte_val: u8,
        samples_to_next: u32,
        start_index: u64,
    },
}

/// Stateful UART decoder (8N1).
///
/// Detects start bits, samples 8 data bits at the configured baud rate,
/// and checks the stop bit. Only supports 8N1 framing.
pub struct UartDecoder {
    rx_bit: u8,
    samples_per_bit: u32,
    state: UartState,
    prev_rx: bool,
    sample_offset: u64,
}

impl UartDecoder {
    /// Create a new UART decoder.
    ///
    /// - `rx_channel`: which bit in each sample byte represents the RX line
    /// - `sample_rate_hz`: the capture sample rate
    /// - `baud_rate`: the expected UART baud rate
    pub fn new(rx_channel: u8, sample_rate_hz: u32, baud_rate: u32) -> Self {
        Self {
            rx_bit: rx_channel,
            samples_per_bit: sample_rate_hz / baud_rate,
            state: UartState::Idle,
            prev_rx: true,
            sample_offset: 0,
        }
    }

    /// Reset decoder state.
    pub fn reset(&mut self) {
        self.state = UartState::Idle;
        self.prev_rx = true;
    }

    /// Feed a chunk of raw samples and return decoded bytes.
    pub fn feed<'a>(&'a mut self, samples: &'a [u8]) -> UartIter<'a> {
        UartIter {
            decoder: self,
            samples,
            pos: 0,
        }
    }

    /// Process a single sample.
    fn process_sample(&mut self, sample: u8, index: u64) -> Option<UartByte> {
        let rx = (sample >> self.rx_bit) & 1 == 1;
        let rx_fell = self.prev_rx && !rx;
        self.prev_rx = rx;

        match self.state {
            UartState::Idle => {
                if rx_fell {
                    // Start bit detected. Skip to midpoint of first data bit:
                    // half a bit (to center of start bit) + one full bit.
                    let half_bit = self.samples_per_bit / 2;
                    self.state = UartState::Receiving {
                        byte_val: 0,
                        bit_count: 0,
                        samples_to_next: half_bit + self.samples_per_bit,
                        start_index: index,
                    };
                }
                None
            }
            UartState::Receiving {
                byte_val,
                bit_count,
                samples_to_next,
                start_index,
            } => {
                if samples_to_next > 1 {
                    self.state = UartState::Receiving {
                        byte_val,
                        bit_count,
                        samples_to_next: samples_to_next - 1,
                        start_index,
                    };
                    return None;
                }
                // Sample this data bit (LSB first).
                let new_val = byte_val | ((rx as u8) << bit_count);
                let new_count = bit_count + 1;
                if new_count == 8 {
                    // Wait for stop bit.
                    self.state = UartState::WaitStop {
                        byte_val: new_val,
                        samples_to_next: self.samples_per_bit,
                        start_index,
                    };
                } else {
                    self.state = UartState::Receiving {
                        byte_val: new_val,
                        bit_count: new_count,
                        samples_to_next: self.samples_per_bit,
                        start_index,
                    };
                }
                None
            }
            UartState::WaitStop {
                byte_val,
                samples_to_next,
                start_index,
            } => {
                if samples_to_next > 1 {
                    self.state = UartState::WaitStop {
                        byte_val,
                        samples_to_next: samples_to_next - 1,
                        start_index,
                    };
                    return None;
                }
                // Sample stop bit.
                let framing_error = !rx; // stop bit should be high
                self.state = UartState::Idle;
                Some(UartByte {
                    value: byte_val,
                    sample_index: start_index,
                    framing_error,
                })
            }
        }
    }
}

/// Iterator returned by [`UartDecoder::feed`].
pub struct UartIter<'a> {
    decoder: &'a mut UartDecoder,
    samples: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for UartIter<'a> {
    type Item = UartByte;

    fn next(&mut self) -> Option<Self::Item> {
        while self.pos < self.samples.len() {
            let idx = self.decoder.sample_offset + self.pos as u64;
            let sample = self.samples[self.pos];
            self.pos += 1;
            if let Some(byte) = self.decoder.process_sample(sample, idx) {
                return Some(byte);
            }
        }
        self.decoder.sample_offset += self.samples.len() as u64;
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: generate I2C waveform samples.
    // Each sample byte: bit 0 = SDA, bit 1 = SCL.
    fn i2c_idle() -> u8 {
        0b11 // SDA=1, SCL=1
    }
    fn i2c_sda_low_scl_high() -> u8 {
        0b10 // SDA=0, SCL=1
    }
    fn i2c_sda_high_scl_low() -> u8 {
        0b01 // SDA=1, SCL=0
    }
    fn i2c_both_low() -> u8 {
        0b00 // SDA=0, SCL=0
    }

    /// Build a synthetic I2C waveform that sends a START, one byte, ACK, STOP.
    fn make_i2c_byte_waveform(byte_val: u8, ack: bool) -> Vec<u8> {
        let mut samples = Vec::new();

        // Idle
        samples.extend_from_slice(&[i2c_idle(); 4]);

        // START: SDA falls while SCL high
        samples.push(i2c_sda_low_scl_high()); // SDA↓, SCL=1

        // 8 data bits, MSB first
        for bit_idx in (0..8).rev() {
            let sda_val = (byte_val >> bit_idx) & 1;
            // SCL low: set SDA
            if sda_val == 1 {
                samples.push(i2c_sda_high_scl_low()); // SDA=1, SCL=0
            } else {
                samples.push(i2c_both_low()); // SDA=0, SCL=0
            }
            // SCL high: data sampled
            if sda_val == 1 {
                samples.push(i2c_idle()); // SDA=1, SCL=1
            } else {
                samples.push(i2c_sda_low_scl_high()); // SDA=0, SCL=1
            }
            // SCL back low
            if sda_val == 1 {
                samples.push(i2c_sda_high_scl_low());
            } else {
                samples.push(i2c_both_low());
            }
        }

        // ACK/NACK bit
        if ack {
            samples.push(i2c_both_low()); // SDA=0, SCL=0
            samples.push(i2c_sda_low_scl_high()); // SDA=0, SCL=1 (ACK)
            samples.push(i2c_both_low()); // SCL low
        } else {
            samples.push(i2c_sda_high_scl_low()); // SDA=1, SCL=0
            samples.push(i2c_idle()); // SDA=1, SCL=1 (NACK)
            samples.push(i2c_sda_high_scl_low()); // SCL low
        }

        // STOP: SDA rises while SCL high
        samples.push(i2c_both_low()); // SDA=0, SCL=0
        samples.push(i2c_sda_low_scl_high()); // SDA=0, SCL=1
        samples.push(i2c_idle()); // SDA=1, SCL=1 (STOP)

        samples
    }

    #[test]
    fn i2c_decodes_start_stop() {
        let mut decoder = I2cDecoder::new(0, 1, 500_000);

        // START
        let mut samples = vec![i2c_idle(); 4];
        samples.push(i2c_sda_low_scl_high()); // START
        // idle a bit, then STOP
        samples.push(i2c_both_low()); // SDA=0, SCL=0
        samples.push(i2c_sda_low_scl_high()); // SDA=0, SCL=1
        samples.push(i2c_idle()); // STOP

        let frames: Vec<_> = decoder.feed(&samples).collect();
        assert!(frames.len() >= 2);
        assert!(matches!(frames[0], I2cFrame::Start { .. }));
        assert!(matches!(frames.last().unwrap(), I2cFrame::Stop { .. }));
    }

    #[test]
    fn i2c_decodes_byte_with_ack() {
        let mut decoder = I2cDecoder::new(0, 1, 500_000);
        let samples = make_i2c_byte_waveform(0xA5, true);
        let frames: Vec<_> = decoder.feed(&samples).collect();

        // Should have: Start, Data(0xA5, ack=true), Stop
        assert_eq!(frames.len(), 3, "frames: {frames:?}");
        assert!(matches!(frames[0], I2cFrame::Start { .. }));
        assert_eq!(
            frames[1],
            I2cFrame::Data {
                value: 0xA5,
                ack: true,
                sample_index: frames[1].sample_index(),
            }
        );
        assert!(matches!(frames[2], I2cFrame::Stop { .. }));
    }

    #[test]
    fn i2c_decodes_byte_with_nack() {
        let mut decoder = I2cDecoder::new(0, 1, 500_000);
        let samples = make_i2c_byte_waveform(0x3C, false);
        let frames: Vec<_> = decoder.feed(&samples).collect();

        assert_eq!(frames.len(), 3, "frames: {frames:?}");
        match &frames[1] {
            I2cFrame::Data { value, ack, .. } => {
                assert_eq!(*value, 0x3C);
                assert!(!ack);
            }
            other => panic!("expected Data, got {other:?}"),
        }
    }

    #[test]
    fn i2c_state_spans_chunks() {
        let mut decoder = I2cDecoder::new(0, 1, 500_000);
        let samples = make_i2c_byte_waveform(0x55, true);
        let mid = samples.len() / 2;

        let mut all_frames = Vec::new();
        all_frames.extend(decoder.feed(&samples[..mid]));
        all_frames.extend(decoder.feed(&samples[mid..]));

        // Should still decode Start + Data + Stop
        assert_eq!(all_frames.len(), 3, "frames: {all_frames:?}");
        match &all_frames[1] {
            I2cFrame::Data { value, ack, .. } => {
                assert_eq!(*value, 0x55);
                assert!(*ack);
            }
            other => panic!("expected Data, got {other:?}"),
        }
    }

    #[test]
    fn i2c_frame_display() {
        assert_eq!(I2cFrame::Start { sample_index: 10 }.to_string(), "[S @10]");
        assert_eq!(I2cFrame::Stop { sample_index: 99 }.to_string(), "[P @99]");
        assert_eq!(
            I2cFrame::Data {
                value: 0xAB,
                ack: true,
                sample_index: 50,
            }
            .to_string(),
            "[0xAB A @50]"
        );
        assert_eq!(
            I2cFrame::Data {
                value: 0x01,
                ack: false,
                sample_index: 70,
            }
            .to_string(),
            "[0x01 N @70]"
        );
    }

    // Helper: generate UART 8N1 waveform.
    // Channel bit = 0 (bit 0 of each sample).
    // Idle = high (bit 0 = 1).
    fn make_uart_waveform(byte_val: u8, samples_per_bit: u32) -> Vec<u8> {
        let spb = samples_per_bit as usize;
        let mut samples = Vec::new();

        // Idle
        samples.extend(std::iter::repeat_n(0x01u8, spb * 2));

        // Start bit (low)
        samples.extend(std::iter::repeat_n(0x00u8, spb));

        // 8 data bits, LSB first
        for bit in 0..8 {
            let bit_val = (byte_val >> bit) & 1;
            samples.extend(std::iter::repeat_n(bit_val, spb));
        }

        // Stop bit (high)
        samples.extend(std::iter::repeat_n(0x01u8, spb));

        // Idle
        samples.extend(std::iter::repeat_n(0x01u8, spb * 2));

        samples
    }

    #[test]
    fn uart_decodes_byte() {
        // 500kHz sample rate, 9600 baud → ~52 samples/bit
        let mut decoder = UartDecoder::new(0, 500_000, 9600);
        let samples = make_uart_waveform(0x55, 52);
        let decoded: Vec<_> = decoder.feed(&samples).collect();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].value, 0x55);
        assert!(!decoded[0].framing_error);
    }

    #[test]
    fn uart_decodes_multiple_bytes() {
        let spb = 52u32; // samples per bit
        let mut decoder = UartDecoder::new(0, 500_000, 9600);

        let mut samples = make_uart_waveform(0x48, spb); // 'H'
        samples.extend(make_uart_waveform(0x69, spb)); // 'i'

        let decoded: Vec<_> = decoder.feed(&samples).collect();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].value, 0x48);
        assert_eq!(decoded[1].value, 0x69);
    }

    #[test]
    fn uart_framing_error() {
        let spb = 52usize;
        let mut samples = Vec::new();

        // Idle
        samples.extend(std::iter::repeat_n(0x01u8, spb * 2));
        // Start bit
        samples.extend(std::iter::repeat_n(0x00u8, spb));
        // 8 data bits (all zeros)
        samples.extend(std::iter::repeat_n(0x00u8, spb * 8));
        // Stop bit should be high, but we send low → framing error
        samples.extend(std::iter::repeat_n(0x00u8, spb));
        // Return to idle
        samples.extend(std::iter::repeat_n(0x01u8, spb * 2));

        let mut decoder = UartDecoder::new(0, 500_000, 9600);
        let decoded: Vec<_> = decoder.feed(&samples).collect();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].value, 0x00);
        assert!(decoded[0].framing_error);
    }

    #[test]
    fn uart_state_spans_chunks() {
        let spb = 52u32;
        let mut decoder = UartDecoder::new(0, 500_000, 9600);
        let samples = make_uart_waveform(0xAA, spb);
        let mid = samples.len() / 2;

        let mut decoded = Vec::new();
        decoded.extend(decoder.feed(&samples[..mid]));
        decoded.extend(decoder.feed(&samples[mid..]));

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].value, 0xAA);
        assert!(!decoded[0].framing_error);
    }

    #[test]
    fn uart_byte_display() {
        let b = UartByte {
            value: 0x48,
            sample_index: 100,
            framing_error: false,
        };
        assert_eq!(b.to_string(), "[0x48 @100]");

        let b = UartByte {
            value: 0xFF,
            sample_index: 200,
            framing_error: true,
        };
        assert_eq!(b.to_string(), "[0xFF FE @200]");
    }

    // Helper trait for tests to extract sample_index from any frame variant.
    impl I2cFrame {
        fn sample_index(&self) -> u64 {
            match self {
                Self::Start { sample_index } => *sample_index,
                Self::RepeatedStart { sample_index } => *sample_index,
                Self::Stop { sample_index } => *sample_index,
                Self::Data { sample_index, .. } => *sample_index,
            }
        }
    }
}
