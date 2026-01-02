# How It Works

## Architecture Overview

```
┌────────────────┐        QUIC/P2P         ┌────────────────┐
│   Your Machine │◄───────────────────────►│  Teammate      │
│                │     (encrypted)         │                │
│  localhost:3000│                         │  localhost:3000│
│  localhost:5432│                         │  localhost:5432│
└────────────────┘                         └────────────────┘
        ▲                                          │
        │         TCP ←→ QUIC stream bridge        │
        └──────────────────────────────────────────┘
```

---

## Connection Flow

### 1. Discovery

**Local networks (same WiFi):**
- Uses **mDNS** (multicast DNS) for automatic peer discovery
- No configuration needed
- Typically discovers peers in 100-200ms

**Remote networks:**
- Uses **libp2p relay servers** for initial connection
- Relay helps bootstrap the connection
- No relay sees plaintext (encrypted end-to-end)

---

### 2. Connection Establishment

**NAT hole-punching (DCUtR):**
1. Initial connection through relay
2. **DCUtR** (Direct Connection Upgrade through Relay) attempts direct connection
3. If successful, traffic flows peer-to-peer
4. Relay is no longer involved

**QUIC transport:**
- UDP-based, multiplexed streams
- Built-in encryption and congestion control
- Multiple logical streams over one connection

---

### 3. Approval Gate

```
┌─────────────────────────────────────┐
│  Incoming connection request        │
│  from: 12D3KooW... (peer ID)        │
│                                     │
│  [Y] Approve   [N] Deny             │
└─────────────────────────────────────┘
```

- Every connection pauses until host approves
- Host sees peer ID in TUI
- Can use `--auto-approve` to skip (trusted networks only)

---

### 4. TCP ↔ QUIC Bridge

```
Client app → localhost:3000 (TCP)
              ↓
          Rift client
              ↓
          QUIC stream (encrypted P2P)
              ↓
          Rift host
              ↓
          localhost:3000 (TCP) → Host service
```

**Zero-copy streaming:**
- TCP bytes read from client app
- Pumped through QUIC stream
- Written to host's local service
- Bi-directional, full-duplex

---

### 5. Secrets Exchange (Optional)

```
┌──────────────┐                      ┌──────────────┐
│    Client    │                      │     Host     │
└──────┬───────┘                      └──────┬───────┘
       │                                     │
       │  1. Send public key (X25519)        │
       ├────────────────────────────────────►│
       │                                     │
       │  2. Encrypt secrets with ECDH       │
       │     (AES-256-GCM)                   │
       │◄────────────────────────────────────┤
       │                                     │
       │  3. Decrypt & save (optional)       │
       │                                     │
```

**Encryption:**
1. Client generates ephemeral X25519 keypair
2. Client sends public key to host
3. Host performs ECDH to derive shared secret
4. Host encrypts `.env.rift` with AES-256-GCM
5. Client decrypts with their private key
6. Optionally saves to file with `--save-secrets`

---

## Technology Stack

### libp2p
- **What:** Modular P2P networking framework
- **Used by:** IPFS, Filecoin, Polkadot, Ethereum 2.0
- **Provides:** Transport abstraction, peer discovery, NAT traversal

### QUIC
- **What:** Modern transport protocol (UDP-based)
- **Benefits:** Multiplexing, low latency, built-in encryption
- **Used by:** HTTP/3, many P2P systems

### Noise Protocol
- **What:** Cryptographic handshake framework
- **Used by:** WireGuard, WhatsApp, Lightning Network
- **Provides:** Forward secrecy, mutual authentication

### Rust + Tokio
- **Memory safety** without garbage collection
- **Async I/O** for high concurrency
- **Zero-cost abstractions** for performance

---

## Project Structure

```
rift/
├── crates/
│   ├── wh-core/           # Core P2P networking and tunneling
│   │   ├── network/       # libp2p swarm, QUIC, mDNS, DCUtR
│   │   ├── secrets.rs     # EnvVault (X25519 + AES-GCM)
│   │   ├── crypto.rs      # Key exchange, encryption
│   │   └── proxy/         # TCP ↔ QUIC stream bridging
│   │
│   ├── wh-daemon/         # Session management, event loop
│   │   ├── server.rs      # Main daemon orchestration
│   │   └── session.rs     # Share/Connect session handlers
│   │
│   └── wh-cli/            # CLI commands + TUI
│       ├── cli/           # Command implementations
│       └── tui/           # Terminal UI (ratatui)
│
└── target/release/rift    # Compiled binary
```

---

## Performance Characteristics

### Latency
- **Local network:** ~1-5ms added latency (mDNS discovery)
- **Remote network:** ~10-50ms added latency (relay bootstrap + DCUtR)
- **After direct connection:** Near-native TCP latency

### Throughput
- **QUIC overhead:** ~5-10% vs raw TCP
- **Encryption overhead:** Negligible (hardware-accelerated)
- **Bottleneck:** Usually your network, not Rift

### Resource Usage
- **Memory:** ~10-20MB per tunnel
- **CPU:** Minimal (async I/O, zero-copy where possible)
- **Network:** Direct P2P (no relay data overhead after DCUtR)
