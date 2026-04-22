use pico_de_gallo_lib::PicoDeGallo;
use tokio::sync::mpsc;

/// Messages sent from the capture task to the UI loop.
pub enum CaptureMsg {
    /// A chunk of raw sample bytes.
    Samples(Vec<u8>),
    /// Capture has stopped (with a status string).
    Stopped(String),
    /// An error occurred.
    Error(String),
}

/// Start the async capture loop.
///
/// Connects to the device, starts capture, and streams sample chunks over `tx`.
/// Runs until `stop_rx` receives a signal.
pub async fn capture_task(
    pins: Vec<u8>,
    sample_rate_hz: u32,
    tx: mpsc::Sender<CaptureMsg>,
    mut stop_rx: mpsc::Receiver<()>,
) {
    let device = PicoDeGallo::new();

    // Subscribe before starting so we don't miss early data.
    let mut sub = match device.subscribe_capture_data(64).await {
        Ok(s) => s,
        Err(e) => {
            let _ = tx
                .send(CaptureMsg::Error(format!("subscribe failed: {e}")))
                .await;
            return;
        }
    };

    if let Err(e) = device.capture_start(&pins, sample_rate_hz).await {
        let _ = tx
            .send(CaptureMsg::Error(format!("capture_start failed: {e}")))
            .await;
        return;
    }

    loop {
        tokio::select! {
            _ = stop_rx.recv() => {
                break;
            }
            msg = sub.recv() => {
                match msg {
                    Ok(data) => {
                        if tx.send(CaptureMsg::Samples(data.samples)).await.is_err() {
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(CaptureMsg::Error(format!("recv error: {e}"))).await;
                        break;
                    }
                }
            }
        }
    }

    match device.capture_stop().await {
        Ok(info) => {
            let msg = format!(
                "Stopped — {} samples, {} chunks",
                info.total_samples, info.chunks_sent
            );
            let _ = tx.send(CaptureMsg::Stopped(msg)).await;
        }
        Err(e) => {
            let _ = tx
                .send(CaptureMsg::Error(format!("capture_stop failed: {e}")))
                .await;
        }
    }
}
