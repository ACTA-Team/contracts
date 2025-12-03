#!/bin/sh
soroban contract build
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/vault_contract.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/issuance_contract.wasm
