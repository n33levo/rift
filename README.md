<div align="center">

# ⚡ Rift

**Your teammate's localhost. On your localhost. Because "just push to staging" is so 2019.**

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Built with libp2p](https://img.shields.io/badge/built%20with-libp2p-blueviolet.svg)](https://libp2p.io)

<br>

<img src="assets/screenshot.png" alt="Rift TUI" width="700">

<br>

**Stop deploying to staging. Stop screen-sharing. Stop Slack-ing `.env` screenshots at 2am.**

</div>

---

## What's This?

Remember the last time your teammate said *"works on my machine"* and you wanted to reach through Slack and use *their* machine? 

**Now you can.**

Rift makes their `localhost:3000` appear on *your* `localhost:3000`. Encrypted. P2P. No public URLs. No servers. No BS.

```
Your machine                    Teammate's machine
localhost:3000  ◄─ encrypted ─►  localhost:3000
     ↓                                ↓
  your app                      their broken API
```

Finally debug against the *actual* thing that's broken, not a "reproduction" that mysteriously works.

---

## 30-Second Demo

```bash
# Teammate A (has the cursed service)
rift share 3000

# Teammate B (needs to see the curse firsthand)
rift connect rift://12D3KooW.../3000
```

Done. `localhost:3000` on B's machine → A's service. Encrypted, P2P, explicit approval.

**Bonus:** Share the config too, because "just set these 47 env vars" is nobody's idea of fun:
```bash
rift share 3000 --secrets .env.rift
rift connect rift://... --request-secrets --save-secrets .env
```

---

## Install

**Homebrew (macOS/Linux):**
```bash
brew tap n33levo/rift
brew install rift
```

**From source** (if you enjoy waiting for Rust to compile):
```bash
git clone https://github.com/n33levo/rift
cd rift
cargo build --release
# Go make coffee. Maybe lunch. Possibly dinner.
```

---

## Why Tho

**The problem:** Your teammate's service works. Yours doesn't. They say "just run it locally." You try. It doesn't work. They send you their `.env`. Still doesn't work. They screen-share. You see it working. You cry.

**The solution:** Just... use their service. On your `localhost`. Like you're sitting at their desk. But you're not. You're in your underwear. They don't need to know that.

**Why Rift:**
- 🚫 **No more "deploy to staging just to test"** — tunnel straight to their machine
- 🎭 **No more "send me your .env in Slack"** — encrypted config handoff built-in
- 🔒 **No more "oops I exposed my dev DB"** — localhost binding by default, explicit approval
- 🌐 **No more "works on my machine"** — literally use *their* machine
- ⚡ **Share anything:** APIs, databases, Streamlit apps, Jupyter, GPU servers, that weird internal tool

It's like ngrok (The Boring But Important Part)

1. **mDNS discovery** — finds peers on your network (same WiFi? instant)
2. **QUIC tunnel** — establishes encrypted P2P connection (relays for NAT, then direct)
3. **Y/N approval** — your teammate sees a prompt and consciously decides to trust you
4. **TCP bridge** — bytes flow from your `localhost` → encrypted stream → their `localhost`
5. **Magic** — it just works™

Built on [libp2p](https://libp2p.io) (same tech as IPFS). Encrypted end-to-end (X25519 + ChaCha20-Poly1305). NAT hole-punching (DCUtR). All the fancy acronyms.

[→ Architecture deep dive](docs/HOW_IT_WORKS.md) (for when you can't sleep

**mDNS discovery** → **QUIC tunnel** → **Y/N approval** → **TCP bridge** → **Done**

Built on [libp2p](https://libp2p.io) (IPFS stack). Encrypted end-to-end. Direct P2P (relays for NAT only).

[→ Architec (Yes, We Thought About It)

**Short version:** Encrypted end-to-end. Explicit approval. Localhost by default. Secrets opt-in only.

**Slightly longer version:**
- 🔐 **E2E encrypted** — X25519 key exchange, ChaCha20-Poly1305, all the crypto people smarter than us recommend
- 👤 **Explicit approval** — every connection needs a Y/N from the host (no surprise visitors)
- 🏠 **Localhost binding** — your tunnel isn't exposed to your coffee shop WiFi by default
- 🔑 **Secrets encrypted** — AES-256-GCM, only if both sides opt-in, session-only

**Trust model:** Made for teammates you trust. Not for randos on the internet. If you need zero-trust, use Tailscale.

[→ Full security details](docs/SECURITY.md) (threat model, best practices, the whole dealns
- Localhost binding by default
- Secrets encrypted with AES-256-GCM

[→ Security details](docs/SECURITY.md)

---

## What This ISN'T

Let's be clear so nobody's disappointed:

- ❌ **Not for public URLs** → Use ngrok/Cloudflare Tunnel for that
- ❌ **Not for production** → Please use actual infrastructure
- ❌ **Not for permanent infra** → Sessions are ephemeral by design
- ❌ **Not for people you don't trust** → Share with teammates, not Twitter

If you need to show a client a demo, you want ngrok. If you need to debug with your teammate at 11pm, you want Rift.model, best practices
- [How It Works](docs/HOW_IT_WORKS.md) — Architecture deep dive

---

## Not For

❌ Public URLs (use ngrok)  
❌ Production (use real infra)  
❌ Untrusted parties (made for teammates)

---

## Contributing

## FAQ Nobody Asked But We'll Answer Anyway

**Q: Is this like ngrok?**  
A: If ngrok and SSH had a baby, and that baby was raised by libp2p. Also no public URLs.

**Q: Why not just use SSH tunnels?**  
A: Because it's 2026 and we have better things to do than debug SSH config and port forwarding.

**Q: Is this secure?**  
A: Yes. See [SECURITY.md](docs/SECURITY.md). We use the same crypto as WireGuard and WhatsApp.

**Q: Can I share multiple ports?**  
A: Yes. Run `rift share` in multiple terminals. Live your best life.

**Q: Does this work over the internet?**  
A: Yes. Relay-assisted bootstrap → NAT hole-punching → direct P2P. (Usually.)

**Q: What if my teammate is evil?**  
A: Don't share with evil people. This is a people problem, not a tech problem.

---

<div align="center">

**Made with ⚡ by developers who got tired of deploying to staging**

*"It's like AirDrop, but for localhost ports. And with more cyberpunk vibes."*

[⭐ Star if you've ever said "works on my machine" ⭐](https://github.com/n33levo/rift)

## License

MIT

---

<div align="center">

**Made for developers who pair**

</div>
