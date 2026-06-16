#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> 1/2 UniFFI bindings..."
bash scripts/generate-bindings.sh

echo "==> 2/2 Android shared library..."
bash scripts/build-android-lib.sh

echo ""
echo "Open android/ in Android Studio and sync Gradle."
