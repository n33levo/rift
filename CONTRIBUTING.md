# Contributing to Rift

Thanks for helping improve Rift — a pairing-grade, peer-to-peer localhost tunneling tool.

## Quick Start (Dev)

### Prerequisites
- **Rust:** stable toolchain (1.75+) via [rustup.rs](https://rustup.rs)
- **Build:** `cargo build`
- **Test:** `cargo test`
- **Run locally:** `cargo run -- <args>`

> **Note:** Rift uses libp2p (QUIC transport) and the Noise Protocol. On some systems you may need OpenSSL development headers. On Ubuntu/Debian: `sudo apt-get install libssl-dev pkg-config`.

---

## Project Goals

Rift optimizes for:
- **Pairing workflows:** Local-to-local port mapping for teammates debugging together
- **Secure by default:** Noise Protocol encryption, explicit connection approval
- **Explicit consent:** Every inbound connection requires host approval
- **Optional secrets handoff:** Config/env sharing is opt-in, never implicit

**Non-goals:**
- Being a public hosting platform (use ngrok / Cloudflare Tunnel for that)
- "Magic links" that bypass explicit approval
- Silent or automatic secrets exfiltration

---

## Repo Structure

```
rift/
├── crates/
│   ├── wh-core/           # Core P2P networking and tunneling
│   │   ├── network/       # libp2p swarm, QUIC transport, peer discovery
│   │   ├── secrets.rs     # EnvVault: encrypted secrets (X25519 + AES-GCM)
│   │   ├── crypto.rs      # Noise Protocol, key exchange
│   │   └── proxy/         # TCP ↔ QUIC stream bridging
│   ├── wh-daemon/         # Background daemon, session management
│   └── wh-cli/            # CLI commands and TUI
│       ├── cli/           # Command implementations (share, connect, info)
│       └── tui/           # Terminal UI (ratatui-based dashboard)
└── target/release/rift    # Compiled binary
```

---

## Security Model (Important)

When contributing, **preserve these invariants:**

### 1. Explicit Host Approval
- Every inbound connection **must** trigger a host approval prompt (unless `--auto-approve` is explicitly set).
- No "magic links" that auto-accept connections.

### 2. No Secrets Exfiltration
- Secrets sharing is **opt-in only**.
- Host must use `--secrets <file>` to share.
- Peer must use `--request-secrets` to receive.
- Both actions should be visible and auditable in logs/UI.

### 3. Encryption by Default
- All tunnel traffic uses QUIC + Noise Protocol.
- All secret payloads use X25519 (ECDH) + AES-256-GCM.
- No plaintext secrets on the wire.

### 4. No Silent Persistence
- Secrets should not be automatically saved to disk unless the user explicitly requests it (`--save-secrets`).
- Identity keys are stored securely in the system keyring where possible.

**If you change anything in the trust boundary** (handshake, approval flow, secret handling, key storage), **open a PR with:**
- A short threat-model note (what changes, what could go wrong)
- Tests or reproduction steps demonstrating the security property is maintained

---

## Coding Standards

- **Formatting:** `cargo fmt` before committing
- **Linting:** `cargo clippy -- -D warnings` (treat warnings as errors)
- **Prefer small, focused PRs:** Easier to review, faster to merge

### Style Guide
- Use descriptive variable names
- Add doc comments (`///`) for public APIs
- Keep functions small and single-purpose
- Handle errors explicitly (avoid `.unwrap()` in production code)

---

## Testing Guidelines

### Unit Tests
- Parser/config/crypto helper functions should have unit tests
- Run: `cargo test --lib`

### Integration Tests
Where feasible, add tests for:
- **Handshake approval path:** Verify that connections are rejected without approval
- **Port mapping correctness:** Ensure traffic flows correctly through the tunnel
- **Secrets opt-in behavior:** Verify secrets are only sent when explicitly requested

### Manual Testing
For end-to-end scenarios (requires two terminals or two machines):

**Terminal 1 - Start a test server:**
```bash
python3 -m http.server 3000
```

**Terminal 2 - Share the port:**
```bash
cargo run --release -- share 3000 --secrets .env.test
```

**Terminal 3 - Connect:**
```bash
cargo run --release -- connect rift://<LINK> --request-secrets
```

**Terminal 4 - Test the tunnel:**
```bash
curl http://localhost:3000
```

**Document manual tests** in your PR if they can't be automated.

---

## Issue Reporting

When filing an issue, include:
- **OS + Rust version:** `uname -a` and `rustc --version`
- **Command used:** Full command with flags (redact secrets!)
- **Logs:** Run with `RUST_LOG=debug` and include relevant output
- **Expected vs actual behavior:** What should happen vs what did happen

---

## Pull Request Checklist

Before submitting a PR:
- [ ] `cargo test` passes
- [ ] `cargo fmt` applied
- [ ] `cargo clippy -- -D warnings` clean
- [ ] README/docs updated if behavior changes
- [ ] Security invariants maintained:
  - [ ] Connection approval required (unless `--auto-approve`)
  - [ ] Secrets sharing is opt-in
  - [ ] Encryption enabled by default
  - [ ] No silent persistence of secrets

---

## Code of Conduct

- Be respectful and constructive
- Assume good faith in discussions
- No harassment, discrimination, or toxic behavior
- Focus on the code and ideas, not the person

---

## Roadmap / Areas Needing Help

- **Platform support:** Windows testing and compatibility fixes
- **CI/CD:** GitHub Actions for automated testing and releases
- **Packaging:** Homebrew formula, pre-built binaries
- **Documentation:** Tutorials, architecture diagrams, security audits
- **Performance:** Benchmarking, optimization of QUIC stream handling

---

Thank you for making Rift better! 🚀 
