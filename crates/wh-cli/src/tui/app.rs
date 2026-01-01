//! TUI Application State and Logic

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use wh_daemon::{DaemonCommand, DaemonEvent};
use ratatui::{prelude::*, Terminal};
use std::io;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

use super::ui;

/// Application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Share,
    Connect,
}

/// Connection entry for display
#[derive(Debug, Clone)]
pub struct ConnectionEntry {
    pub peer_id: String,
    pub connected_at: Instant,
    #[allow(dead_code)]
    pub bytes_sent: u64,
    #[allow(dead_code)]
    pub bytes_received: u64,
    #[allow(dead_code)]
    pub active: bool,
}

/// Application state
pub struct App {
    /// Current mode
    pub mode: AppMode,

    /// Local port (share mode) or remote port (connect mode)
    pub port: u16,

    /// Local port for connect mode
    #[allow(dead_code)]
    pub local_port: u16,

    /// Rift link
    pub link: String,

    /// Active connections
    pub connections: Vec<ConnectionEntry>,

    /// Total bytes sent
    pub bytes_sent: u64,

    /// Total bytes received
    pub bytes_received: u64,

    /// Log messages
    pub logs: Vec<String>,

    /// Should quit
    pub should_quit: bool,

    /// Status message
    pub status: String,

    /// Secrets count (if any)
    pub secrets_count: usize,

    /// Show help overlay
    pub show_help: bool,

    /// Traffic history for graph (bytes per second)
    pub traffic_history: Vec<u64>,

    /// Last stats update time
    last_stats_update: Instant,

    /// Pending connection approval request
    pub pending_approval: Option<String>,
}

impl App {
    /// Create a new app for share mode
    pub fn new_share(port: u16, link: String) -> Self {
        Self {
            mode: AppMode::Share,
            port,
            local_port: port,
            link,
            connections: Vec::new(),
            bytes_sent: 0,
            bytes_received: 0,
            logs: vec!["Rift started...".to_string()],
            should_quit: false,
            status: "Waiting for connections".to_string(),
            secrets_count: 0,
            show_help: false,
            traffic_history: vec![0; 60],
            last_stats_update: Instant::now(),
            pending_approval: None,
        }
    }

    /// Create a new app for connect mode
    pub fn new_connect(peer_link: String, remote_port: u16, local_port: u16) -> Self {
        Self {
            mode: AppMode::Connect,
            port: remote_port,
            local_port,
            link: peer_link,
            connections: Vec::new(),
            bytes_sent: 0,
            bytes_received: 0,
            logs: vec!["Rift started...".to_string()],
            should_quit: false,
            status: "Connecting...".to_string(),
            secrets_count: 0,
            show_help: false,
            traffic_history: vec![0; 60],
            last_stats_update: Instant::now(),
            pending_approval: None,
        }
    }

    /// Add a log message
    pub fn log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.logs.push(format!("[{}] {}", chrono_lite(), msg));

