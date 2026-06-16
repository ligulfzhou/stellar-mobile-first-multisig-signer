set shell := ["bash", "-cu"]

alias b := build
alias c := check
alias f := fmt
alias t := test

build:
    cargo +nightly fmt
    cargo build -j1

check:
    cargo +nightly fmt
    cargo check -j1

fmt:
    cargo +nightly fmt

test:
    cargo +nightly fmt
    cargo test -- --test-threads=1

contract-build:
    cd contracts && stellar contract build

contract-test:
    cd contracts && cargo test

# Deploy factory + vault to testnet (requires funded `admin` identity)
deploy-testnet:
    bash scripts/deploy-testnet.sh

# Full on-chain flow: fund → propose → approve → execute
e2e-testnet:
    bash scripts/e2e-testnet.sh

# iOS: bindings + static libs + Xcode project (run after API changes)
ios-setup:
    bash scripts/generate-bindings.sh
    bash scripts/build-ios-lib.sh
    cd ios && xcodegen generate

ios-lib:
    bash scripts/build-ios-lib.sh

# Android: bindings + arm64 .so
android-setup:
    bash scripts/android-setup.sh

android-lib:
    bash scripts/build-android-lib.sh

# Build UniFFI library and generate Swift/Kotlin bindings
ffi:
    cargo build -p vault-signer-ffi
    bash scripts/generate-bindings.sh

ffi-test:
    cargo test -p vault-signer-ffi -- --nocapture
