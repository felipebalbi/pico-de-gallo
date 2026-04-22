use std::collections::VecDeque;

/// Maximum number of raw samples kept in the ring buffer.
const MAX_SAMPLES: usize = 64 * 1024;

/// Decoded protocol event with a display string.
#[derive(Debug, Clone)]
pub struct DecodedEvent {
    pub text: String,
}

/// Top-level application state shared between the UI and capture task.
pub struct App {
    /// Ring buffer of raw samples (keeps the last [`MAX_SAMPLES`]).
    pub samples: VecDeque<u8>,
    /// Total number of samples received since capture started.
    pub sample_offset: u64,
    /// Whether a capture is currently running.
    pub running: bool,
    /// Horizontal scroll position (in samples) for the waveform view.
    pub scroll: usize,
    /// Zoom level — number of samples per display column.
    pub zoom: usize,
    /// Currently highlighted channel index.
    pub selected_channel: usize,
    /// Human-readable decoded protocol events.
    pub decoded_events: Vec<DecodedEvent>,
    /// Status bar text.
    pub status: String,
    /// Number of capture channels (pins).
    pub num_channels: usize,
    /// Sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Pin numbers being captured.
    pub pins: Vec<u8>,
}

impl App {
    pub fn new(pins: Vec<u8>, sample_rate_hz: u32) -> Self {
        let num_channels = pins.len();
        Self {
            samples: VecDeque::with_capacity(MAX_SAMPLES),
            sample_offset: 0,
            running: false,
            scroll: 0,
            zoom: 1,
            selected_channel: 0,
            decoded_events: Vec::new(),
            status: "Ready — press Space to start capture".into(),
            num_channels,
            sample_rate_hz,
            pins,
        }
    }

    /// Append new raw samples from a capture chunk.
    pub fn push_samples(&mut self, data: &[u8]) {
        for &b in data {
            if self.samples.len() >= MAX_SAMPLES {
                self.samples.pop_front();
            }
            self.samples.push_back(b);
        }
        self.sample_offset += data.len() as u64;
    }

    /// Clear decoded events list.
    pub fn clear_decoded(&mut self) {
        self.decoded_events.clear();
    }

    /// Scroll left by one screen-width worth of samples.
    pub fn scroll_left(&mut self) {
        self.scroll = self.scroll.saturating_sub(self.zoom * 10);
    }

    /// Scroll right.
    pub fn scroll_right(&mut self) {
        self.scroll = self.scroll.saturating_add(self.zoom * 10);
    }

    /// Select the previous channel.
    pub fn channel_up(&mut self) {
        if self.selected_channel > 0 {
            self.selected_channel -= 1;
        }
    }

    /// Select the next channel.
    pub fn channel_down(&mut self) {
        if self.selected_channel + 1 < self.num_channels {
            self.selected_channel += 1;
        }
    }

    /// Zoom in (fewer samples per column).
    pub fn zoom_in(&mut self) {
        if self.zoom > 1 {
            self.zoom /= 2;
            if self.zoom < 1 {
                self.zoom = 1;
            }
        }
    }

    /// Zoom out (more samples per column).
    pub fn zoom_out(&mut self) {
        if self.zoom < 1024 {
            self.zoom *= 2;
        }
    }
}
