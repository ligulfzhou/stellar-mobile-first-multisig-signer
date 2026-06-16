# Vault Signer

Mobile-first multisig treasury signer for Stellar Soroban. Rust core with UniFFI bindings for iOS and Android.

## Repository layout

```
multisig-wallet/
├── contracts/          # Soroban: vault, factory, registry
├── crates/
│   ├── stellar-core/   # Keys, RPC, tx build/sign/submit
│   ├── vault-client/   # Vault contract client
│   ├── vault-signer-cli/
│   └── vault-signer-ffi/   # UniFFI → Swift/Kotlin
├── bindings/           # Generated mobile bindings (just ffi)
└── ios/                # SwiftUI app (WIP)
```

## Build

```bash
# Rust signer + CLI
cargo build --release
just build

# Soroban contracts (requires stellar CLI)
just contract-build
just contract-test
```

## CLI

Point at a deployed vault contract (`C...` address):

```bash
export VAULT_ADDRESS=C...your_deployed_vault...
export STELLAR_SECRET=S...signer_secret...   # for approve/reject only

cargo run -p vault-signer-cli -- config
cargo run -p vault-signer-cli -- signers
cargo run -p vault-signer-cli -- proposals
cargo run -p vault-signer-cli -- proposals --pending-only
cargo run -p vault-signer-cli -- proposal --id 1
cargo run -p vault-signer-cli -- approve --id 2
```

Deploy a vault first — see [contracts/README.md](contracts/README.md).

## UniFFI (mobile)

```bash
just ffi        # bindings/swift + bindings/kotlin
just ffi-test   # needs VAULT_ADDRESS env
```

```swift
let signer = VaultSigner()
let pending = try signer.listPendingProposals(
    vault: vaultAddress,
    network: "testnet",
    rpcUrl: nil
)
```

## Testnet

- RPC: `https://soroban-testnet.stellar.org`
- Horizon: `https://horizon-testnet.stellar.org`

## Roadmap

1. iOS SwiftUI shell (Keychain, pending list, approve flow)
2. Android Compose shell
3. Push notifications + deep links
4. Contract audit + mainnet deploy
