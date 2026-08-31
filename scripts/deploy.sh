#!/bin/sh
set -eu

# Deploy a contract to a Stellar network.
#
# Usage:
#   ./scripts/deploy.sh <package> <network> <source-account>
#
#   package:        did-stellar-registry | vc-vault | vc-vault-factory
#   network:        testnet | mainnet
#   source-account: stellar keys alias (e.g. acta_deployer)
#
# Deployment model (since vc-vault v0.4.0):
#   - did-stellar-registry: deployed standalone (constructor: --admin).
#   - vc-vault: single-tenant TEMPLATE - not deployed standalone. Its WASM is
#     installed (uploaded) and the resulting hash is handed to the factory.
#     Individual vaults are created by calling vc-vault-factory.deploy(...).
#   - vc-vault-factory: deployed with the vault template hash + a contract admin.
#
# Examples:
#   ./scripts/deploy.sh did-stellar-registry testnet acta_deployer
#   ./scripts/deploy.sh vc-vault            testnet acta_deployer   # install template
#   ./scripts/deploy.sh vc-vault-factory    testnet acta_deployer   # installs vault + deploys factory
#
# Env overrides:
#   DID_ADMIN / FACTORY_ADMIN - admin address (defaults to the source address)
#   VAULT_HASH - reuse a pre-installed vc-vault template hash
#
# Prerequisites: stellar-cli configured, network added, source key funded, and
#   ./scripts/build.sh <package>  run first.

PACKAGE=${1:-}
NETWORK=${2:-}
SOURCE=${3:-}

if [ -z "$PACKAGE" ] || [ -z "$NETWORK" ] || [ -z "$SOURCE" ]; then
    echo "Usage: $0 <package> <network> <source-account>" >&2
    echo "  package:  did-stellar-registry | vc-vault | vc-vault-factory" >&2
    echo "  network:  testnet | mainnet" >&2
    exit 1
fi

REL="target/wasm32v1-none/release"
VAULT_WASM="$REL/vc_vault_contract.optimized.wasm"
FACTORY_WASM="$REL/vc_vault_factory_contract.optimized.wasm"
DID_WASM="$REL/did_stellar_registry.optimized.wasm"

require_wasm() {
    if [ ! -f "$1" ]; then
        echo "WASM not found: $1" >&2
        echo "Run: ./scripts/build.sh $2" >&2
        exit 1
    fi
}

case "$PACKAGE" in
    did-stellar-registry)
        require_wasm "$DID_WASM" did-stellar-registry
        ADMIN=${DID_ADMIN:-$(stellar keys address "$SOURCE")}
        echo "Deploying did-stellar-registry to $NETWORK (admin $ADMIN)..."
        CONTRACT_ID=$(stellar contract deploy \
            --wasm "$DID_WASM" --source "$SOURCE" --network "$NETWORK" \
            -- --admin "$ADMIN")
        echo ""
        echo "Contract ID: $CONTRACT_ID"
        ;;

    vc-vault)
        require_wasm "$VAULT_WASM" vc-vault
        echo "Installing vc-vault template WASM on $NETWORK..."
        HASH=$(stellar contract upload \
            --wasm "$VAULT_WASM" --source "$SOURCE" --network "$NETWORK")
        echo ""
        echo "vc-vault template WASM hash: $HASH"
        echo "Pass this to the factory (VAULT_HASH=$HASH ./scripts/deploy.sh vc-vault-factory ...)."
        ;;

    vc-vault-factory)
        require_wasm "$VAULT_WASM" vc-vault
        require_wasm "$FACTORY_WASM" vc-vault-factory
        ADMIN=${FACTORY_ADMIN:-$(stellar keys address "$SOURCE")}
        VAULT_HASH=${VAULT_HASH:-$(stellar contract upload \
            --wasm "$VAULT_WASM" --source "$SOURCE" --network "$NETWORK")}
        echo "Deploying vc-vault-factory to $NETWORK (admin $ADMIN, vault_hash $VAULT_HASH)..."
        CONTRACT_ID=$(stellar contract deploy \
            --wasm "$FACTORY_WASM" --source "$SOURCE" --network "$NETWORK" \
            -- --vault_init_meta "{\"vault_hash\":\"$VAULT_HASH\",\"contract_admin\":\"$ADMIN\"}")
        echo ""
        echo "Factory Contract ID: $CONTRACT_ID"
        echo "Vault template hash: $VAULT_HASH"
        ;;

    *)
        echo "Unknown package: $PACKAGE" >&2
        echo "  package: did-stellar-registry | vc-vault | vc-vault-factory" >&2
        exit 1 ;;
esac

echo ""
echo "Record the result in docs/deployments/$NETWORK.md"
