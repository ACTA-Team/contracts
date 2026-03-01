#!/bin/sh
set -eu

# Config testnet in local (idempotent: skip if already configured).
stellar config network ls 2>/dev/null | grep -q testnet || \
  stellar config network add testnet \
    --rpc-url https://soroban-testnet.stellar.org:443 \
    --network-passphrase "Test SDF Network ; September 2015"

# Generate key to sign the transactions (idempotent: skip if key already exists).
stellar keys show vc_vault_admin 2>/dev/null || \
  stellar keys generate vc_vault_admin --network testnet

# Build + optimize
sh scripts/build.sh

echo "VC Vault contract ID:"
stellar contract deploy \
  --wasm target/wasm32v1-none/release/vc_vault_contract.optimized.wasm \
  --source vc_vault_admin \
  --network testnet
