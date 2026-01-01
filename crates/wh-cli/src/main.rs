//! PortKey CLI
//!
//! Command-line interface for the PortKey P2P tunneling tool.

mod cli;
mod tui;

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("debug")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();

    // Execute command
    match cli.command {
        Commands::Share { port, secrets, auto_approve } => {
            cli::share::run(port, secrets, auto_approve, cli.no_tui).await?;
        }
        Commands::Connect { link, local_port, request_secrets, save_secrets } => {
            cli::connect::run(link, local_port, request_secrets, save_secrets, cli.no_tui).await?;
        }
        Commands::Info => {
            cli::info::run().await?;
        }
    }

    Ok(())
}
