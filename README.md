# PortKey 🔑

**Local-First P2P Tunneling Tool** - An ngrok alternative using QUIC and libp2p

> ⚠️ **Status**: Core tunnel implementation complete, currently in testing phase

## What is PortKey?

PortKey is a peer-to-peer tunneling tool that lets you expose local servers to remote clients without any central infrastructure. Unlike traditional tunneling tools (ngrok, localtunnel), PortKey:

- ✅ **Runs completely P2P** - no relay servers needed (with NAT hole punching)
- ✅ **Uses QUIC** - modern, fast, reliable transport over UDP
- ✅ **Local-first** - works on local networks via mDNS
- ✅ **Open source** - MIT licensed

## Architecture

```
┌─────────────┐                  ┌─────────────┐                  ┌─────────────┐
│   Client    │                  │   Peer B    │                  │   Peer A    │
│  (curl)     │─TCP:8080─────────▶  Connector  │─QUIC Stream──────▶   Sharer    │
└─────────────┘                  └─────────────┘                  └──────┬──────┘
                                                                          │ TCP:3000
                                                                   ┌──────▼──────┐
                                                                   │   Target    │
                                                                   │   Server    │
                                                                   └─────────────┘
```

**Data Flow**:
1. Client connects to Peer B's local TCP port (8080)
2. Peer B opens a QUIC stream to Peer A
3. Peer A bridges the stream to the target server (localhost:3000)
4. Bytes flow bidirectionally: `TCP ↔ QUIC ↔ TCP`

## Implementation Status

### ✅ Completed
- [x] P2P networking with libp2p (QUIC, noise, yamux)
- [x] Peer discovery (mDNS for local, relay for remote)
- [x] NAT hole punching (dcutr)
- [x] Identity management
- [x] Stream-based tunneling with `libp2p-stream`
- [x] Bidirectional data forwarding
- [x] Event loop and command handling
- [x] Basic CLI commands (share, connect, info)
- [x] Smoke tests

### 🚧 In Progress
- [ ] End-to-end integration testing
- [ ] Manual verification with real services

### 📋 TODO
- [ ] Secrets management (EnvVault)
- [ ] TUI implementation
- [ ] Error recovery and reconnection
- [ ] Performance optimization
- [ ] Documentation
- [ ] Real-world testing

## Quick Start

### Build
```bash
cargo build --release
```

### Run Tests
```bash
# Quick smoke tests
cargo test -p pk-core --test smoke_test

# Full integration test (ignored by default)
cargo test -p pk-core --test tunnel_integration -- --ignored --nocapture
```

### Manual Test

**Terminal 1** - Start a local server:
```bash
python3 -m http.server 8000
```

**Terminal 2** - Share it:
```bash
./target/release/pk share 8000 --no-tui
# Copy the pk:// link
```

**Terminal 3** - Connect from another peer:
```bash
./target/release/pk connect <pk-link> --no-tui
```

**Terminal 4** - Test it:
```bash
curl http://localhost:8000
```

## How It Works

### Peer A (Sharer/Host)
1. Listens for incoming QUIC streams on `/portkey/tunnel/1.0.0` protocol
2. When a stream arrives, connects to `localhost:PORT`
3. Pumps bytes bidirectionally: `QUIC stream ↔ TCP socket`

### Peer B (Connector/Client)
1. Connects to Peer A via libp2p
2. Starts a local `TcpListener` on specified port
3. When a TCP connection arrives:
   - Opens a QUIC stream to Peer A
   - Pumps bytes bidirectionally: `TCP socket ↔ QUIC stream`

### The Magic
- Uses `tokio::io::copy` for efficient byte copying
- `tokio-util::compat` bridges futures and tokio async traits
- libp2p handles connection management, encryption, and NAT traversal
- QUIC provides reliable, multiplexed streams over UDP

## Tech Stack

- **Language**: Rust 2024 edition
- **Async Runtime**: Tokio
- **P2P Networking**: libp2p 0.54
  - Transport: QUIC (UDP)
  - Security: Noise protocol
  - Multiplexing: Yamux
  - Discovery: mDNS, Identify
  - NAT Traversal: Relay + DCUtR
- **Stream Protocol**: libp2p-stream 0.2.0-alpha
- **CLI**: clap 4.5
- **Logging**: tracing

## Project Structure

```
portkey/
├── crates/
│   ├── pk-core/         # Core P2P and tunneling logic
│   │   ├── src/
│   │   │   ├── network/     # libp2p swarm, behaviour, streams
│   │   │   ├── config.rs    # Configuration
│   │   │   ├── crypto.rs    # X25519/AES-GCM
│   │   │   ├── secrets.rs   # EnvVault (secrets management)
│   │   │   └── error.rs     # Error types
│   │   └── tests/           # Integration tests
│   ├── pk-daemon/       # Background daemon (session management)
│   └── pk-cli/          # CLI interface
├── TESTING.md          # Testing guide
└── Cargo.toml          # Workspace config
```

## License

MIT

## Contributing

This is a fresh implementation. Testing and feedback welcome!
