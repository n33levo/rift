//! CLI Command Definitions
//!
//! Defines the command-line interface using clap.

pub mod connect;
pub mod info;
pub mod share;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// PortKey - Local-First P2P Tunneling Tool
///
/// Share local ports securely with peers over a QUIC-based P2P network.
/// No central server required.
#[derive(Parser, Debug)]
#[command(name = "pk")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Enable verbose debug logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable the TUI dashboard
    #[arg(long, global = true)]
    pub no_tui: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Share a local port with peers
    ///
    /// Starts listening for peer connections and forwards traffic
    /// to the specified local port.
    #[command(visible_alias = "s")]
    Share {
        /// The local port to share
        #[arg(value_name = "PORT")]
        port: u16,

        /// Path to .env file containing secrets to share
        #[arg(short, long, value_name = "FILE")]
        secrets: Option<PathBuf>,

        /// Automatically approve all incoming connections (insecure)
        #[arg(long)]
        auto_approve: bool,
    },

    /// Connect to a shared port
    ///
    /// Connects to a peer's shared port and creates a local proxy.
    #[command(visible_alias = "c")]
    Connect {
        /// The PortKey link (pk://<PEER_ID>) or peer ID to connect to
        #[arg(value_name = "LINK")]
        link: String,

        /// Local port to listen on (defaults to the remote port)
        #[arg(short, long, value_name = "PORT")]
        local_port: Option<u16>,

        /// Request secrets from the peer
        #[arg(long)]
        request_secrets: bool,

        /// Save received secrets to a file (requires --request-secrets)
        #[arg(long, value_name = "FILE", requires = "request_secrets")]
        save_secrets: Option<PathBuf>,
    },

    /// Show node information
    ///
    /// Displays the local peer ID and PortKey link.
    #[command(visible_alias = "i")]
    Info,
}
