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

## Deploy (testnet)

From repo root (uses `admin` identity):

```bash
chmod +x scripts/deploy-testnet.sh
just deploy-testnet
```

This will:

1. Upload `multisig_vault.wasm`
2. Deploy + initialize `multisig_vault_factory`
3. `create_vault` with admin + alice (2-of-2), fee = 0
4. Write addresses to `deploy/testnet.json`

Override: `VAULT_NAME=myteam THRESHOLD=1 SOURCE=admin just deploy-testnet`
