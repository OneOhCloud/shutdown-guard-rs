#!/bin/bash
# Build script for cross-compilation on macOS

set -e

echo "Building shutdown_demo for all platforms..."
echo ""

# macOS (native)
echo "[1/3] Building for macOS (native)..."
cargo build --release --example shutdown_demo
mkdir -p dist
cp target/release/examples/shutdown_demo dist/shutdown_demo-macos
echo "✓ macOS build complete"
echo ""

# Check for cross tool and Docker
if ! command -v cross &> /dev/null; then
    echo "⚠️  'cross' tool not found. Install with: cargo install cross"
    echo "Skipping Linux and Windows builds..."
    exit 0
fi

if ! docker info > /dev/null 2>&1; then
    echo "⚠️  Docker is not running. Please start Docker Desktop."
    echo "Skipping Linux and Windows builds..."
    echo ""
    echo "To enable cross-compilation:"
    echo "  1. Install Docker Desktop for macOS"
    echo "  2. Start Docker Desktop"
    echo "  3. Run this script again"
    exit 0
fi

# Linux (using cross)
echo "[2/3] Building for Linux (using Docker)..."
if cross build --release --example shutdown_demo --target x86_64-unknown-linux-gnu; then
    cp target/x86_64-unknown-linux-gnu/release/examples/shutdown_demo dist/shutdown_demo-linux
    echo "✓ Linux build complete"
else
    echo "✗ Linux build failed"
fi
echo ""

# Windows (using cross)
echo "[3/3] Building for Windows (using Docker)..."
if cross build --release --example shutdown_demo --target x86_64-pc-windows-gnu; then
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
