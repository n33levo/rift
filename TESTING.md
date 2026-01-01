# Testing PortKey

## Quick Smoke Tests

Run the basic smoke tests to verify the code compiles and basic functionality works:

```bash
cargo test -p pk-core --test smoke_test
```

These tests verify:
- TCP echo server basics
- Function signatures exist
- Config creation works

## Integration Test (End-to-End Tunnel)

The full integration test verifies the complete tunnel data path:
`Client → TCP:8080 → Peer B → QUIC Stream → Peer A → TCP:3000 → Target Server`

⚠️ **Note**: This test is currently marked with `#[ignore]` because it requires:
- Full P2P connection establishment (mDNS, hole punching, etc.)
- Network timeouts
- May be flaky in CI environments

To run the integration test manually:

```bash
cargo test -p pk-core --test tunnel_integration -- --ignored --nocapture
```

Expected output:
```
=== Starting End-to-End Tunnel Test ===

[Setup] Starting target server on port 3000
[Target Server] Listening on 127.0.0.1:3000
[Setup] Starting Peer A (sharer)
[Peer A] Link: pk://12D3KooW...
[Peer A] Started listening
[Peer A] Waiting for incoming streams...
[Setup] Starting Peer B (connector)
[Peer B] Connecting to Peer A: pk://12D3KooW...
[Peer B] Waiting for connection to establish...
[Peer B] Local listener started on 127.0.0.1:8080

[Test] Waiting for tunnel to stabilize...
[Test] Sending request through tunnel...
[Test Client] Connecting to 127.0.0.1:8080
[Peer B] Incoming TCP connection from 127.0.0.1:...
[Peer B] Opened stream to peer, starting bridge
[Peer A] Incoming stream! Bridging to localhost:3000
[Target Server] Accepted connection from 127.0.0.1:...
[Target Server] Received: GET / HTTP/1.1
[Target Server] Sent response
[Test Client] Received 72 bytes: HTTP/1.1 200 OK...

✅ Test PASSED! Tunnel is working end-to-end!
```

## Manual End-to-End Testing

For manual testing with real processes:

### Terminal 1 - Start a test HTTP server
```bash
python3 -m http.server 8000
```

### Terminal 2 - Share the server
```bash
cargo run --release -- share 8000 --no-tui
```
Copy the `pk://` link shown.

### Terminal 3 - Connect to the tunnel
```bash
cargo run --release -- connect <pk-link> --no-tui
```

### Terminal 4 - Test the tunnel
```bash
curl http://localhost:8000
```

You should see the directory listing from the Python server!

## Debugging Tips

If the tunnel doesn't work:

1. **Check logs**: Run with `RUST_LOG=debug`
   ```bash
   RUST_LOG=pk_core=debug,pk_daemon=debug cargo run -- share 8000 --no-tui
   ```

2. **Verify P2P connection**: Look for "Connected to peer" messages

3. **Check firewall**: Ensure QUIC (UDP) isn't blocked

4. **Test locally**: Both peers on same machine should work via mDNS

5. **Port conflicts**: Ensure target port is actually running a server
