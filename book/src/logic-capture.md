# Logic Capture

Pico de Gallo provides logic capture support through the RP2350's PIO
(Programmable I/O) state machine hardware. The capture channels share the
4 GPIO pins (**GPIO 8–11**, channels 0–3), sampled by **PIO1/SM0** at a
configurable sample rate. Raw sample data is streamed to the host as
`CaptureData` chunks via a topic subscription.

When a capture session is active, the captured GPIO pins are unavailable
for normal GPIO operations. They are returned automatically when capture
stops.

## Operations

| Operation | Description |
|-----------|-------------|
| **Start** | Begins capturing on the selected channels at the given sample rate |
| **Stop** | Stops an active capture session and returns pins to GPIO |
| **Raw stream** | Subscribes to raw `CaptureData` chunks streamed from the device |

## Capturing I2C Traffic Example

A common use case is sniffing I2C traffic. Connect the I2C SDA and SCL
lines to two of the capture channels (e.g., channel 0 for SCL, channel 1
for SDA — GPIO 8 and GPIO 9) and start a capture session.

### CLI

```bash
# 1. Start capturing on channels 0–1 at 1 MHz
gallo capture start --pins 0,1 --rate 1000000

# 2. Stream raw capture data to a file
gallo capture raw > capture.bin

# 3. Stop the capture (or press Ctrl+C during raw streaming)
gallo capture stop
```

### Rust Library

```rust,no_run
use pico_de_gallo_lib::PicoDeGallo;

async fn capture_i2c_traffic(gallo: &PicoDeGallo) {
    // Start capturing on channels 0–1 at 1 MHz
    let info = gallo.capture_start(&[0, 1], 1_000_000).await.unwrap();
    println!("Capture started: {:?}", info);

    // Subscribe to raw capture data (buffer depth of 16 chunks)
    let mut sub = gallo.subscribe_capture_data(16).await.unwrap();

    // Read a few chunks
    for _ in 0..10 {
        let chunk = sub.recv().await.unwrap();
        println!("Got {} bytes", chunk.data.len());
    }

    // Stop the capture
    let stop_info = gallo.capture_stop().await.unwrap();
    println!("Capture stopped: {:?}", stop_info);
}
```

### C (FFI)

```c
#include "pico_de_gallo.h"
#include <stdio.h>

void capture_i2c(PicoDeGallo *gallo) {
    GalloCaptureStartInfo start_info;
    uint8_t pins[] = {0, 1};

    int rc = gallo_capture_start(gallo, pins, 2, 1000000, &start_info);
    if (rc < 0) {
        fprintf(stderr, "Failed to start capture: %d\n", rc);
        return;
    }

    printf("Capture started\n");

    /* ... read data via topic subscription ... */

    GalloCaptureStopInfo stop_info;
    gallo_capture_stop(gallo, &stop_info);
    printf("Capture stopped\n");
}
```

### Protocol Decoding

The host library includes a `decode` module (`pico_de_gallo_lib::decode`)
with built-in protocol decoders for analyzing captured data:

- **`I2cDecoder`** — detects START/STOP conditions, extracts data bytes,
  and identifies ACK/NAK bits from raw I2C bus traffic.
- **`UartDecoder`** — decodes 8N1 UART frames from a single-channel
  capture stream.

## Limitations

- **USB Full Speed bandwidth** caps throughput at approximately **700 KB/s**.
  At higher sample rates or channel counts, the firmware may drop chunks.
- **Maximum sample rate** depends on the number of active channels. Fewer
  channels allow higher per-channel rates within the USB bandwidth budget.
- The PIO program uses a single instruction (`in pins, 8` with autopush),
  so all 8 GPIO lines (8–15) are always sampled together — only bits 0–3
  (channels 0–3) carry valid capture data; bits 4–7 (PWM pins) are ignored.
- Capture pins cannot be used for GPIO operations while a capture session
  is active.

## Pin Mapping

| Channel | GPIO | Notes |
|---------|------|-------|
| 0       | 8    | Shared with GPIO 0 |
| 1       | 9    | Shared with GPIO 1 |
| 2       | 10   | Shared with GPIO 2 |
| 3       | 11   | Shared with GPIO 3 |
