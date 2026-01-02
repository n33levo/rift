<div align="center">

# ⚡ Rift

**Your teammate's localhost on yours — one command**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with libp2p](https://img.shields.io/badge/built%20with-libp2p-blueviolet.svg)](https://libp2p.io)

<br>

<img src="assets/screenshot.png" alt="Rift TUI" width="700">

<br>

</div>

---

## What

**P2P localhost tunneling for pairing & debugging.**

```
Your machine                    Teammate's machine
localhost:3000  ◄─ encrypted ─►  localhost:3000
```

No staging. No public URLs. No `.env` screenshots in Slack.

---

## Quick Start

```bash
# Teammate A (sharing)
rift share 3000

# Teammate B (connecting)
rift connect rift://12D3KooW.../3000
```

Now `localhost:3000` on B's machine hits A's service. Encrypted, P2P.

**With env context:**
```bash
rift share 3000 --secrets .env.rift
rift connect rift://... --request-secrets --save-secrets .env
```

---

## Install

```bash
git clone https://github.com/n33levo/rift
cd rift
cargo build --release
```

---

## Why Use This

- ✅ Pair-debug without deploying to staging
- ✅ Share databases, APIs, any local service
- ✅ Share Streamlit/Gradio/Jupyter UIs instantly
- ✅ GPU server access as `localhost:8000`
- ✅ Encrypted (QUIC + Noise), P2P
- ✅ Explicit approval for every connection

[→ Full use cases](docs/USE_CASES.md)

---

## How It Works

**mDNS discovery** → **QUIC tunnel** → **Y/N approval** → **TCP bridge** → **Done**

Built on [libp2p](https://libp2p.io) (IPFS stack). Encrypted end-to-end. Direct P2P (relays for NAT only).

[→ Architecture details](docs/HOW_IT_WORKS.md)

---

## Security

- End-to-end encrypted (X25519, ChaCha20-Poly1305)
- Explicit Y/N approval for connections
- Localhost binding by default
- Secrets encrypted with AES-256-GCM

[→ Security details](docs/SECURITY.md)

---

## Docs

- [Usage Guide](docs/USAGE.md) — CLI reference, examples, recipes
- [Use Cases](docs/USE_CASES.md) — When to use Rift
- [Security](docs/SECURITY.md) — Threat model, best practices
- [How It Works](docs/HOW_IT_WORKS.md) — Architecture deep dive

---

## Not For

❌ Public URLs (use ngrok)  
❌ Production (use real infra)  
❌ Untrusted parties (made for teammates)

---

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md)

---

## License

MIT

---

<div align="center">

**Made for developers who pair**

</div>
