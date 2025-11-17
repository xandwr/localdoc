#!/bin/bash
set -e

echo "Building localdoc..."
cargo build --release

echo ""
echo "Build complete! The binary is at: target/release/localdoc"
echo ""
echo "To install system-wide, run:"
echo "  sudo cp target/release/localdoc /usr/local/bin/"
echo ""
echo "Or add to your PATH:"
echo "  export PATH=\"\$PATH:$(pwd)/target/release\""
