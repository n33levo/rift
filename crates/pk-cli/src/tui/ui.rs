//! TUI UI Rendering

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::app::{App, AppMode};

/// Draw the main UI
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Length(7),  // Info panel
            Constraint::Min(10),    // Connections / Logs
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_info_panel(f, app, chunks[1]);
    draw_main_content(f, app, chunks[2]);
    draw_footer(f, app, chunks[3]);
}

/// Draw the header
fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        AppMode::Share => format!(" 🔑 PortKey Share - Port {} ", app.port),
        AppMode::Connect => format!(" 🔗 PortKey Connect - {} ", app.link),
    };

    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(header, area);
}

/// Draw the info panel
fn draw_info_panel(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left panel - Connection info
    let info_text = match app.mode {
        AppMode::Share => {
            vec![
                Line::from(vec![
                    Span::raw("Port: "),
                    Span::styled(
                        format!("{}", app.port),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Link: "),
                    Span::styled(
                        &app.link,
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        &app.status,
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                if app.secrets_count > 0 {
                    Line::from(vec![
                        Span::raw("Secrets: "),
                        Span::styled(
                            format!("{} loaded", app.secrets_count),
                            Style::default().fg(Color::Magenta),
                        ),
                    ])
                } else {
                    Line::from("")
                },
            ]
        }
        AppMode::Connect => {
            vec![
                Line::from(vec![
                    Span::raw("Remote: "),
                    Span::styled(
                        format!("{}:{}", &app.link, app.port),
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Local: "),
                    Span::styled(
                        format!("localhost:{}", app.local_port),
                        Style::default().fg(Color::Yellow),
                    ),
                ]),
                Line::from(vec![
                    Span::raw("Status: "),
                    Span::styled(
                        &app.status,
                        Style::default().fg(Color::Cyan),
                    ),
                ]),
                if app.secrets_count > 0 {
                    Line::from(vec![
                        Span::raw("Secrets: "),
                        Span::styled(
                            format!("{} received", app.secrets_count),
                            Style::default().fg(Color::Magenta),
                        ),
                    ])
                } else {
                    Line::from("")
                },
            ]
        }
    };

    let info_paragraph = Paragraph::new(info_text)
        .block(
            Block::default()
                .title(" Connection ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

    f.render_widget(info_paragraph, chunks[0]);

    // Right panel - Statistics
    let stats_text = vec![
        Line::from(vec![
            Span::raw("↑ Sent: "),
            Span::styled(
                format_bytes(app.bytes_sent),
                Style::default().fg(Color::Green),
            ),
        ]),
        Line::from(vec![
            Span::raw("↓ Received: "),
            Span::styled(
                format_bytes(app.bytes_received),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::raw("Connections: "),
            Span::styled(
                format!("{}", app.connections.len()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
    ];

    let stats_paragraph = Paragraph::new(stats_text)
        .block(
            Block::default()
                .title(" Statistics ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        );

    f.render_widget(stats_paragraph, chunks[1]);
}

/// Draw the main content area
fn draw_main_content(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(area);

    // Left - Connected peers
    let items: Vec<ListItem> = app
        .connections
        .iter()
        .map(|conn| {
            let status = if conn.active { "●" } else { "○" };
            let peer_short = if conn.peer_id.len() > 16 {
                &conn.peer_id[..16]
            } else {
                &conn.peer_id
            };
            let duration = conn.connected_at.elapsed();
            let duration_str = format!("{}s", duration.as_secs());

            ListItem::new(Line::from(vec![
                Span::styled(
                    status,
                    Style::default().fg(if conn.active {
                        Color::Green
                    } else {
                        Color::Red
                    }),
                ),
                Span::raw(" "),
                Span::styled(peer_short, Style::default().fg(Color::White)),
                Span::raw(" "),
                Span::styled(duration_str, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let peers_list = List::new(items)
        .block(
            Block::default()
                .title(" Connected Peers ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(peers_list, chunks[0]);

    // Right - Logs
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(20)
        .map(|log| {
            let style = if log.contains("Error") {
                Style::default().fg(Color::Red)
            } else if log.contains("connected") {
                Style::default().fg(Color::Green)
            } else if log.contains("disconnected") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            ListItem::new(Span::styled(log.clone(), style))
        })
        .collect();

    let logs_list = List::new(log_items).block(
        Block::default()
            .title(" Logs ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::White)),
    );

    f.render_widget(logs_list, chunks[1]);
}

/// Draw the footer
fn draw_footer(f: &mut Frame, _app: &App, area: Rect) {
    let footer = Paragraph::new(" [q] Quit | [c] Copy Link | [r] Refresh ")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(footer, area);
}

/// Format bytes for display
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
