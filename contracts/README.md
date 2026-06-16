# Soroban Contracts

Multisig treasury vault on Stellar Soroban.

| Contract | Crate | Role |
|----------|-------|------|
| Vault | `multisig-vault` | M-of-N proposals, approve/reject/execute, timelocks |
| Factory | `multisig-vault-factory` | Deploy and index vault instances |
| Registry | `multisig-vault-registry` | Contract deployment registry |

## Build

Requires [Stellar CLI](https://developers.stellar.org/docs/tools/cli):

```bash
cd contracts
make build    # stellar contract build
make test     # cargo test
```

WASM output: `target/wasm32v1-none/release/multisig_vault.wasm`

## Deploy (testnet)

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/multisig_vault.wasm \
  --network testnet \
  --source YOUR_ACCOUNT
```

After deploy, set `VAULT_ADDRESS` for the CLI and mobile signer.
