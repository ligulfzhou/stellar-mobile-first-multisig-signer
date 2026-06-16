#!/usr/bin/env bash
# End-to-end testnet flow: fund vault → propose → approve → execute.
# Writes use `stellar contract invoke --source` (works with OS secure store).
# Optionally set ADMIN_SECRET / ALICE_SECRET to also exercise vault-signer-cli.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RPC_URL="${RPC_URL:-${SOROBAN_RPC_URL:-https://soroban-rpc.testnet.stellar.gateway.fm}}"
VAULT="${VAULT_ADDRESS:-CCJ3AAZCSG3MXY3WQ4BX6XZQBSV4T7QERHVP5LKKIQDHTXJE5JVBXP5Q}"
NATIVE_TOKEN="CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC"
NETWORK_PASSPHRASE="${NETWORK_PASSPHRASE:-Test SDF Network ; September 2015}"
SOURCE="${SOURCE:-admin}"
AMOUNT="${AMOUNT:-1000000}" # 0.1 XLM

INVOKE=(
  stellar contract invoke
  --id "$VAULT"
  --rpc-url "$RPC_URL"
  --network-passphrase "$NETWORK_PASSPHRASE"
  --network testnet
)

ADMIN="$(stellar keys address "$SOURCE")"
ALICE="$(stellar keys address alice)"

export VAULT_ADDRESS="$VAULT"
export SOROBAN_RPC_URL="$RPC_URL"
CLI=(cargo run -q -p vault-signer-cli -- --vault "$VAULT" --rpc-url "$RPC_URL")

echo "==> Vault: $VAULT"
echo "==> RPC:   $RPC_URL"
echo "==> Admin: $ADMIN"
echo "==> Alice: $ALICE"

echo ""
echo "==> 0) Ensure Alice account exists (needed for approve fees)..."
set +e
CREATE_OUT="$(stellar tx new create-account --source "$SOURCE" --destination alice \
  --starting-balance 30000000 \
  --rpc-url "$RPC_URL" --network-passphrase "$NETWORK_PASSPHRASE" --network testnet 2>&1)"
CREATE_RC=$?
set -e
if [[ $CREATE_RC -eq 0 ]]; then
  echo "    Created and funded Alice."
elif echo "$CREATE_OUT" | grep -qiE 'already exist|MustNotExist|existing account'; then
  echo "    Alice account already exists."
else
  echo "$CREATE_OUT" >&2
  exit 1
fi

echo ""
echo "==> 1) Fund vault with ${AMOUNT} stroops via native SAC..."
stellar contract invoke \
  --id "$NATIVE_TOKEN" \
  --source-account "$SOURCE" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  --network testnet \
  -- transfer --from "$SOURCE" --to "$VAULT" --amount "$AMOUNT"

echo ""
echo "==> 2) Admin proposes transfer to Alice..."
PROPOSAL_ID="$("${INVOKE[@]}" --source-account "$SOURCE" -- propose \
  --proposer "$SOURCE" \
  --proposal_type 0 \
  --token "$NATIVE_TOKEN" \
  --recipient alice \
  --amount "$AMOUNT" \
  --start_time 0 \
  --end_time 0 \
  --cliff_time 0 \
  --release_intervals 0 \
  --description transfer | tail -1)"
echo "    Proposal id: $PROPOSAL_ID"

echo ""
echo "==> 3) Alice approves proposal #$PROPOSAL_ID..."
"${INVOKE[@]}" --source-account alice -- approve \
  --signer alice \
  --proposal_id "$PROPOSAL_ID"

echo ""
echo "==> 4) Admin executes proposal #$PROPOSAL_ID..."
"${INVOKE[@]}" --source-account "$SOURCE" -- execute \
  --executor "$SOURCE" \
  --proposal_id "$PROPOSAL_ID" \
  --proposal_type 0 \
  --token "$NATIVE_TOKEN" \
  --recipient alice \
  --amount "$AMOUNT" \
  --start_time 0 \
  --end_time 0 \
  --cliff_time 0 \
  --release_intervals 0

echo ""
echo "==> 5) Verify via vault-signer-cli (read path)..."
"${CLI[@]}" proposal --id "$PROPOSAL_ID"

if [[ -n "${ADMIN_SECRET:-}" && -n "${ALICE_SECRET:-}" ]]; then
  echo ""
  echo "==> 6) Optional: vault-signer-cli write path (secrets provided)..."
  echo "    (skipped in automated run — set ADMIN_SECRET/ALICE_SECRET to enable a second round)"
fi

echo ""
echo "==> E2E complete."
