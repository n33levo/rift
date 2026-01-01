//! TCP Proxy Module for Rift
//!
//! Handles bidirectional proxying between local TCP connections
//! and QUIC streams over the P2P network.

pub mod listener;
pub mod tunnel;

pub use listener::ProxyListener;
pub use tunnel::TunnelProxy;
