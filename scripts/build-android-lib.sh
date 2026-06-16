#!/usr/bin/env bash
# Build vault-signer-ffi shared library for Android arm64.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
JNI_DIR="$ROOT/android/app/src/main/jniLibs/arm64-v8a"
PROFILE="${PROFILE:-release}"

# Prefer env; fall back to common macOS Android SDK layout.
export ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-${ANDROID_NDK:-$HOME/Library/Android/sdk/ndk/30.0.14904198}}"
NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$NDK_BIN/aarch64-linux-android26-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_AR="$NDK_BIN/llvm-ar"
export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android26-clang"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"
export CFLAGS_aarch64_linux_android="--target=aarch64-linux-android26"

mkdir -p "$JNI_DIR"

echo "==> Building for aarch64-linux-android ($PROFILE)..."
cargo build -p vault-signer-ffi --"$PROFILE" --target aarch64-linux-android

cp "$ROOT/target/aarch64-linux-android/$PROFILE/libvault_signer_ffi.so" "$JNI_DIR/"

echo "==> Copied to android/app/src/main/jniLibs/arm64-v8a/"
