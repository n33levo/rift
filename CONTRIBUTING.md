# Contributing to Rift

Thank you for your interest in contributing to Rift! This document provides guidelines and instructions for contributing.

## Getting Started

### Prerequisites

- Rust 1.75 or higher (install via [rustup.rs](https://rustup.rs))
- Git
- Basic familiarity with P2P networking concepts (helpful but not required)

### Development Setup

1. **Fork the repository** on GitHub
2. **Clone your fork:**
   ```bash
   git clone https://github.com/yourusername/rift
   cd rift
   ```
3. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```
4. **Build the project:**
   ```bash
   cargo build
   ```

## Running Tests

### Quick Smoke Tests

Run basic unit tests to verify functionality:

```bash
cargo test
```

For verbose output:
```bash
cargo test -- --nocapture
```

### Integration Tests

Run specific integration tests:

```bash
cargo test -p wh-core --test tunnel_integration
```

### Manual Testing

For end-to-end manual testing:

**Terminal 1 - Start a test server:**
```bash
python3 -m http.server 3000
```

**Terminal 2 - Share the port:**
```bash
cargo run --release -- share 3000
```

**Terminal 3 - Connect to it:**
```bash
cargo run --release -- connect rift://<LINK-FROM-TERMINAL-2>
```

**Terminal 4 - Test the tunnel:**
```bash
curl http://localhost:3000
```

### Debug Logging

Enable debug logs for troubleshooting:

```bash
RUST_LOG=debug cargo run -- share 3000
```

## Code Style

- **Formatting:** Run `cargo fmt` before committing
- **Linting:** Run `cargo clippy` and fix warnings
- **Documentation:** Add doc comments for public APIs

## Submitting Changes

1. **Ensure all tests pass:** `cargo test`
2. **Format your code:** `cargo fmt`
3. **Check for issues:** `cargo clippy`
4. **Commit with clear messages:**
   ```bash
   git commit -m "feat: Add connection timeout handling"
   ```
5. **Push to your fork:**
   ```bash
   git push origin feature/your-feature-name
   ```
6. **Open a Pull Request** on GitHub

### Commit Message Guidelines

Use conventional commits format:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Example: `feat: Add automatic reconnection on connection loss`

## What to Contribute

### Good First Issues

- Documentation improvements
- Bug fixes
- Test coverage improvements
- Examples and tutorials

### Feature Ideas

- Custom domain support
- QR code generation for mobile
- Web UI alternative to TUI
- Performance optimizations
- Platform-specific improvements

### Areas Needing Help

- Windows support testing
- CI/CD pipeline setup
- Homebrew formula
- Binary release automation

## Code Review Process

1. Maintainers will review your PR within a few days
2. Address any requested changes
3. Once approved, your PR will be merged

## Getting Help

- **Questions?** Open a GitHub issue with the `question` label
- **Bug reports?** Open an issue with detailed reproduction steps
- **Feature requests?** Open an issue describing the use case

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for making Rift better! 🚀