        // Keep only last 100 logs
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    /// Handle a daemon event
    pub fn handle_event(&mut self, event: DaemonEvent) {
        match event {
            DaemonEvent::Ready { peer_id, link } => {
                self.log(format!("Ready! Peer ID: {}", peer_id));
                if self.link.is_empty() {
                    self.link = link;
                }
            }
            DaemonEvent::Listening { address } => {
                self.log(format!("Listening on {}", address));
            }
            DaemonEvent::PeerConnected { peer_id } => {
                self.log(format!("Peer connected: {}", &peer_id[..16]));
                self.connections.push(ConnectionEntry {
                    peer_id: peer_id.clone(),
                    connected_at: Instant::now(),
                    bytes_sent: 0,
                    bytes_received: 0,
                    active: true,
                });
                self.status = format!("{} peer(s) connected", self.connections.len());
            }
            DaemonEvent::PeerDisconnected { peer_id } => {
                self.log(format!("Peer disconnected: {}", &peer_id[..16]));
                self.connections.retain(|c| c.peer_id != peer_id);
                if self.connections.is_empty() {
                    self.status = "Waiting for connections".to_string();
                } else {
                    self.status = format!("{} peer(s) connected", self.connections.len());
                }
            }
            DaemonEvent::TunnelEstablished { peer_id, port } => {
                self.log(format!("Tunnel established with {} on port {}", &peer_id[..16], port));
            }
            DaemonEvent::TunnelConnection { connection_id } => {
                self.log(format!("New tunnel connection #{}", connection_id));
            }
            DaemonEvent::IncomingConnectionRequest { peer_id } => {
                self.log(format!("Connection request from {}", &peer_id[..16]));
                self.pending_approval = Some(peer_id);
            }
            DaemonEvent::SecretsReceived { count } => {
                self.secrets_count = count;
                self.log(format!("Received {} secrets", count));
            }
            DaemonEvent::StatsUpdate {
                bytes_sent,
                bytes_received,
                active_connections: _,
            } => {
                // Calculate bytes/sec since last update
                let elapsed = self.last_stats_update.elapsed().as_secs_f64();
                if elapsed > 0.0 {
                    let bytes_delta = (bytes_sent + bytes_received).saturating_sub(self.bytes_sent + self.bytes_received);
                    let bytes_per_sec = (bytes_delta as f64 / elapsed) as u64;
                    
                    // Add to history and shift
                    self.traffic_history.remove(0);
                    self.traffic_history.push(bytes_per_sec);
                }
                
                self.bytes_sent = bytes_sent;
                self.bytes_received = bytes_received;
                self.last_stats_update = Instant::now();
            }
            DaemonEvent::Error { message } => {
                self.log(format!("Error: {}", message));
            }
            DaemonEvent::Shutdown => {
                self.should_quit = true;
            }
        }
    }

    /// Handle keyboard input
    pub fn handle_key(&mut self, key: KeyCode) -> Option<DaemonCommand> {
        // If there's a pending approval, handle y/n first
        if let Some(peer_id) = &self.pending_approval {
            match key {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    let peer_id = peer_id.clone();
                    self.pending_approval = None;
                    self.log("Connection approved");
                    return Some(DaemonCommand::ApproveConnection { peer_id });
                }
                KeyCode::Char('n') | KeyCode::Char('N') => {
                    let peer_id = peer_id.clone();
                    self.pending_approval = None;
                    self.log("Connection denied");
                    return Some(DaemonCommand::DenyConnection { peer_id });
                }
                _ => return None,
            }
        }

        // Normal key handling
        match key {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
                Some(DaemonCommand::Shutdown)
            }
            KeyCode::Char('h') => {
                self.show_help = !self.show_help;
                None
            }
            _ => None,
        }
    }
}

/// Simple time formatter (avoids chrono dependency)
fn chrono_lite() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs() % 86400;
    let hours = (secs / 3600) % 24;
    let mins = (secs / 60) % 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

/// Run the TUI for share mode
pub async fn run_share_tui(
    port: u16,
    link: String,
    event_rx: mpsc::Receiver<DaemonEvent>,
    command_tx: mpsc::Sender<DaemonCommand>,
) -> Result<()> {
    let app = App::new_share(port, link);
    run_tui(app, event_rx, command_tx).await
}

/// Run the TUI for connect mode
pub async fn run_connect_tui(
    peer_link: String,
    remote_port: u16,
    local_port: u16,
    event_rx: mpsc::Receiver<DaemonEvent>,
    command_tx: mpsc::Sender<DaemonCommand>,
) -> Result<()> {
    let app = App::new_connect(peer_link, remote_port, local_port);
    run_tui(app, event_rx, command_tx).await
}

/// Main TUI run loop
async fn run_tui(
    mut app: App,
    mut event_rx: mpsc::Receiver<DaemonEvent>,
    command_tx: mpsc::Sender<DaemonCommand>,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run loop
    let tick_rate = Duration::from_millis(100);
    let mut last_tick = Instant::now();

    loop {
        // Draw
        terminal.draw(|f| ui::draw(f, &app))?;

        // Handle events with timeout
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Check for keyboard events
        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let Some(cmd) = app.handle_key(key.code) {
                        let _ = command_tx.send(cmd).await;
                    }
                }
            }
        }

        // Check for daemon events (non-blocking)
        while let Ok(event) = event_rx.try_recv() {
            app.handle_event(event);
        }

        // Check quit
        if app.should_quit {
            break;
        }

        // Update tick
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
