#!/usr/bin/env bash
# Deploy multisig-vault contracts to Stellar testnet using local `admin` identity.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CONTRACTS="$ROOT/contracts"
DEPLOY_DIR="$ROOT/deploy"
OUT="$DEPLOY_DIR/testnet.json"

SOURCE="${SOURCE:-admin}"
NETWORK="${NETWORK:-testnet}"
# Official SDF RPC is blocked/refused in some regions; Gateway.fm works as fallback.
RPC_URL="${RPC_URL:-${STELLAR_RPC_URL:-https://soroban-rpc.testnet.stellar.gateway.fm}}"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-${STELLAR_NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}}"
RPC_ARGS=(--rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE")
NATIVE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
VAULT_NAME="${VAULT_NAME:-demo}"
THRESHOLD="${THRESHOLD:-2}"

mkdir -p "$DEPLOY_DIR"

echo "==> Building contracts..."
cd "$CONTRACTS"
stellar contract build

WASM_VAULT="$CONTRACTS/target/wasm32v1-none/release/multisig_vault.wasm"
WASM_FACTORY="$CONTRACTS/target/wasm32v1-none/release/multisig_vault_factory.wasm"

ADMIN="$(stellar keys address "$SOURCE")"
ALICE="$(stellar keys address alice 2>/dev/null || true)"

echo "==> Admin: $ADMIN"
echo "==> RPC: $RPC_URL"
stellar network use "$NETWORK" >/dev/null

echo "==> Uploading vault WASM..."
set +e
UPLOAD_OUT="$(stellar contract upload "${RPC_ARGS[@]}" --wasm "$WASM_VAULT" --source "$SOURCE" --network "$NETWORK" 2>&1)"
UPLOAD_RC=$?
set -e
echo "$UPLOAD_OUT"
if [[ $UPLOAD_RC -ne 0 ]]; then
  echo "ERROR: upload failed. Try another RPC, e.g.:" >&2
  echo "  RPC_URL=https://rpc.ankr.com/stellar_testnet_soroban just deploy-testnet" >&2
  exit $UPLOAD_RC
fi
VAULT_WASM_HASH="$(echo "$UPLOAD_OUT" | grep -oE '[0-9a-f]{64}' | tail -1 || true)"
if [[ -z "$VAULT_WASM_HASH" ]]; then
  echo "ERROR: could not determine vault WASM hash" >&2
  exit 1
fi
echo "    vault wasm hash: $VAULT_WASM_HASH"

echo "==> Deploying factory..."
FACTORY_OUT="$(stellar contract deploy "${RPC_ARGS[@]}" --wasm "$WASM_FACTORY" --source "$SOURCE" --network "$NETWORK" 2>&1)"
echo "$FACTORY_OUT"
FACTORY_ID="$(echo "$FACTORY_OUT" | grep -oE 'C[A-Z0-9]{55}' | tail -1)"
if [[ -z "$FACTORY_ID" ]]; then
  echo "ERROR: could not parse factory contract id" >&2
  exit 1
fi
echo "    factory: $FACTORY_ID"

echo "==> Initializing factory..."
stellar contract invoke \
  "${RPC_ARGS[@]}" \
  --id "$FACTORY_ID" \
  --source "$SOURCE" \
  --network "$NETWORK" \
  -- initialize \
  --admin "$ADMIN" \
  --vault_wasm_hash "$VAULT_WASM_HASH" \
  --fee_token "$NATIVE_TOKEN" \
  --fee_amount 0 \
  --fee_recipient "$ADMIN"

# Signers: admin + alice (2-of-2) if alice exists, else admin only (1-of-1)
if [[ -n "$ALICE" && "$THRESHOLD" -gt 1 ]]; then
  SIGNERS_JSON="[\"${ADMIN}\",\"${ALICE}\"]"
else
  SIGNERS_JSON="[\"${ADMIN}\"]"
  THRESHOLD=1
fi

echo "==> Creating vault (signers=$SIGNERS_JSON, threshold=$THRESHOLD)..."
CREATE_OUT="$(stellar contract invoke \
  "${RPC_ARGS[@]}" \
  --id "$FACTORY_ID" \
  --source "$SOURCE" \
  --network "$NETWORK" \
  -- create_vault \
  --creator "$ADMIN" \
  --name "$VAULT_NAME" \
  --signers "$SIGNERS_JSON" \
  --threshold "$THRESHOLD" 2>&1)"
echo "$CREATE_OUT"
VAULT_ID="$(echo "$CREATE_OUT" | grep -oE 'C[A-Z0-9]{55}' | tail -1)"
if [[ -z "$VAULT_ID" ]]; then
  echo "ERROR: could not parse vault contract id" >&2
  exit 1
fi
echo "    vault: $VAULT_ID"

cat >"$OUT" <<EOF
{
  "network": "$NETWORK",
  "deployed_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "admin": "$ADMIN",
  "alice": "${ALICE:-}",
  "native_token": "$NATIVE_TOKEN",
  "rpc_url": "$RPC_URL",
  "vault_wasm_hash": "$VAULT_WASM_HASH",
  "factory_id": "$FACTORY_ID",
  "vault_id": "$VAULT_ID",
  "vault_name": "$VAULT_NAME",
  "threshold": $THRESHOLD
}
EOF

echo ""
echo "==> Deployment saved to $OUT"
echo "    export VAULT_ADDRESS=$VAULT_ID"
echo "    export FACTORY_ADDRESS=$FACTORY_ID"
echo ""
echo "Verify:"
echo "  cargo run -p vault-signer-cli -- --vault $VAULT_ID config"
echo "  cargo run -p vault-signer-cli -- --vault $VAULT_ID signers"
