//! TUI Module for Rift
//!
//! Provides a ratatui-based terminal UI for monitoring connections and traffic.

mod app;
mod ui;

pub use app::{run_connect_tui, run_share_tui};
