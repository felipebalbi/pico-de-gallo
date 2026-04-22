use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::App;

/// Render the full TUI.
pub fn draw(f: &mut Frame, app: &App) {
    // Split into: status bar, waveform area, decoded events, help bar.
    let has_decoder = !app.decoded_events.is_empty() || app.num_channels > 0;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(if has_decoder {
            vec![
                Constraint::Length(3),  // status bar
                Constraint::Min(5),     // waveform
                Constraint::Length(10), // decoded events
                Constraint::Length(1),  // help bar
            ]
        } else {
            vec![
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(0),
                Constraint::Length(1),
            ]
        })
        .split(f.area());

    // --- Status bar ---
    let state_str = if app.running { "RUNNING" } else { "STOPPED" };
    let status_text = format!(
        " {} | Rate: {} Hz | Pins: {} | Samples: {} | Zoom: {}x | {}",
        state_str,
        app.sample_rate_hz,
        app.pins
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(","),
        app.sample_offset,
        app.zoom,
        app.status,
    );
    let status_bar = Paragraph::new(status_text).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Pico de Gallo Analyzer "),
    );
    f.render_widget(status_bar, chunks[0]);

    // --- Waveform area ---
    let waveform_block = Block::default().borders(Borders::ALL).title(" Waveform ");

    let inner = waveform_block.inner(chunks[1]);
    f.render_widget(waveform_block, chunks[1]);

    let cols = inner.width as usize;

    for ch in 0..app.num_channels.min(inner.height as usize) {
        let y = inner.y + ch as u16;
        let is_selected = ch == app.selected_channel;
        let style = if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Cyan)
        };

        // Build the waveform line for this channel.
        let mut spans = Vec::with_capacity(cols + 6);

        // Channel label
        let label = format!("P{:<2} ", app.pins.get(ch).copied().unwrap_or(ch as u8));
        spans.push(Span::styled(label, style));

        let start = app.scroll;
        for col in 0..cols.saturating_sub(4) {
            let sample_idx = start + col * app.zoom;
            if sample_idx < app.samples.len() {
                let sample = app.samples[sample_idx];
                let bit = (sample >> ch) & 1;
                let ch_char = if bit == 1 { "▀" } else { "▄" };
                spans.push(Span::styled(ch_char, style));
            } else {
                spans.push(Span::styled("·", Style::default().fg(Color::DarkGray)));
            }
        }

        let line = Line::from(spans);
        f.render_widget(
            Paragraph::new(line),
            ratatui::layout::Rect::new(inner.x, y, inner.width, 1),
        );
    }

    // --- Decoded events ---
    if chunks[2].height > 0 {
        let items: Vec<ListItem> = app
            .decoded_events
            .iter()
            .rev()
            .take(chunks[2].height as usize)
            .map(|e| ListItem::new(e.text.as_str()))
            .collect();
        let events_list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Decoded Events "),
        );
        f.render_widget(events_list, chunks[2]);
    }

    // --- Help bar ---
    let help = Paragraph::new(
        " q/Esc: Quit | Space: Start/Stop | ←/→: Scroll | ↑/↓: Channel | +/-: Zoom | c: Clear",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[3]);
}
