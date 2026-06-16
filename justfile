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
    # cargo build -j1
    # ./target/debug/sui-arb test
