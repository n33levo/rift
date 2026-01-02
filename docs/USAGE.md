# Usage Guide

## Installation

### From Source
```bash
git clone https://github.com/n33levo/rift
cd rift
cargo build --release
# Binary at ./target/release/rift
```

Requires Rust 1.75+. On some systems: `sudo apt-get install libssl-dev pkg-config`

---

## Commands

### Share a port

```bash
rift share <PORT> [OPTIONS]
```

**Examples:**
```bash
rift share 3000                      # Share port 3000
rift share 3000 --secrets .env.rift  # Share port + encrypted env vars
rift share 3000 --auto-approve       # Skip approval prompt (trusted networks)
rift share 3000 --no-tui             # Headless mode (servers, CI)
```

**Options:**
- `--secrets <FILE>` — Path to .env file containing secrets to share
- `--auto-approve` — Automatically approve all incoming connections (insecure)
- `--no-tui` — Disable the TUI dashboard

---

### Connect to a peer

```bash
rift connect <LINK> [OPTIONS]
```

**Examples:**
```bash
rift connect rift://12D3KooW.../3000          # Connect, bind to same port
rift connect rift://... -l 8080                # Bind to different local port
rift connect rift://... --request-secrets      # Request shared config
rift connect rift://... --request-secrets --save-secrets .env.local
rift connect rift://... --public               # Bind to 0.0.0.0 (expose to network)
```

**Options:**
- `-l, --local-port <PORT>` — Local port to listen on (defaults to remote port)
- `--public` — Bind to 0.0.0.0 instead of 127.0.0.1 (allows external connections)
- `--request-secrets` — Request secrets from the peer
- `--save-secrets <FILE>` — Save received secrets to a file (requires --request-secrets)
- `--no-tui` — Disable the TUI dashboard

---

### Show node info

```bash
rift info
```

Displays your peer ID and Rift link.

---

## Recipes

### Share API + Database together

```bash
# Terminal 1: Share your API
rift share 3000 --secrets .env.rift

# Terminal 2: Share your Postgres  
rift share 5432
```

Teammate connects to both and has your full backend stack on their localhost.

---

### Quick UI demo without deployment

```bash
# You're developing a Gradio app
python app.py  # Running on :7860

# Share it instantly
rift share 7860

# Teammate views on their machine
rift connect rift://...  # localhost:7860 shows your app
```

---

### Cross-machine debugging

```bash
# Your service is failing on your machine
rift share 3000 --secrets .env.rift

# Teammate connects and runs tests against YOUR running service
rift connect rift://... --request-secrets --save-secrets .env
pytest tests/  # Tests hit your actual service
```

---

### Share GPU model server

```bash
# GPU machine: run a model server
python -m vllm.entrypoints.openai.api_server --model mistral-7b
rift share 8000

# Laptop: use it as if local
rift connect rift://... 
curl http://localhost:8000/v1/completions ...
```
