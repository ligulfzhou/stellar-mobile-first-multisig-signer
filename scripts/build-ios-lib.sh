#!/usr/bin/env bash
# Build vault-signer-ffi static libraries for iOS device + simulator.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VENDOR="$ROOT/ios/Vendor"
PROFILE="${PROFILE:-release}"

mkdir -p "$VENDOR/ios-sim" "$VENDOR/ios"

echo "==> Building for aarch64-apple-ios-sim ($PROFILE)..."
cargo build -p vault-signer-ffi --"$PROFILE" --target aarch64-apple-ios-sim

echo "==> Building for aarch64-apple-ios ($PROFILE)..."
cargo build -p vault-signer-ffi --"$PROFILE" --target aarch64-apple-ios

cp "$ROOT/target/aarch64-apple-ios-sim/$PROFILE/libvault_signer_ffi.a" "$VENDOR/ios-sim/"
cp "$ROOT/target/aarch64-apple-ios/$PROFILE/libvault_signer_ffi.a" "$VENDOR/ios/"

echo "==> Libraries copied to ios/Vendor/{ios-sim,ios}/"
echo "    Run from repo root: just ios-setup"
