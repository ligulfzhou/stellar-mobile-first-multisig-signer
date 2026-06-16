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

# Build UniFFI library and generate Swift/Kotlin bindings
ffi:
    cargo build -p vault-signer-ffi
    bash scripts/generate-bindings.sh

ffi-test:
    cargo test -p vault-signer-ffi -- --nocapture
