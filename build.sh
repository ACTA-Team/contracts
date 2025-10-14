#!/bin/sh
soroban contract build
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_deployer_contract.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_vault_contract.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_issuance_contract.wasm
