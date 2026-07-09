#!/bin/sh
set -eu

# Usage:
#   ./scripts/deploy-mainnet.sh [--dry-run]      # full deploy
#   ./scripts/deploy-mainnet.sh derive-key       # extract S… from a seed phrase
#
# Required env (loaded from .env): DEPLOYER_SECRET (S… key), DEPLOYER_ADDRESS (G…).
# Optional env overrides: DID_ADMIN FACTORY_ADMIN FEE_TOKEN FEE_DEST FEE_STANDARD FEE_ENABLED.

NETWORK_NAME=mainnet
PASSPHRASE="Public Global Stellar Network ; September 2015"
RPC_URL="${RPC_URL:-https://mainnet.sorobanrpc.com}"
HORIZON_URL="https://horizon.stellar.org"
USDC_ISSUER="GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN"
TMP_IDENTITY="_acta_deployer"

if [ -f .env ]; then
    set -a
    . ./.env
    set +a
fi

# Network-level constants.
FEE_TOKEN="${FEE_TOKEN:-CCW67TSZV3SSS2HXMBQ5JFGCKJNXKZM7UQUWUZPUTHXSTZLEO7SJMI75}"
FEE_STANDARD="${FEE_STANDARD:-10000000}"
FEE_ENABLED="${FEE_ENABLED:-true}"
INCLUSION_FEE="${INCLUSION_FEE:-1000000}"

# Identity-dependent params.
DEPLOYER_ADDRESS="${DEPLOYER_ADDRESS:-}"
DID_ADMIN="${DID_ADMIN:-}"
FACTORY_ADMIN="${FACTORY_ADMIN:-}"
FEE_DEST="${FEE_DEST:-}"

# Resume support: pre-set any of these (e.g. in .env) to skip an already-done
# step after a partial run, so a re-run doesn't redeploy/repay for it.
DID_ID="${DID_ID:-}"
VAULT_HASH="${VAULT_HASH:-}"
FACTORY_ID="${FACTORY_ID:-}"

DRY_RUN=0
SUBCOMMAND=""

usage() {
    cat >&2 <<'EOF'
Usage:
  ./scripts/deploy-mainnet.sh [--dry-run]   Deploy all three contracts to mainnet
  ./scripts/deploy-mainnet.sh derive-key    Derive a single-account S… from a seed phrase
  ./scripts/deploy-mainnet.sh -h|--help     Show this help
EOF
}

# Parse args: optional subcommand + flags.
for arg in "$@"; do
    case "$arg" in
        derive-key) SUBCOMMAND="derive-key" ;;
        --dry-run)  DRY_RUN=1 ;;
        -h|--help)  usage; exit 0 ;;
        *) echo "Unknown argument: $arg" >&2; usage; exit 1 ;;
    esac
done

say()  { printf '%s\n' "$*" >&2; }
step() { printf '\n==> %s\n' "$*" >&2; }

# Run a side-effecting command; under --dry-run just print it.
run() {
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '  [dry-run] %s\n' "$*" >&2
        return 0
    fi
    "$@"
}

# Run a command that produces a value on stdout (e.g. a contract id).
# Under --dry-run, print the command and echo the placeholder instead.
capture() {
    placeholder="$1"; shift
    if [ "$DRY_RUN" -eq 1 ]; then
        printf '  [dry-run] %s\n' "$*" >&2
        printf '%s' "$placeholder"
        return 0
    fi
    "$@"
}

# Require the deployer address from the env and default the identity-derived
# params to it. Kept out of top-level so --help works without the env set.
resolve_identity() {
    : "${DEPLOYER_ADDRESS:?set DEPLOYER_ADDRESS to the deployer G… address in .env}"
    DID_ADMIN="${DID_ADMIN:-$DEPLOYER_ADDRESS}"
    FACTORY_ADMIN="${FACTORY_ADMIN:-$DEPLOYER_ADDRESS}"
    FEE_DEST="${FEE_DEST:-$DEPLOYER_ADDRESS}"
}

print_params() {
    step "Resolved mainnet parameters"
    say "  network          : $NETWORK_NAME ($RPC_URL)"
    say "  deployer address : $DEPLOYER_ADDRESS"
    say "  DID admin        : $DID_ADMIN"
    say "  factory admin    : $FACTORY_ADMIN"
    say "  fee token (USDC) : $FEE_TOKEN"
    say "  fee dest         : $FEE_DEST"
    say "  fee standard     : $FEE_STANDARD (= $((FEE_STANDARD / 10000000)) USDC)"
    say "  fee enabled      : $FEE_ENABLED"
}

