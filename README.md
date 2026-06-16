# Vault Signer

Mobile-first Stellar Vault signer infrastructure. Rust core extracted from [stellar-arb](https://github.com/ligulfzhou) and [stellar-dex-aggregator](https://github.com/ligulfzhou), targeting Soroban vault multisig on iOS/Android (UniFFI in a later phase).

## Workspace

| Crate | Purpose |
|-------|---------|
| `stellar-core` | Keys, RPC simulate/prepare, ScVal helpers, sign/submit |
| `vault-client` | Stellar Vault contract read/write (approve, reject, get_config, …) |
| `vault-signer-cli` | Phase 0 CLI to validate against testnet |

## Build

```bash
cargo build --release
```

## CLI usage (testnet)

Set your vault contract address (from [Stellar Vault dashboard](https://stellar-vault-eta.vercel.app/) or factory):

```bash
# Example: public test vault from Stellar Vault docs
export VAULT_ADDRESS=CBJ4BFOUDMQWFPCBALQTO2565STNGFMGQWDYVQ7MBWRZF5WSI2Z4VT5W
export STELLAR_SECRET=S...your_signer_secret...

# Read vault state
cargo run -p vault-signer-cli -- config
cargo run -p vault-signer-cli -- signers
cargo run -p vault-signer-cli -- proposal --id 1

# Approve proposal #1
cargo run -p vault-signer-cli -- approve --id 1
```

## Testnet defaults

From Stellar Vault fork (`dashboard/src/config.ts`):

- RPC: `https://soroban-testnet.stellar.org`
- Factory: `CCNGOW6UCZKELBAR377HDHWAJJLKD6SJHUFCDT4UM6M2AYPSOEBYLDVA`
- Registry: `CDJCQNXYTWZ3VF2FL2MCWMZB6RPQYRAFNNO6KEKW2MN7ALXGB5SGYTJ4`

## Next steps

1. UniFFI bindings (`vault-signer-ffi` crate)
2. Swift/Kotlin shell apps
3. Push notifications + deep links for pending proposals
4. Own Soroban vault contract (replace `vault-client` backend)
