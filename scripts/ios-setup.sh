#!/usr/bin/env bash
# Prepare iOS Xcode project: FFI bindings + static libs + xcodeproj.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> 1/3 UniFFI bindings..."
bash scripts/generate-bindings.sh

echo "==> 2/3 iOS static libraries..."
bash scripts/build-ios-lib.sh

echo "==> 3/3 Xcode project..."
if command -v xcodegen >/dev/null 2>&1; then
  (cd ios && xcodegen generate)
  echo "    Generated ios/VaultSigner.xcodeproj"
else
  echo "    Install xcodegen to auto-generate the project:"
  echo "      brew install xcodegen && cd ios && xcodegen generate"
fi

echo ""
echo "Open in Xcode:"
echo "  open ios/VaultSigner.xcodeproj"
