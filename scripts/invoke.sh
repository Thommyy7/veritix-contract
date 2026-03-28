#!/usr/bin/env bash
# Invoke VeritixToken contract functions on localnet or testnet.
#
# Required environment variables:
#   CONTRACT_ID      — Contract address returned by deploy.sh
#   STELLAR_NETWORK  — "localnet" or "testnet" (default: localnet)
#   STELLAR_ACCOUNT  — Stellar account alias (default: alice)
#
# Usage examples:
#   CONTRACT_ID=C... ./scripts/invoke.sh mint alice bob 1000
#   CONTRACT_ID=C... ./scripts/invoke.sh transfer alice bob 100
#   CONTRACT_ID=C... ./scripts/invoke.sh balance alice

set -euo pipefail

NETWORK="${STELLAR_NETWORK:-localnet}"
ACCOUNT="${STELLAR_ACCOUNT:-alice}"
CONTRACT_ID="${CONTRACT_ID:?CONTRACT_ID is required}"
COMMAND="${1:?Usage: invoke.sh <command> [args...]}"

invoke() {
  stellar contract invoke \
    --id "$CONTRACT_ID" \
    --source "$ACCOUNT" \
    --network "$NETWORK" \
    -- "$@"
}

case "$COMMAND" in
  mint)
    # mint <admin> <to> <amount>
    ADMIN_ADDR=$(stellar keys address "${2:-$ACCOUNT}" --network "$NETWORK")
    TO_ADDR=$(stellar keys address "${3:-$ACCOUNT}" --network "$NETWORK")
    invoke mint --admin "$ADMIN_ADDR" --to "$TO_ADDR" --amount "${4:-1000}"
    ;;
  transfer)
    # transfer <from> <to> <amount>
    FROM_ADDR=$(stellar keys address "${2:-$ACCOUNT}" --network "$NETWORK")
    TO_ADDR=$(stellar keys address "${3:-$ACCOUNT}" --network "$NETWORK")
    invoke transfer --from "$FROM_ADDR" --to "$TO_ADDR" --amount "${4:-100}"
    ;;
  balance)
    # balance <address>
    ADDR=$(stellar keys address "${2:-$ACCOUNT}" --network "$NETWORK")
    invoke balance --id "$ADDR"
    ;;
  approve)
    # approve <from> <spender> <amount> <expiration_ledger>
    FROM_ADDR=$(stellar keys address "${2:-$ACCOUNT}" --network "$NETWORK")
    SPENDER_ADDR=$(stellar keys address "${3:-$ACCOUNT}" --network "$NETWORK")
    invoke approve --from "$FROM_ADDR" --spender "$SPENDER_ADDR" \
      --amount "${4:-100}" --expiration_ledger "${5:-1000000}"
    ;;
  *)
    echo "Running raw invoke: $*"
    invoke "$@"
    ;;
esac
