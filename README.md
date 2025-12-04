# Shutdown Guard

Cross-platform Rust library for executing cleanup callbacks before system shutdown.

## Usage

```rust
use shutdown_guard::ShutdownGuard;

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

## Platform Support

- **macOS**: Signal handlers (SIGTERM, SIGINT, SIGHUP)
- **Windows**: WM_QUERYENDSESSION, WM_ENDSESSION
- **Linux**: systemd-logind D-Bus signals

## Notes

- Callbacks must complete quickly (< 5 seconds)
- Use `sync_all()` to flush data to disk
- macOS: Run in background to avoid shutdown dialog
