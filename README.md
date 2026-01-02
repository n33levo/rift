<div align="center">

# ⚡ Rift

**AirDrop for Localhost. Pairing-grade P2P tunneling for ports + config.**  
Bring a teammate's service to *your* `localhost` — **not** a public URL.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with libp2p](https://img.shields.io/badge/built%20with-libp2p-blueviolet.svg)](https://libp2p.io)

<br>

<img src="assets/screenshot.png" alt="Rift TUI" width="700">

</div>

---

**Highlights**
- ⚡ **Low-latency QUIC tunnel** (libp2p)  
- 🔒 **End-to-end encrypted by default** (Noise + ChaCha20-Poly1305)  
- ✅ **Explicit host approval** (no "magic links")  
- 🧩 **Local-to-local port mapping** (`their localhost` → `your localhost`)  
- 🧪 **Optional EnvVault** to sync *selected* env/config  
- 🧭 **P2P-first discovery** + NAT traversal (where possible)
- 🖥️ **TUI + ergonomic CLI**

---

## The Problem

Pairing on real systems is always the same mess:

- "Push to staging so I can see it" (**slow**)
- "Share your screen" (**pain**)
- "Paste your `.env` in Slack" (**please don't**)

You don't want a public URL — you want your teammate's service to behave like it's running locally.

---

## The Solution

Rift **maps their `localhost` into yours**.

```text
Your machine                       Teammate's machine
localhost:3000  ◄── E2E-encrypted ─►  localhost:3000
```

No public endpoints. No deploy. No ceremony.  
Just "connect" → **debug immediately**.

---

## 🔒 Security & Trust (Read This First)

- ✅ **Explicit approval:** every inbound session requires a host **Y/N** prompt.
- 🔐 **Encrypted by default:** tunnel traffic is wrapped in **Noise** over **QUIC** (ChaCha20-Poly1305).
- 🕳️ **P2P-first:** data flows peer-to-peer when possible; if relays are used for connectivity, payloads remain **end-to-end encrypted**.
- 🧾 **Secrets are opt-in:** config sharing is explicit (you choose what to send). Rift never silently uploads secrets anywhere.

> **Threat model:** built for pairing/debugging with teammates you trust — not for anonymous public access.

[→ Full security details](docs/SECURITY.md)

---

## Install

```bash
brew install n33levo/rift/rift
```

<details>
<summary>Other installation methods</summary>

**From source:**
```bash
git clone https://github.com/n33levo/rift
cd rift && cargo build --release
```

</details>

---

## Quickstart

```bash
# On the machine running the service:
rift share 3000

# On your machine:
rift connect rift://12D3KooW.../3000
```

Now their service is reachable on **your** `http://localhost:3000`.

### Optional: share just enough config (EnvVault)

```bash
# Host
rift share 3000 --secrets .env.rift

# Client
rift connect rift://.../3000 --request-secrets
```

Stop doing "set these 47 env vars" archaeology.

[→ Full CLI reference](docs/USAGE.md)

---

## Use Cases

- 🔧 **Backend ↔ Frontend pairing** (no staging deploy)
- 🗄️ **Databases on localhost** (Postgres/Redis/Mongo)
- 🧠 **GPU inference as localhost** (model server on a GPU box → your laptop)
- 📈 **Dashboards & demos** (Streamlit/Gradio) without hosting
- 🔭 **Observability ports** (TensorBoard, metrics UIs)
- 🧰 **Internal tooling** behind NAT

👉 More: [docs/USE_CASES.md](docs/USE_CASES.md)

---

## How It Works

```text
Discovery → Encrypted QUIC session → Host approval → TCP bridge (local ↔ local)
```

Built on **libp2p**:

* mDNS discovery on LAN
* QUIC transport + Noise encryption
* NAT traversal (DCUtR) where possible

[→ Architecture deep dive](docs/HOW_IT_WORKS.md)

---

## Not For

Rift is **not** a public hosting platform.

If you need a public URL for customers/PMs, use **ngrok / Cloudflare Tunnel / Vercel previews**.  
Use Rift when you want **pairing-grade, local-to-local debugging** with teammates.

---

## Docs

- [Usage Guide](docs/USAGE.md) — CLI reference, examples
- [Use Cases](docs/USE_CASES.md) — Real-world scenarios  
- [Security](docs/SECURITY.md) — Threat model, best practices
- [Architecture](docs/HOW_IT_WORKS.md) — How it's built

---

## Contributing

[CONTRIBUTING.md](CONTRIBUTING.md)

---

<div align="center">

**MIT License**

Built with ⚡ by developers tired of deploying to staging

[⭐ Give it a star if you've ever said "works on my machine" ⭐](https://github.com/n33levo/rift)

</div>
