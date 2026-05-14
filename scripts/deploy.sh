#!/bin/sh
set -eu

# Deploy a contract to a Stellar network.
#
# Usage:
#   ./scripts/deploy.sh <package> <network> <source-account>
#
#   package:        vc-vault | did-stellar-registry
#   network:        testnet | mainnet
#   source-account: stellar keys alias (e.g. acta_deployer)
#
# Examples:
#   ./scripts/deploy.sh did-stellar-registry testnet acta_deployer
#   ./scripts/deploy.sh vc-vault testnet acta_deployer
#
# Prerequisites:
#   - stellar-cli installed and configured
#   - Network already added:
#       stellar config network add testnet \
#         --rpc-url https://soroban-testnet.stellar.org:443 \
#         --network-passphrase "Test SDF Network ; September 2015"
#   - Source account key generated:
#       stellar keys generate acta_deployer --network <network>
#   - WASM built:
#       ./scripts/build.sh <package>

PACKAGE=${1:-}
NETWORK=${2:-}
SOURCE=${3:-}

if [ -z "$PACKAGE" ] || [ -z "$NETWORK" ] || [ -z "$SOURCE" ]; then
    echo "Usage: $0 <package> <network> <source-account>" >&2
    echo "  package:  vc-vault | did-stellar-registry" >&2
    echo "  network:  testnet | mainnet" >&2
    exit 1
fi

case "$PACKAGE" in
    vc-vault)
        WASM="target/wasm32v1-none/release/vc_vault_contract.optimized.wasm"
        ADMIN=${VC_ADMIN:-$(stellar keys address "$SOURCE")}
        CONSTRUCTOR_ARGS="-- --contract_admin $ADMIN"
        ;;
    did-stellar-registry)
        WASM="target/wasm32v1-none/release/did_stellar_registry.optimized.wasm"
        # Requires an admin address at construction.
        # Defaults to the deployer address; override by setting DID_ADMIN env var.
        ADMIN=${DID_ADMIN:-$(stellar keys address "$SOURCE")}
        CONSTRUCTOR_ARGS="-- --admin $ADMIN"
        ;;
    *)
        echo "Unknown package: $PACKAGE" >&2
        echo "  package: vc-vault | did-stellar-registry" >&2
        exit 1 ;;
esac

if [ ! -f "$WASM" ]; then
    echo "WASM not found: $WASM" >&2
    echo "Run: ./scripts/build.sh $PACKAGE" >&2
    exit 1
fi

echo "Deploying $PACKAGE to $NETWORK..."
echo "  WASM: $WASM"
echo "  Source: $SOURCE"

CONTRACT_ID=$(stellar contract deploy \
    --wasm "$WASM" \
    --source "$SOURCE" \
    --network "$NETWORK" \
    $CONSTRUCTOR_ARGS)

echo ""
echo "Contract ID: $CONTRACT_ID"
echo ""
echo "Add this entry to docs/deployments/$NETWORK.md:"
echo ""
echo "| $PACKAGE | $(date +%Y-%m-%d) | \`$CONTRACT_ID\` | $NETWORK |"
