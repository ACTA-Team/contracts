#!/bin/sh
set -eu

# Build the WASM artifact. We keep crate-type=["rlib"] in Cargo.toml so that
# native builds (tests, fuzzing) never produce a cdylib, which would fail to
# link sancov symbols on macOS. The --crate-type cdylib flag here is passed
# directly to rustc, overriding the manifest for this WASM-only invocation.
cargo rustc \
    -p vc-vault-contract \
    --target wasm32v1-none \
    --release \
    -- --crate-type cdylib

stellar contract optimize \
    --wasm target/wasm32v1-none/release/vc_vault_contract.wasm
