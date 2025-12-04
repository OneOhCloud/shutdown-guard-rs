# Shutdown Guard

Cross-platform Rust library for executing cleanup callbacks before system shutdown.

## Usage

```rust
use shutdown_guard_rs::ShutdownGuard;

let guard = ShutdownGuard::new();
guard.register(Box::new(|| {
    // Your cleanup code
}));
guard.start().unwrap();
```

## Demo

Example: `shutdown_demo` - records shutdown time to file

```bash
# Run example
cargo run --release --example shutdown_demo


```

## Build

```bash
# Build library
cargo build --release

# Build examples
cargo build --release --examples

# Run examples
cargo run --example basic
cargo run --example file_cleanup
cargo run --example shutdown_demo

# Test
cargo test
```

### Cross-compilation on macOS

#### Option 1: Using cargo-zigbuild (Recommended - No Docker needed)

**Prerequisites:**
```bash
brew install zig
cargo install cargo-zigbuild
rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu
```

**Build all platforms:**
```bash
./build-all-zig.sh
```

**Manual builds:**
```bash
# macOS (native)
cargo build --release --example shutdown_demo

# Linux
cargo zigbuild --release --example shutdown_demo --target x86_64-unknown-linux-gnu

# Windows
cargo zigbuild --release --example shutdown_demo --target x86_64-pc-windows-gnu
```

#### Option 2: Using cross with Docker

**Prerequisites:**
1. Install Docker Desktop for macOS and start it
2. Install cross: `cargo install cross --git https://github.com/cross-rs/cross`
3. Add targets: `rustup target add x86_64-unknown-linux-gnu x86_64-pc-windows-gnu`

**Build all platforms:**
```bash
./build-all.sh
```

**Note:** The `cargo-zigbuild` method is simpler as it doesn't require Docker.

## Platform Support

- **macOS**: Signal handlers (SIGTERM, SIGINT, SIGHUP)
- **Windows**: WM_QUERYENDSESSION, WM_ENDSESSION
- **Linux**: systemd-logind D-Bus signals

## Notes

- Callbacks must complete quickly (< 5 seconds)
- Use `sync_all()` to flush data to disk
- macOS: Run in background to avoid shutdown dialog
