//! Rift Daemon
//!
//! Background service that manages P2P connections, tunnels, and secrets sharing.

pub mod server;
pub mod session;

pub use server::{DaemonCommand, DaemonEvent, DaemonServer};
pub use session::{ConnectSession, ShareSession};
