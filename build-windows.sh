#!/usr/bin/env bash
# Build kangaroo for Windows (x86_64) from macOS / Linux.
# Requires: Rust toolchain, cross (cargo install cross), and Docker (for cross).

set -euo pipefail

TARGET="x86_64-pc-windows-gnu"
BINARY="kangaroo"
OUT_DIR="dist/windows"

echo "=== Kangaroo — Windows build ==="

# Prefer 'cross' (Docker-based) which ships a complete MinGW toolchain.
if command -v cross &>/dev/null; then
    echo "[+] Using 'cross' for build (Docker backend)"
    cross build --release --target "$TARGET"
else
    # Fallback: native cargo with mingw toolchain
    echo "[+] 'cross' not found — using native cargo with $TARGET"
    rustup target add "$TARGET"
    # macOS: brew install mingw-w64
    cargo build --release --target "$TARGET"
fi

mkdir -p "$OUT_DIR"
BIN_SRC="target/$TARGET/release/$BINARY.exe"
cp "$BIN_SRC" "$OUT_DIR/"

echo ""
echo "=== Build complete ==="
echo "Binary: $OUT_DIR/$BINARY.exe"
echo ""
echo "Usage on Windows:"
echo "  kangaroo.exe --pubkey <hex> --start <hex> --range <bits>"
echo "  kangaroo.exe --pubkey <hex> --start <hex> --range <bits> --checkpoint my.json"
echo "  kangaroo.exe --pubkey <hex> --start <hex> --range <bits> --no-checkpoint"
