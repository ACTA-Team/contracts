#!/bin/sh
set -eu

# Build and optimize WASM artifacts for one or all contracts.
#
# Usage:
#   ./scripts/build.sh                      # build all contracts
#   ./scripts/build.sh vc-vault             # build a specific contract
#
# Output: target/wasm32v1-none/release/<name>.optimized.wasm

PACKAGE=${1:-all}

# vc-vault declares crate-type=["rlib"] to keep native test/fuzz builds clean,
# so we override crate-type at the rustc level for the WASM target.
build_vc_vault() {
    cargo rustc \
        -p vc-vault-contract \
        --target wasm32v1-none \
        --release \
        -- --crate-type cdylib
    stellar contract optimize \
        --wasm target/wasm32v1-none/release/vc_vault_contract.wasm
    echo "Built: target/wasm32v1-none/release/vc_vault_contract.optimized.wasm"
}

# did-stellar-registry already declares cdylib so a standard cargo build suffices.
build_did_registry() {
    cargo build \
        -p did-stellar-registry \
        --target wasm32v1-none \
        --release
    stellar contract optimize \
        --wasm target/wasm32v1-none/release/did_stellar_registry.wasm
    echo "Built: target/wasm32v1-none/release/did_stellar_registry.optimized.wasm"
}

case "$PACKAGE" in
    vc-vault)
        build_vc_vault ;;
    did-stellar-registry)
        build_did_registry ;;
    all)
        build_vc_vault
        build_did_registry ;;
    *)
        echo "Unknown package: $PACKAGE" >&2
        echo "Usage: $0 [vc-vault|did-stellar-registry|all]" >&2
        exit 1 ;;
esac