bootstrap_network() {
    step "Configuring '$NETWORK_NAME' network ($RPC_URL)"
    run stellar network add "$NETWORK_NAME" \
        --rpc-url "$RPC_URL" \
        --network-passphrase "$PASSPHRASE"
    say "  network '$NETWORK_NAME' -> $RPC_URL"
}

confirm_gate() {
    if [ "$DRY_RUN" -eq 1 ]; then
        say ""
        say "  [dry-run] skipping confirmation gate"
        return 0
    fi
    say ""
    say "This will deploy to MAINNET and spend real XLM from $DEPLOYER_ADDRESS."
    printf 'Type exactly "DEPLOY MAINNET" to continue: ' >&2
    read -r reply
    if [ "$reply" != "DEPLOY MAINNET" ]; then
        say "Aborted (got: '$reply')."
        exit 1
    fi
}

# Remove the temporary signing identity on any exit path.
cleanup() {
    stellar keys rm "$TMP_IDENTITY" --force >/dev/null 2>&1 || true
}

import_and_verify_identity() {
    step "Importing + verifying deployer identity"
    : "${DEPLOYER_SECRET:?DEPLOYER_SECRET is not set in .env}"
    stellar keys rm "$TMP_IDENTITY" --force >/dev/null 2>&1 || true
    printf '%s\n' "$DEPLOYER_SECRET" | stellar keys add "$TMP_IDENTITY" --secret-key >/dev/null
    derived="$(stellar keys public-key "$TMP_IDENTITY")"
    if [ "$derived" != "$DEPLOYER_ADDRESS" ]; then
        say "  ABORT: secret resolves to $derived but DEPLOYER_ADDRESS is $DEPLOYER_ADDRESS"
        exit 1
    fi
    say "  verified: $derived"
}

check_funding() {
    step "Checking deployer is funded (XLM for tx fees)"
    if [ "$DRY_RUN" -eq 1 ]; then
        say "  [dry-run] would GET $HORIZON_URL/accounts/$DEPLOYER_ADDRESS"
        return 0
    fi
    code="$(curl -s -o /dev/null -w '%{http_code}' "$HORIZON_URL/accounts/$DEPLOYER_ADDRESS")"
    if [ "$code" = "200" ]; then
        say "  funded (account exists on mainnet)"
    elif [ "$code" = "404" ]; then
        say "  ABORT: $DEPLOYER_ADDRESS is not funded on mainnet (Horizon 404)."
        say "  Fund it with XLM before deploying."
        exit 1
    else
        say "  ABORT: unexpected Horizon status $code for $DEPLOYER_ADDRESS"
        exit 1
    fi
}

build_all() {
    step "Building optimized WASMs"
    run sh scripts/build.sh
}

deploy_did() {
    if [ -n "$DID_ID" ]; then
        step "Skipping did-stellar-registry (using existing $DID_ID)"
        return 0
    fi
    step "Deploying did-stellar-registry (admin $DID_ADMIN)"
    DID_ID="$(capture '<DID_ID>' stellar contract deploy \
        --wasm target/wasm32v1-none/release/did_stellar_registry.optimized.wasm \
        --source "$TMP_IDENTITY" --network "$NETWORK_NAME" --inclusion-fee "$INCLUSION_FEE" \
        -- --admin "$DID_ADMIN")"
    say "  DID contract id: $DID_ID"
}

upload_vault() {
    if [ -n "$VAULT_HASH" ]; then
        step "Skipping vc-vault upload (using existing hash $VAULT_HASH)"
        return 0
    fi
    step "Uploading vc-vault template WASM"
    VAULT_HASH="$(capture '<VAULT_HASH>' stellar contract upload \
        --wasm target/wasm32v1-none/release/vc_vault_contract.optimized.wasm \
        --source "$TMP_IDENTITY" --network "$NETWORK_NAME" --inclusion-fee "$INCLUSION_FEE")"
    say "  vault template hash: $VAULT_HASH"
}

deploy_factory() {
    if [ -n "$FACTORY_ID" ]; then
        step "Skipping vc-vault-factory (using existing $FACTORY_ID)"
        return 0
    fi
    step "Deploying vc-vault-factory (admin $FACTORY_ADMIN)"
    FACTORY_ID="$(capture '<FACTORY_ID>' stellar contract deploy \
        --wasm target/wasm32v1-none/release/vc_vault_factory_contract.optimized.wasm \
        --source "$TMP_IDENTITY" --network "$NETWORK_NAME" --inclusion-fee "$INCLUSION_FEE" \
        -- --vault_init_meta "{\"vault_hash\":\"$VAULT_HASH\",\"contract_admin\":\"$FACTORY_ADMIN\"}")"
    say "  factory contract id: $FACTORY_ID"
}

