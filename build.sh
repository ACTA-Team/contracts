#!/bin/sh
soroban contract build --package acta-did-contract
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_did_contract.wasm
soroban contract build
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_deployer_contract.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_vault_contract.wasm
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/acta_issuance_contract.wasm
