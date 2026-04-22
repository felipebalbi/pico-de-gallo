mod app;
mod capture;
mod decode;
mod ui;

use std::io;
use std::time::Duration;

use app::{App, DecodedEvent};
use capture::{capture_task, CaptureMsg};
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use decode::{Decoder, DecoderKind};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(
    name = "gallo-analyzer",
    about = "Terminal logic analyzer for Pico de Gallo"
)]
struct Cli {
    /// Comma-separated channel indices to capture (0–3, e.g. 0,1)
    #[arg(long, value_delimiter = ',')]
    pins: Vec<u8>,

    /// Sample rate in Hz
    #[arg(long, default_value_t = 100_000)]
    rate: u32,

    /// Protocol decoder to use (i2c or uart)
    #[arg(long)]
    decode: Option<String>,

    /// UART baud rate (used with --decode uart)
    #[arg(long, default_value_t = 115200)]
    uart_baud: u32,

    /// I2C SDA channel index (used with --decode i2c)
    #[arg(long, default_value_t = 0)]
    sda_channel: u8,

    /// I2C SCL channel index (used with --decode i2c)
    #[arg(long, default_value_t = 1)]
    scl_channel: u8,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.pins.is_empty() {
        eprintln!("Error: --pins is required (e.g. --pins 0,1)");
        std::process::exit(1);
    }

    let decoder_kind = cli.decode.as_deref().map(|proto| match proto {
        "i2c" => DecoderKind::I2c {
            sda_channel: cli.sda_channel,
            scl_channel: cli.scl_channel,
        },
        "uart" => DecoderKind::Uart {
            rx_channel: 0,
            baud_rate: cli.uart_baud,
        },
        other => {
            eprintln!("Unknown protocol: {other}. Supported: i2c, uart");
            std::process::exit(1);
        }
    });

    let rt = tokio::runtime::Runtime::new().expect("failed to create tokio runtime");

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, &rt, cli.pins, cli.rate, decoder_kind);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {e}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    rt: &tokio::runtime::Runtime,
    pins: Vec<u8>,
    sample_rate_hz: u32,
    decoder_kind: Option<DecoderKind>,
) -> io::Result<()> {
    let mut app = App::new(pins.clone(), sample_rate_hz);
    let mut decoder = decoder_kind
        .as_ref()
        .map(|kind| Decoder::new(kind, sample_rate_hz));

    let (data_tx, mut data_rx) = mpsc::channel::<CaptureMsg>(256);
    let mut stop_tx: Option<mpsc::Sender<()>> = None;
    let mut capture_handle: Option<tokio::task::JoinHandle<()>> = None;

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Drain pending capture messages (non-blocking).
        while let Ok(msg) = data_rx.try_recv() {
            match msg {
                CaptureMsg::Samples(samples) => {
                    if let Some(ref mut dec) = decoder {
                        let events = dec.feed(&samples);
                        for text in events {
                            app.decoded_events.push(DecodedEvent { text });
                        }
                    }
                    app.push_samples(&samples);
                }
                CaptureMsg::Stopped(status) => {
                    app.running = false;
                    app.status = status;
                }
                CaptureMsg::Error(err) => {
                    app.running = false;
                    app.status = format!("Error: {err}");
                }
            }
        }

        // Poll for keyboard events with a short timeout.
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if let Some(tx) = stop_tx.take() {
                            let _ = tx.try_send(());
                        }
                        break;
                    }
                    KeyCode::Char(' ') => {
                        if app.running {
                            if let Some(tx) = stop_tx.take() {
                                let _ = tx.try_send(());
                            }
                            app.status = "Stopping…".into();
                        } else {
                            // Fresh stop channel for this capture session.
                            let (new_stop_tx, new_stop_rx) = mpsc::channel::<()>(1);
                            let handle = rt.spawn(capture_task(
                                pins.clone(),
                                sample_rate_hz,
                                data_tx.clone(),
                                new_stop_rx,
                            ));
                            stop_tx = Some(new_stop_tx);
                            capture_handle = Some(handle);
                            app.running = true;
                            app.status = "Capturing…".into();
                        }
                    }
                    KeyCode::Left => app.scroll_left(),
                    KeyCode::Right => app.scroll_right(),
                    KeyCode::Up => app.channel_up(),
                    KeyCode::Down => app.channel_down(),
                    KeyCode::Char('+') | KeyCode::Char('=') => app.zoom_in(),
                    KeyCode::Char('-') => app.zoom_out(),
                    KeyCode::Char('c') => app.clear_decoded(),
                    _ => {}
                }
            }
        }
    }

    if let Some(handle) = capture_handle {
        let _ = rt.block_on(handle);
    }

    Ok(())
}
