#!/bin/sh
# Config testnet in local.
soroban config network add testnet \
  --rpc-url https://soroban-testnet.stellar.org:443 \
  --network-passphrase "Test SDF Network ; September 2015"

# Generate key to sign the transactions.
soroban keys generate acta_sc_source --network testnet

# Install and deploy contracts.
echo "Vault contract WASM ID:"
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/acta_vault_contract.optimized.wasm \
  --source acta_sc_source \
  --network testnet

echo "Issuance contract WASM ID:"
soroban contract install \
  --wasm target/wasm32-unknown-unknown/release/acta_issuance_contract.optimized.wasm \
  --source acta_sc_source \
  --network testnet

echo "Deployer contract Address:"
soroban contract deploy \
  --wasm target/wasm32-unknown-unknown/release/acta_deployer_contract.optimized.wasm \
  --source acta_sc_source \
  --network testnet
