<div align="center">

# ⚡ Rift

### Pairing-grade localhost tunneling

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with libp2p](https://img.shields.io/badge/built%20with-libp2p-blueviolet.svg)](https://libp2p.io)

<br>

<img src="assets/screenshot.png" alt="Rift TUI" width="700">

<br>

**Your service appears on your teammate's `localhost` — not a public URL.**

[Quick Start](#-quick-start) • [How It Works](#-how-it-works) • [Why Rift](#-why-rift)

</div>

---

## What is Rift?

Rift is a **peer-to-peer tunnel** that makes a teammate's local service appear on *your* localhost. No public URLs, no relay servers seeing your traffic, no copy-pasting `.env` files in Slack.

```
Peer A: localhost:3000  ←──encrypted P2P──→  Peer B: localhost:3000
```

**Not for public hosting** — use ngrok or Cloudflare Tunnel for that. Rift is for **pairing and debugging** with teammates you trust.

---

## 🚀 Quick Start

**Peer A (sharing):**
```bash
rift share 3000
# Copies rift://12D3KooW... to clipboard
```

**Peer B (connecting):**
```bash
rift connect rift://12D3KooW.../3000
# Access at http://localhost:3000
```

That's it. Peer B can now hit `localhost:3000` and traffic flows encrypted to Peer A's machine.

---

## ✨ Why Rift?

| Problem | Rift Solution |
|---------|---------------|
| "Deploy to staging just to debug" | Direct P2P tunnel — no deployment |
| "Send me your .env" in Slack | `--secrets .env` encrypts and sends config |
| "Works on my machine" | Teammate uses *your* actual service |
| Public tunnel exposes sensitive APIs | Binds to `127.0.0.1` by default |
| Magic links bypass consent | Explicit Y/N approval for every connection |

### The Workflow Rift Enables

> "Bind your local service into my localhost, with explicit approval, over an encrypted P2P channel, and optionally give me just enough config to run it — for this debugging session."

Most teams do this today by pushing to staging, screen-sharing, or Slack-ing `.env` screenshots. Rift collapses that into **one command**.

---

## 🔒 Security

- **No magic links** — Every connection requires explicit host approval (Y/N prompt)
- **Encrypted end-to-end** — Noise Protocol (ChaCha20-Poly1305) over QUIC
- **Peer-to-peer** — No central server sees your traffic
- **Secrets are opt-in** — Host uses `--secrets`, peer uses `--request-secrets`
- **Localhost by default** — Client binds to `127.0.0.1` unless you use `--public`

---

## 🔧 How It Works

```
┌─────────────┐     QUIC/P2P      ┌─────────────┐
│  Peer B     │◄──────────────────►│  Peer A     │
│  localhost  │   (encrypted)     │  localhost  │
│  :3000      │                   │  :3000      │
└─────────────┘                   └─────────────┘
```

1. **Discovery** — mDNS on local networks, IPFS relays for remote peers
2. **Connection** — Direct QUIC stream with NAT hole-punching (DCUtR)
3. **Approval** — Host sees popup, presses Y to allow
4. **Bridge** — TCP ↔ QUIC byte pumping, zero-copy

Built on [libp2p](https://libp2p.io) (same stack as IPFS/Filecoin) with Rust + Tokio.

---

## 📦 Installation

```bash
# From source
git clone https://github.com/n33levo/rift
cd rift
cargo build --release
# Binary at ./target/release/rift
```

---

## 📖 Usage

### Share a port
```bash
rift share 3000                    # Basic share
rift share 3000 --secrets .env     # Include environment variables
rift share 3000 --auto-approve     # Skip approval (trusted networks)
rift share 3000 --no-tui           # Headless mode
```

### Connect to a peer
```bash
rift connect rift://...            # Connect and bind to same port
rift connect rift://... -l 8080    # Bind to different local port
rift connect rift://... --request-secrets --save-secrets .env
```

### Check your peer ID
```bash
rift info
```

---

## 🎯 Use Cases

- **Backend ↔ Frontend pairing** — Share your API, teammate's React app hits their localhost
- **GPU server sharing** — Run vLLM on a GPU box, use it from your laptop as `localhost:8000`
- **"Works on my machine" debugging** — Let teammate access your actual running service
- **Database sharing** — Share Postgres/Redis for cross-team debugging
- **Demo internal tools** — Share admin dashboards without deploying

---

## 🏗️ Project Structure

```
rift/
├── crates/
│   ├── wh-core/      # P2P networking, encryption, tunneling
│   ├── wh-daemon/    # Background daemon, session management
│   └── wh-cli/       # CLI + TUI (ratatui)
```

---

## 🤝 Contributing

PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

**Made with ⚡ for developers who pair**

</div>
