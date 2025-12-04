#!/bin/bash
# Alternative build script using cargo-zigbuild for cross-compilation on macOS

set -e

echo "Building shutdown_demo for all platforms using zigbuild..."
echo ""

# Check if cargo-zigbuild is installed
if ! command -v cargo-zigbuild &> /dev/null; then
    echo "⚠️  cargo-zigbuild not found. Installing..."
    cargo install cargo-zigbuild
fi

# Check if zig is installed
if ! command -v zig &> /dev/null; then
    echo "⚠️  zig not found. Installing via homebrew..."
    brew install zig
fi

mkdir -p dist

# macOS (native)
echo "[1/3] Building for macOS (native)..."
cargo build --release --example shutdown_demo
cp target/release/examples/shutdown_demo dist/shutdown_demo-macos
echo "✓ macOS build complete"
echo ""

# Linux
echo "[2/3] Building for Linux (using zig)..."
if cargo zigbuild --release --example shutdown_demo --target x86_64-unknown-linux-gnu; then
    cp target/x86_64-unknown-linux-gnu/release/examples/shutdown_demo dist/shutdown_demo-linux
    echo "✓ Linux build complete"
else
    echo "✗ Linux build failed"
fi
echo ""

# Windows
echo "[3/3] Building for Windows (using zig)..."
if cargo zigbuild --release --example shutdown_demo --target x86_64-pc-windows-gnu; then
    cp target/x86_64-pc-windows-gnu/release/examples/shutdown_demo.exe dist/shutdown_demo-windows.exe
    echo "✓ Windows build complete"
else
    echo "✗ Windows build failed"
fi

echo ""
echo "====================================="
echo "Build complete!"
echo "Binaries are in the 'dist' directory:"
ls -lh dist/ 2>/dev/null || echo "No files in dist/"
echo "====================================="
