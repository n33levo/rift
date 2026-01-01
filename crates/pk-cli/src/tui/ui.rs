//! TUI UI Rendering - Cyberpunk Edition

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Sparkline},
};

use super::app::{App, AppMode};

/// Draw the main UI
pub fn draw(f: &mut Frame, app: &App) {
    if app.show_help {
        draw_help(f);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),   // Header - 20%
            Constraint::Min(15),     // Traffic Graph - 60%
            Constraint::Length(12),  // Logs - 20%
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_traffic_graph(f, app, chunks[1]);
    draw_logs(f, app, chunks[2]);
}

/// Draw the cyberpunk header with ASCII art
fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(70),  // Title
            Constraint::Percentage(30),  // Status
        ])
        .split(area);

    // ASCII art title
    let title_lines = vec![
        "██████╗  ██████╗ ██████╗ ████████╗██╗  ██╗███████╗██╗   ██╗",
        "██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝██║ ██╔╝██╔════╝╚██╗ ██╔╝",
        "██████╔╝██║   ██║██████╔╝   ██║   █████╔╝ █████╗   ╚████╔╝ ",
        "██╔═══╝ ██║   ██║██╔══██╗   ██║   ██╔═██╗ ██╔══╝    ╚██╔╝  ",
        "██║     ╚██████╔╝██║  ██║   ██║   ██║  ██╗███████╗   ██║   ",
        "╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝   ╚═╝   ",
    ];

    let title_text: Vec<Line> = title_lines
        .iter()
        .map(|line| Line::from(Span::styled(*line, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))))
        .collect();

    let title = Paragraph::new(title_text).alignment(Alignment::Left);

    f.render_widget(title, chunks[0]);

    // Status panel
    let status_color = if app.connections.is_empty() {
        Color::Yellow
    } else {
        Color::Green
    };

    let mode_text = match app.mode {
        AppMode::Share => "SHARE MODE",
        AppMode::Connect => "CONNECT MODE",
    };

    let status_text = vec![
        Line::from(vec![
            Span::styled("● ", Style::default().fg(status_color)),
            Span::styled("ONLINE", Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
        Line::from(Span::styled(mode_text, Style::default().fg(Color::Cyan))),
        Line::from(vec![
            Span::styled("PORT: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.port), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("PEERS: ", Style::default().fg(Color::DarkGray)),
            Span::styled(format!("{}", app.connections.len()), Style::default().fg(Color::Green)),
        ]),
        if app.secrets_count > 0 {
            Line::from(vec![
                Span::styled("SECRETS: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", app.secrets_count), Style::default().fg(Color::Magenta)),
            ])
        } else {
            Line::from("")
        },
    ];

    let status_panel = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title(" STATUS ")
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        )
        .alignment(Alignment::Left);

    f.render_widget(status_panel, chunks[1]);
}

/// Draw the traffic graph
fn draw_traffic_graph(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),      // Graph
            Constraint::Length(5),   // Stats
        ])
        .split(area);

    // Traffic sparkline
    let max_traffic = app.traffic_history.iter().max().copied().unwrap_or(1);
    let sparkline = Sparkline::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" TRAFFIC MONITOR [BYTES/SEC] ")
                .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        )
        .data(&app.traffic_history)
        .style(Style::default().fg(Color::Cyan))
        .max(max_traffic);

    f.render_widget(sparkline, chunks[0]);

    // Stats panel
    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(33),
            Constraint::Percentage(34),
        ])
        .split(chunks[1]);

    // Upload stats
    let upload_text = vec![
        Line::from(Span::styled("↑ UPLOAD", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(format_bytes(app.bytes_sent), Style::default().fg(Color::Green))),
    ];

    let upload_panel = Paragraph::new(upload_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)),
        )
        .alignment(Alignment::Center);

    f.render_widget(upload_panel, stats_chunks[0]);

    // Download stats
    let download_text = vec![
        Line::from(Span::styled("↓ DOWNLOAD", Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(format_bytes(app.bytes_received), Style::default().fg(Color::Blue))),
    ];

    let download_panel = Paragraph::new(download_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .alignment(Alignment::Center);

    f.render_widget(download_panel, stats_chunks[1]);

    // Connection info
    let total = app.bytes_sent + app.bytes_received;
    let info_text = vec![
        Line::from(Span::styled("⚡ TOTAL", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(format_bytes(total), Style::default().fg(Color::Yellow))),
    ];

    let info_panel = Paragraph::new(info_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Center);

    f.render_widget(info_panel, stats_chunks[2]);
}

/// Draw the logs panel
fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40),  // Peers
            Constraint::Percentage(60),  // Event log
        ])
        .split(area);

    // Peers list
    let peer_items: Vec<ListItem> = if app.connections.is_empty() {
        vec![ListItem::new(Line::from(vec![
            Span::styled("⌀ ", Style::default().fg(Color::DarkGray)),
            Span::styled("No active connections", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        ]))]
    } else {
        app.connections
            .iter()
            .map(|conn| {
                let peer_short = if conn.peer_id.len() > 12 {
                    format!("{}..{}", &conn.peer_id[..6], &conn.peer_id[conn.peer_id.len()-4..])
                } else {
                    conn.peer_id.clone()
                };
                let duration = conn.connected_at.elapsed();
                let duration_str = if duration.as_secs() > 60 {
                    format!("{}m", duration.as_secs() / 60)
                } else {
                    format!("{}s", duration.as_secs())
                };

                ListItem::new(Line::from(vec![
                    Span::styled("◉ ", Style::default().fg(Color::Green)),
                    Span::styled(peer_short, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(format!("[{}]", duration_str), Style::default().fg(Color::DarkGray)),
                ]))
            })
            .collect()
    };

    let peers_list = List::new(peer_items)
        .block(
            Block::default()
                .title(" PEERS ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan))
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        );

    f.render_widget(peers_list, chunks[0]);

    // Event logs
    let log_items: Vec<ListItem> = app
        .logs
        .iter()
        .rev()
        .take(8)
        .map(|log| {
            let (icon, style) = if log.contains("Error") || log.contains("Failed") {
                ("✗", Style::default().fg(Color::Red))
            } else if log.contains("connected") || log.contains("established") {
                ("✓", Style::default().fg(Color::Green))
            } else if log.contains("disconnected") {
                ("⚠", Style::default().fg(Color::Yellow))
            } else if log.contains("secrets") || log.contains("Secrets") {
                ("🔐", Style::default().fg(Color::Magenta))
            } else {
                ("•", Style::default().fg(Color::Gray))
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", icon), style),
                Span::styled(log.clone(), style),
            ]))
        })
        .collect();

    let logs_list = List::new(log_items).block(
        Block::default()
            .title(" EVENT LOG ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
    );

    f.render_widget(logs_list, chunks[1]);

    // Footer hint
    let footer_area = Rect {
        x: area.x,
        y: area.y + area.height - 1,
        width: area.width,
        height: 1,
    };

    let footer_text = Span::styled(
        " [q] QUIT | [h] HELP ",
        Style::default().fg(Color::DarkGray),
    );

    let footer = Paragraph::new(footer_text).alignment(Alignment::Center);
    f.render_widget(footer, footer_area);
}

/// Draw help overlay
fn draw_help(f: &mut Frame) {
    let area = centered_rect(60, 60, f.area());

    let help_text = vec![
        Line::from(""),
        Line::from(Span::styled("PORTKEY CONTROLS", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("  q  ", Style::default().fg(Color::Cyan)),
            Span::raw(" - Quit application"),
        ]),
        Line::from(vec![
            Span::styled("  h  ", Style::default().fg(Color::Cyan)),
            Span::raw(" - Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled(" ESC ", Style::default().fg(Color::Cyan)),
            Span::raw(" - Quit application"),
        ]),
        Line::from(""),
        Line::from(Span::styled("ABOUT", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("PortKey is a local-first P2P tunneling tool."),
        Line::from("Share ports securely over QUIC without a relay."),
        Line::from(""),
        Line::from(Span::styled("Press [h] to close", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))),
    ];

    let help_block = Paragraph::new(help_text)
        .block(
            Block::default()
                .title(" HELP ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan))
                .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left);

    f.render_widget(help_block, area);
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
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
