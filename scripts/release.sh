#!/bin/sh
# Config testnet in local.
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate key to sign the transactions.
soroban keys generate acta_admin --network testnet

# Install and deploy contracts.
echo "Vault contract WASM ID:"
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/vault_contract.optimized.wasm \
  --source acta_sc_source \
  --network testnet

echo "Issuance contract WASM ID:"
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/issuance_contract.optimized.wasm \
  --source acta_sc_source \
  --network testnet
