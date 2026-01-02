<div align="center">

# ⚡ Rift

### Make your teammate's localhost feel like yours

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with libp2p](https://img.shields.io/badge/built%20with-libp2p-blueviolet.svg)](https://libp2p.io)

<br>

<img src="assets/screenshot.png" alt="Rift TUI" width="700">

<br>

**Your teammate's service appears on *your* `localhost` — with their config, zero deployment.**

[Quick Start](#-quick-start) • [Use Cases](#-use-cases) • [Security](#-security) • [How It Works](#-how-it-works)

</div>

---

## What is Rift?

**Rift is the fastest way to make a teammate's local dev setup behave like it's on your machine** — ports on `localhost`, with explicit approval and optional encrypted config handoff.

```
Your machine                        Teammate's machine
localhost:3000  ◄───encrypted P2P───►  localhost:3000
localhost:5432  ◄───────────────────►  localhost:5432 (Postgres)
   + .env       ◄───────────────────►     + .env (encrypted, session-only)
```

No staging deploys. No public URLs. No Slack-ing `.env` screenshots. **One command.**

---

## 🚀 Quick Start

**Teammate A (has the service running):**
```bash
rift share 3000
# → Copies rift://12D3KooW... to clipboard
```

**Teammate B (wants to use it):**
```bash
rift connect rift://12D3KooW.../3000
# → localhost:3000 now routes to A's machine
```

That's it. B hits `http://localhost:3000` and it's A's service — encrypted, P2P.

### With environment context:

```bash
# A shares port + sanitized env config
rift share 3000 --secrets .env.rift

# B connects and gets the config to actually run against it
rift connect rift://... --request-secrets --save-secrets .env.local
```

Now B has the API keys, database URLs, and feature flags needed to run their frontend/tests against A's backend — without copy-paste or reconfiguration.

---

## 🎯 Use Cases

### Development & Pairing

| Scenario | What You Do |
|----------|-------------|
| **Pair-debug a backend** | Share port 3000, teammate's React app hits their `localhost:3000` |
| **"Works on my machine" issues** | Teammate accesses your *actual* running service |
| **Frontend ↔ Backend pairing** | Share your API, they develop against real data |
| **Code review with context** | Share your running branch, reviewer tests locally |

### Database & Infrastructure

| Scenario | What You Do |
|----------|-------------|
| **Share a database** | Share Postgres (5432), teammate uses `localhost:5432` |
| **Share Redis/cache** | Share 6379, teammate's app connects without config change |
| **Multi-service stack** | Share API + DB + Cache as multiple ports |

### Demo & Collaboration

| Scenario | What You Do |
|----------|-------------|
| **Demo internal tools** | Share admin dashboards to teammates without deploying |
| **OSS collaboration** | Maintainer shares failing service, contributor debugs locally |
| **Cross-team debugging** | Backend hosts "known good" state for frontend team |

### Share Any Local Web UI (Streamlit, Gradio, Jupyter, etc.)

Run **any** local web interface and share it with teammates — no cloud deployment needed:

```bash
# You're running Streamlit locally
streamlit run app.py  # → localhost:8501

# Share it
rift share 8501

# Teammate connects
rift connect rift://... 
# → Their localhost:8501 shows YOUR Streamlit app
```

Works with:
- **Streamlit** (`:8501`)
- **Gradio** (`:7860`)
- **Jupyter** (`:8888`)
- **FastAPI docs** (`:8000/docs`)
- **Any local web UI**

Teammate sees your app on their `localhost` — they can interact, test, debug. No Streamlit Cloud, no ngrok, no public exposure.

### GPU & Compute Resources

```bash
# GPU machine: run a model server
python -m vllm.entrypoints.openai.api_server --model mistral-7b
rift share 8000

# Laptop: use it as if local
rift connect rift://... 
curl http://localhost:8000/v1/completions ...
```

Your laptop talks to the GPU box as `localhost`. Run eval scripts, notebooks, anything — the heavy compute stays remote, but feels local.

---

## 🔐 Security

Rift is built for **trust between teammates**, with security defaults that prevent accidents.

### What makes it secure

| Layer | How it works |
|-------|--------------|
| **Explicit approval** | Every connection triggers a Y/N prompt — no silent access |
| **End-to-end encrypted** | QUIC transport + Noise Protocol (X25519, ChaCha20-Poly1305) |
| **Peer-to-peer** | Direct connection — no relay sees your plaintext traffic |
| **Localhost by default** | Client binds to `127.0.0.1` — your tunnel isn't exposed to your network |
| **Secrets are opt-in** | Must explicitly use `--secrets` to share, `--request-secrets` to receive |
| **Session-scoped** | Connections and secrets live only for the session |

### Secrets handling

```bash
# Host explicitly shares specific config
rift share 3000 --secrets .env.rift   # Only what's in .env.rift

# Peer explicitly requests and saves
rift connect rift://... --request-secrets --save-secrets .env.local
```

- **X25519 key exchange** → unique shared secret per session
- **AES-256-GCM encryption** → secrets encrypted before transit
- **No automatic persistence** → secrets saved only with `--save-secrets`
- **System keyring storage** → identity keys stored securely

**Best practice:** Create a `.env.rift` with only the variables needed (not your full `.env`).

---

## 🔧 How It Works

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

1. **Discovery** — mDNS on local networks, relay-assisted for remote
2. **Connection** — Direct QUIC stream, NAT hole-punching via DCUtR
3. **Approval** — Host sees Y/N prompt in TUI, decides to allow
4. **Bridge** — TCP traffic pumped through encrypted QUIC streams
5. **Secrets (optional)** — Encrypted handoff of environment config

Built on [libp2p](https://libp2p.io) (same foundation as IPFS) with Rust + Tokio.

---

## 📦 Installation

```bash
# From source
git clone https://github.com/n33levo/rift
cd rift
cargo build --release

# Binary at ./target/release/rift
```

Requires Rust 1.75+. On some systems: `sudo apt-get install libssl-dev pkg-config`

---

## 📖 Usage Reference

### Share a port

```bash
rift share 3000                      # Share port 3000
rift share 3000 --secrets .env.rift  # Share port + encrypted env vars
rift share 3000 --auto-approve       # Skip approval prompt (trusted networks)
rift share 3000 --no-tui             # Headless mode (servers, CI)
```

### Connect to a peer

```bash
rift connect rift://12D3KooW.../3000          # Connect, bind to same port
rift connect rift://... -l 8080                # Bind to different local port
rift connect rift://... --request-secrets      # Request shared config
rift connect rift://... --request-secrets --save-secrets .env.local
rift connect rift://... --public               # Bind to 0.0.0.0 (expose to network)
```

### Other commands

```bash
rift info    # Show your peer ID and connection info
```

---

## 💡 Recipes

### Share API + Database together

```bash
# Terminal 1: Share your API
rift share 3000 --secrets .env.rift

# Terminal 2: Share your Postgres  
rift share 5432
```

Teammate connects to both and has your full backend stack on their localhost.

### Quick UI demo without deployment

```bash
# You're developing a Gradio app
python app.py  # Running on :7860

# Share it instantly
rift share 7860

# Teammate views on their machine
rift connect rift://...  # localhost:7860 shows your app
```

### Cross-machine debugging

```bash
# Your service is failing on your machine
rift share 3000 --secrets .env.rift

# Teammate connects and runs tests against YOUR running service
rift connect rift://... --request-secrets --save-secrets .env
pytest tests/  # Tests hit your actual service
```

---

## 🚫 What Rift is NOT for

Rift intentionally does **less** than general-purpose tunnels:

- ❌ **Public URLs** — Use ngrok, Cloudflare Tunnel, or Vercel previews
- ❌ **Production deployments** — Use proper CI/CD
- ❌ **Permanent infrastructure** — Sessions are ephemeral
- ❌ **Untrusted parties** — Designed for teammates you trust

**Rift is for:** Pairing, debugging, demos, and development workflows between trusted collaborators.

---

## 🏗️ Project Structure

```
rift/
├── crates/
│   ├── wh-core/      # P2P networking, crypto, tunneling
│   │   ├── network/  # libp2p swarm, QUIC, mDNS, DCUtR
│   │   ├── secrets.rs    # EnvVault (X25519 + AES-GCM)
│   │   └── crypto.rs     # Key exchange, encryption
│   ├── wh-daemon/    # Session management, event loop
│   └── wh-cli/       # CLI commands + TUI (ratatui)
```

---

## 🤝 Contributing

PRs welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

Key principles:
- **Explicit consent** — No silent connections or secret sharing
- **Secure by default** — Encryption required, localhost binding default
- **Simple UX** — One command to share, one to connect

---

## License

MIT — see [LICENSE](LICENSE).

---

<div align="center">

**Made with ⚡ for developers who pair**

*"The fastest way to make two developers' local environments behave like one machine."*

</div>