# Sets EFFECTIVE_FEE_ENABLED based on the dest's USDC trustline.
check_trustline() {
    step "Checking fee dest has a USDC trustline"
    EFFECTIVE_FEE_ENABLED="$FEE_ENABLED"
    if [ "$DRY_RUN" -eq 1 ]; then
        say "  [dry-run] would check USDC trustline for $FEE_DEST"
        return 0
    fi
    body="$(curl -s "$HORIZON_URL/accounts/$FEE_DEST")"
    if printf '%s' "$body" | grep -q "$USDC_ISSUER"; then
        say "  USDC trustline present on $FEE_DEST"
    else
        say "  WARNING: $FEE_DEST has no USDC trustline."
        say "  Without it the first issue() fails when transferring the fee."
        printf 'Enable fees anyway? Type "yes" to enable, anything else defers (config written, disabled): ' >&2
        read -r ans
        if [ "$ans" != "yes" ]; then
            EFFECTIVE_FEE_ENABLED="false"
            say "  Deferring: fees will be configured but left disabled."
        fi
    fi
}

configure_fees() {
    step "Configuring factory fee (USDC)"
    run stellar contract invoke \
        --id "$FACTORY_ID" --source "$TMP_IDENTITY" --network "$NETWORK_NAME" --inclusion-fee "$INCLUSION_FEE" \
        -- set_fee_config --token "$FEE_TOKEN" --dest "$FEE_DEST" --standard "$FEE_STANDARD"
    say "  fee_config set: token=$FEE_TOKEN dest=$FEE_DEST standard=$FEE_STANDARD"
    run stellar contract invoke \
        --id "$FACTORY_ID" --source "$TMP_IDENTITY" --network "$NETWORK_NAME" --inclusion-fee "$INCLUSION_FEE" \
        -- set_fee_enabled --enabled "$EFFECTIVE_FEE_ENABLED"
    say "  fee_enabled set: $EFFECTIVE_FEE_ENABLED"
}

print_summary() {
    step "Deploy summary - record these in docs/deployments/mainnet.md"
    say "  did-stellar-registry : $DID_ID"
    say "  vc-vault template    : $VAULT_HASH"
    say "  vc-vault-factory     : $FACTORY_ID"
    say "  fee token / dest     : $FEE_TOKEN / $FEE_DEST"
    say "  fee standard / state : $FEE_STANDARD / $EFFECTIVE_FEE_ENABLED"
    if [ "$DRY_RUN" -eq 1 ]; then
        say ""
        say "  (dry-run - nothing was deployed; values above are placeholders)"
    fi
}

derive_key() {
    : "${DEPLOYER_ADDRESS:?set DEPLOYER_ADDRESS to the deployer G… address in .env}"
    tmp="_acta_derive"
    stellar keys rm "$tmp" --force >/dev/null 2>&1 || true
    say "Paste your Freighter 12/24-word recovery phrase when prompted."
    stellar keys add "$tmp" --seed-phrase >/dev/null
    found=""
    i=0
    while [ "$i" -le 30 ]; do
        addr="$(stellar keys public-key "$tmp" --hd-path "$i" 2>/dev/null)"
        if [ "$addr" = "$DEPLOYER_ADDRESS" ]; then
            found="$i"
            break
        fi
        i=$((i + 1))
    done
    if [ -z "$found" ]; then
        say "No hd-path 0..30 matched $DEPLOYER_ADDRESS."
        say "That account may have been imported separately (not from this phrase)."
        stellar keys rm "$tmp" --force >/dev/null 2>&1 || true
        exit 1
    fi
    say "Matched $DEPLOYER_ADDRESS at hd-path $found. Secret key below:"
    stellar keys secret "$tmp" --hd-path "$found"
    stellar keys rm "$tmp" --force >/dev/null 2>&1 || true
    say ""
    say "Copy it into the env, then clear your scrollback:"
    say "  read -rs DEPLOYER_SECRET && export DEPLOYER_SECRET"
}

trap cleanup EXIT INT TERM

main() {
    resolve_identity
    print_params
    bootstrap_network
    confirm_gate
    import_and_verify_identity
    check_funding
    build_all
    deploy_did
    upload_vault
    deploy_factory
    check_trustline
    configure_fees
    print_summary
}

# Dispatch.
if [ "$SUBCOMMAND" = "derive-key" ]; then
    derive_key
    exit 0
fi
main
