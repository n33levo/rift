# Security

Rift is built for **trust between teammates**, with security defaults that prevent accidents.

## Security Model

| Layer | How it works |
|-------|--------------|
| **Explicit approval** | Every connection triggers a Y/N prompt — no silent access |
| **End-to-end encrypted** | QUIC transport + Noise Protocol (X25519, ChaCha20-Poly1305) |
| **Peer-to-peer** | Direct connection — no relay sees your plaintext traffic |
| **Localhost by default** | Client binds to `127.0.0.1` — your tunnel isn't exposed to your network |
| **Secrets are opt-in** | Must explicitly use `--secrets` to share, `--request-secrets` to receive |
| **Session-scoped** | Connections and secrets live only for the session |

---

## Encryption Details

### Transport Layer
- **QUIC** — UDP-based, multiplexed, built-in encryption
- **Noise Protocol** — Modern cryptographic handshake framework
- **X25519** — Elliptic curve Diffie-Hellman key exchange
- **ChaCha20-Poly1305** — Authenticated encryption for all tunnel traffic

### Secrets Handling
```bash
# Host explicitly shares specific config
rift share 3000 --secrets .env.rift   # Only what's in .env.rift

# Peer explicitly requests and saves
rift connect rift://... --request-secrets --save-secrets .env.local
```

**Encryption process:**
1. **X25519 key exchange** → unique shared secret per session
2. **AES-256-GCM encryption** → secrets encrypted before transit
3. **No automatic persistence** → secrets saved only with `--save-secrets`
4. **System keyring storage** → identity keys stored securely via OS keyring

---

## Best Practices

### For Secrets Sharing

✅ **DO:**
- Create a `.env.rift` with only the variables needed (subset of your `.env`)
- Review what you're sharing before using `--secrets`
- Use `--request-secrets` explicitly when you need config
- Audit what secrets were received before using them

❌ **DON'T:**
- Share your entire `.env` file (may contain production credentials)
- Use `--auto-approve` on untrusted networks
- Share secrets with people you don't trust
- Commit `.env.rift` to version control if it contains real secrets

### For Network Security

✅ **DO:**
- Keep default `127.0.0.1` binding unless you need LAN access
- Approve connections only from teammates you recognize
- Use `--no-tui` mode for automation, but monitor logs

❌ **DON'T:**
- Use `--public` binding without understanding the implications
- Share links publicly (they contain your peer ID)
- Run on untrusted networks without reviewing approval prompts

---

## Trust Model

Rift assumes:
- You **trust the peers** you connect to
- You're in a **collaborative environment** (team, OSS project)
- Connections are **temporary** (for debugging/pairing sessions)

Rift does NOT:
- Authenticate users (it's peer-to-peer)
- Prevent a malicious peer from reading traffic
- Protect against compromised teammate machines

**If you need zero-trust networking, use a VPN solution like Tailscale instead.**

---

## Threat Model

### What Rift protects against

✅ Passive network eavesdropping (encryption)  
✅ Accidental exposure to local network (localhost binding)  
✅ Silent secret exfiltration (explicit approval + opt-in)  
✅ Relay server snooping (P2P design)  

### What Rift does NOT protect against

❌ Malicious peers you approve  
❌ Compromised teammate machines  
❌ Network-level attacks if peers are untrusted  
❌ Long-term credential storage (use a proper secret manager)  

---

## Reporting Security Issues

If you discover a security vulnerability, please email the maintainers instead of opening a public issue.
