#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

LIB_NAME="vault_signer_ffi"
OUT_DIR="${1:-$ROOT/bindings}"

echo "Building $LIB_NAME..."
cargo build -p vault-signer-ffi

# macOS host library for local Swift/Kotlin codegen smoke tests.
if [[ "$OSTYPE" == "darwin"* ]]; then
  LIB_PATH="$ROOT/target/debug/lib${LIB_NAME}.dylib"
else
  LIB_PATH="$ROOT/target/debug/lib${LIB_NAME}.so"
fi

if [[ ! -f "$LIB_PATH" ]]; then
  echo "Library not found at $LIB_PATH" >&2
  exit 1
fi

mkdir -p "$OUT_DIR/swift" "$OUT_DIR/kotlin"

echo "Generating Swift bindings -> $OUT_DIR/swift"
cargo run -p vault-signer-ffi --bin uniffi-bindgen -- generate \
  --library \
  --language swift \
  --out-dir "$OUT_DIR/swift" \
  "$LIB_PATH"

echo "Generating Kotlin bindings -> $OUT_DIR/kotlin"
cargo run -p vault-signer-ffi --bin uniffi-bindgen -- generate \
  --library \
  --language kotlin \
  --out-dir "$OUT_DIR/kotlin" \
  "$LIB_PATH"

echo "Done."
