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
├── deploy/             # Local deployment records (gitignored)
└── ios/                # SwiftUI app (WIP)
```

## Build

```bash
cargo build --release
just build

# Soroban contracts (requires stellar CLI)
just contract-build
```

## Deploy (testnet)

Uses local Stellar identity `admin` (and `alice` as second signer for 2-of-2):

```bash
just deploy-testnet
# → deploy/testnet.json with factory_id + vault_id
```

Default RPC is Gateway.fm (`soroban-rpc.testnet.stellar.gateway.fm`). The official SDF endpoint may be unreachable in some networks. Override:

```bash
RPC_URL=https://rpc.ankr.com/stellar_testnet_soroban just deploy-testnet
```

Then verify with CLI:

```bash
export VAULT_ADDRESS=$(jq -r .vault_id deploy/testnet.json)
cargo run -p vault-signer-cli -- config
cargo run -p vault-signer-cli -- signers
```

Details: [contracts/README.md](contracts/README.md)

## CLI

```bash
export VAULT_ADDRESS=C...
export STELLAR_SECRET=S...   # vault signer, for approve/reject

cargo run -p vault-signer-cli -- proposals --pending-only
cargo run -p vault-signer-cli -- approve --id 1
```

## UniFFI (mobile)

```bash
just ffi
just ffi-test   # optional; set VAULT_ADDRESS
```

## Testnet

- RPC: `https://soroban-testnet.stellar.org`
- Native XLM (SAC): `CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC`

## Roadmap

1. iOS SwiftUI shell (Keychain, pending list, approve flow)
2. Android Compose shell
3. Push notifications + deep links
4. Contract audit + mainnet deploy
